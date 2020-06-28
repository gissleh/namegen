use rand::{Rng};
use crate::{LearnError, WorkingSet, Sample, SampleSet};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CFGrammar {
    tokens: Vec<Token>,
    subtokens: Vec<String>,
    token_rules: Vec<TokenRule>,
    result_rules: Vec<ResultRule>,
    total_result_weight: u32,
    subtoken_frequencies: Vec<usize>,
    rlf: bool,
    ral: bool,
}

impl CFGrammar {
    pub fn generate(&self, ws: &mut WorkingSet, rng: &mut impl Rng) {
        ws.result.clear();
        ws.stack.clear();
        ws.stack_pos.clear();

        if self.result_rules.len() == 0 {
            return;
        }

        let mut result_index = 0usize;

        loop {
            // Start it off if this is the first run, or all rules failed.
            if ws.stack_pos.len() == 0 {
                result_index = self.pick_result_rule(rng);

                let token_index = self.result_rules[result_index].token_rules[0];

                ws.stack_pos.push(0);
                ws.stack.extend_from_slice(&self.token_rules[token_index].tokens);
            }

            // Find the sub-stack position.
            let stack_pos = ws.stack_pos.last().cloned().unwrap();
            if stack_pos == ws.stack.len() {
                ws.result.pop();
                ws.stack_pos.pop();
                continue;
            }

            // Take a random token off the stack.
            let stack_index = rng.gen_range(stack_pos, ws.stack.len());
            let token_index = ws.stack[stack_index];
            ws.stack.swap_remove(stack_index);

            // Check constraint: restrict adjacent subtoken. This is for dealing with subtokens like 'y'
            // that could be a consonant or vowel (prevents samples "lynaya" and "liyara" allowing
            // result "lyyana"; or "laya" and "lyna" allowing "lyya" if `rlf` is unset.)
            if self.ral && ws.result.len() > 0 {
                let prev_token = &self.tokens[*ws.result.last().unwrap()];
                let curr_token = &self.tokens[token_index];

                if curr_token.first() == prev_token.last() {
                    continue;
                }
            }

            // Stop here if this is the end.
            ws.result.push(token_index);
            if ws.result.len() == self.result_rules[result_index].token_rules.len() {
                // Combine the subtokens.
                ws.subtokens.clear();
                for i in ws.result.iter() {
                    ws.subtokens.extend_from_slice(&self.tokens[*i].subtokens());
                }

                // Check constraint: restrict subtoken frequency.
                if self.rlf {
                    let mut failed = false;
                    for (i, subtoken_index) in ws.subtokens.iter().enumerate() {
                        let mut count = 1;
                        for j in (i+1)..ws.subtokens.len() {
                            if ws.subtokens[j] == *subtoken_index {
                                count += 1;
                            }
                        }

                        if count > self.subtoken_frequencies[*subtoken_index] {
                            failed = true;
                            break;
                        }
                    }

                    if failed {
                        ws.result.pop();
                    } else {
                        ws.result_str.clear();
                        for subtoken_index in ws.subtokens.iter() {
                            ws.result_str.push_str(self.subtokens[*subtoken_index].as_str());
                        }

                        return;
                    }
                }
            } else {
                let token_rule_index = self.result_rules[result_index].token_rules[ws.result.len()];

                ws.stack_pos.push(ws.stack.len());
                ws.stack.extend_from_slice(&self.token_rules[token_rule_index].tokens);
            }
        }
    }

    fn pick_result_rule(&self, rng: &mut impl Rng) -> usize {
        let mut random = rng.gen_range(0, self.total_result_weight);

        for (i, rule) in self.result_rules.iter().enumerate() {
            if rule.weight > random {
                return i;
            } else {
                random -= rule.weight;
            }
        }

        unreachable!()
    }

