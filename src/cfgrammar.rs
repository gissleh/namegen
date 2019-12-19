use rand::{Rng};
use super::core::WorkingSet;

pub struct CFGrammar {
    tokens: Vec<Token>,
    letters: Vec<Letter>,
    token_rules: Vec<TokenRule>,
    result_rules: Vec<ResultRule>,
    total_weight: u32,
}

impl CFGrammar {
    pub fn generate(&self, ws: &mut WorkingSet) {
        ws.result.clear();
        ws.stack.clear();
        ws.stack_pos.clear();


    }

    pub fn learn(&mut self, token_rules: &[Option<&str>], token_lists: &[&[&str]]) -> Result<(), &'static str> {
        let mut token_rule_indexes: Vec<usize> = Vec::with_capacity(token_rules.len());
        let mut result_rule_index = self.result_rules.len();
        let mut had_anon = false;

        if token_rules.len() == 0 || token_rules.len() != token_lists.len() {
            return Err("Invalid lengths, they must be non-zero and match up")
        } else if token_lists.len() > 1 {
            let first_length = token_lists[0].len();
            
            for tokens in token_lists.iter().skip(1) {
                if tokens.len() != first_length {
                    return Err("One of the token lists has a mismatching length")
                }
            }
        }

        // Find token rules
        for token_rule in token_rules.iter() {
            let index = match token_rule {
                Some(s) => self.ensure_token_rule(s),
                None => {
                    had_anon = true;
                    self.ensure_anon_token_rule()
                },
            };

            token_rule_indexes.push(index);
        }

        // Extend token rules
        for (rule_index, tokens) in token_rule_indexes.iter().zip(token_lists).map(|(r, t)| (*r, *t)) {
            for token in tokens.iter() {
                let token_index = self.ensure_token(token);
                let token = &mut self.tokens[token_index];

                self.token_rules[rule_index].tokens.push(token_index);
            }
        }

        // Find result rules (this takes ownership of token_rule_indexes)
        if !had_anon {
            result_rule_index = self.find_result_rule(&token_rule_indexes)
        }
        if result_rule_index == self.result_rules.len() {
            self.result_rules.push(ResultRule{
                token_rules: token_rule_indexes,
                weight: 0,
            });
        }
        self.result_rules[result_rule_index].weight += 1;
        self.total_weight += 1;

        Ok(())
    }

    fn find_next_letter(&self, remainder: &str) -> Option<usize> {       
        self.letters.iter().enumerate().skip(1)
                    .filter(|(_, l)| remainder.starts_with(&l.value))
                    .map(|(i, _)| i )
                    .next()
    }

    fn find_result_rule(&self, token_rules: &[usize]) -> usize {
        for (i, rule) in self.result_rules.iter().enumerate() {
            if rule.token_rules == token_rules {
                return i
            }
        }

        self.result_rules.len()
    }

    fn ensure_letters(&mut self, token_str: &str, buf: &mut Vec<usize>) {
        
    }

    fn ensure_token(&mut self, token_str: &str) -> usize {
        for (i, t) in self.tokens.iter().enumerate() {
            if &t.value == token_str {
                return i
            }
        }

        self.tokens.push(Token::from(token_str));

        self.tokens.len() - 1
    }

    fn ensure_anon_token_rule(&mut self) -> usize {
        let name = format!("__anon_{}", self.token_rules.len());

        self.token_rules.push(TokenRule{
            name: name.clone(),
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
}

struct ResultRule {
    token_rules: Vec<usize>,
    weight: u32,
}

struct TokenRule {
    name: String,
    tokens: Vec<usize>,
}

struct Token {
    value: String,
    letters: Vec<usize>,
    first: char,
    last: char,
}

struct Letter {
    value: String,
}

impl Token {
    fn from(value: &str) -> Token {
        Token{
            value: String::from(value),
            letters: Vec::with_capacity(value.chars().count()),
            first: value.chars().next().unwrap_or('\0'),
            last: value.chars().last().unwrap_or('\0'),
        }
    }
}