    pub fn learn(&mut self, sample_set: &SampleSet) -> Result<(), LearnError> {
        // Validate sample set
        let mut tokens_len = sample_set.labels().len();
        for sample in sample_set.samples().iter() {
            match sample {
                Sample::Tokens(tokens) => {
                    if tokens.len() == 0 || (tokens_len > 0 && tokens_len != tokens.len()) {
                        return Err(LearnError::new(
                            3,
                            "Token lengths must match".to_owned(),
                            Some(sample.clone()),
                        ));
                    }
                    if tokens_len == 0 {
                        tokens_len = tokens.len()
                    }
                }

                Sample::Word(_) => {
                    return Err(LearnError::new(
                        0,
                        "Word type sample not supported".to_owned(),
                        Some(sample.clone()),
                    ));
                }
            }
        }

        // Ensure token rules
        let mut token_rule_indices: Vec<usize> = Vec::with_capacity(tokens_len);
        if sample_set.labels().len() > 0 {
            for label in sample_set.labels().iter() {
                if label.starts_with("anon_") {
                    return Err(LearnError::new(
                        4,
                        format!("Labels cannot use reserved prefix (anon_): {}", label),
                        None
                    ))
                } else if label == "*" {
                    token_rule_indices.push(self.ensure_anon_token_rule());
                } else {
                    token_rule_indices.push(self.ensure_token_rule(label));
                }
            }
        } else {
            for _ in 0..tokens_len {
                token_rule_indices.push(self.ensure_anon_token_rule());
            }
        }

        // Ensure result rule
        let result_index = self.ensure_result_rule(&token_rule_indices);

        // Add tokens
        let mut token_indices_buf = Vec::with_capacity(tokens_len);
        for sample in sample_set.samples().iter() {
            if let Sample::Tokens(tokens) = sample {
                token_indices_buf.clear();

                self.result_rules[result_index].weight += 1;
                self.total_result_weight += 1;

                for (i, token) in tokens.iter().enumerate() {
                    let token_index = self.ensure_token(token);
                    self.token_rules[token_rule_indices[i]].tokens.push(token_index);
                    token_indices_buf.push(token_index);
                }

                self.learn_subtoken_frequencies(&token_indices_buf);
            }
        }

        Ok(())
    }

    fn learn_subtoken_frequencies(&mut self, token_indices: &[usize]) {
        let mut subtoken_indices: Vec<usize> = Vec::with_capacity(token_indices.len() * 4);
        for i in token_indices {
            subtoken_indices.extend_from_slice(self.tokens[*i].subtokens());
        }

        let mut counts = vec![0; self.subtoken_frequencies.len()];
        for i in subtoken_indices {
            counts[i] += 1;
            if counts[i] > self.subtoken_frequencies[i] {
                self.subtoken_frequencies[i] = counts[i];
            }
        }
    }

    fn ensure_subtokens(&mut self, token_str: &str) -> (usize, usize) {
        for (i, subtoken) in self.subtokens.iter().enumerate() {
            if token_str.starts_with(subtoken.as_str()) {
                return (i, subtoken.len());
            }
        }

        self.subtokens.push(token_str.chars().take(1).collect());
        self.subtoken_frequencies.push(1);

        (self.subtokens.len() - 1, 1)
    }

    fn ensure_token(&mut self, token_str: &str) -> usize {
        let mut subtokens = Vec::with_capacity(16);
        let mut subtoken_pos = 0;
        while subtoken_pos < token_str.len() {
            let (subtoken_index, subtoken_len) = self.ensure_subtokens(&token_str[subtoken_pos..]);

            subtokens.push(subtoken_index);
            subtoken_pos += subtoken_len;
        }

        for (i, Token(subtokens2)) in self.tokens.iter().enumerate() {
            if subtokens2 == &subtokens {
                return i
            }
        }

        self.tokens.push(Token::new(&subtokens));

        self.tokens.len() - 1
    }

    fn ensure_anon_token_rule(&mut self) -> usize {
        self.token_rules.push(TokenRule{
            name: format!("anon_{}", self.token_rules.len()),
            tokens: Vec::with_capacity(4),
        });

        self.token_rules.len() - 1
    }

    fn ensure_token_rule(&mut self, name: &str) -> usize {
        for (i, rule) in self.token_rules.iter().enumerate() {
            if &rule.name == name {
                return i;
            }
        }

        self.token_rules.push(TokenRule{
            name: String::from(name),
            tokens: Vec::with_capacity(4),
        });

        self.token_rules.len() - 1
    }

    fn ensure_result_rule(&mut self, token_indices: &[usize]) -> usize {
        for (i, token_rule) in self.result_rules.iter().enumerate() {
            if token_rule.token_rules == token_indices {
                return i;
            }
        }

        self.result_rules.push(ResultRule{
            token_rules: token_indices.to_vec(),
            weight: 0,
        });

        self.result_rules.len() - 1
    }

    pub fn new(initial_subtokens: &[&str], rlf: bool, ral: bool) -> CFGrammar {
        CFGrammar{
            tokens: Vec::new(),
            subtokens: initial_subtokens.iter().map(|s| (*s).to_owned()).collect(),
            result_rules: Vec::new(),
            token_rules: Vec::new(),
            total_result_weight: 0,
            subtoken_frequencies: vec![1; initial_subtokens.len()],

            rlf, ral,
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
struct ResultRule {
    token_rules: Vec<usize>,
    weight: u32,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
struct TokenRule {
    name: String,
    tokens: Vec<usize>,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
struct Token (Vec<usize>);

impl Token {
    fn subtokens(&self) -> &[usize] {
        &self.0
    }

    fn first(&self) -> usize {
        self.0.first().cloned().unwrap_or(0)
    }

    fn last(&self) -> usize {
        self.0.last().cloned().unwrap_or(0)
    }

    fn new(subtoken_indices: &[usize]) -> Token {
        Token(subtoken_indices.to_vec())
    }
}