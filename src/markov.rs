use rand::{Rng};
use crate::{Sample, SampleSet, WorkingSet, LearnError};
use crate::core::ValidationError;
use std::collections::HashSet;

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Markov {
    tokens: Vec<String>,
    max_tokens: Vec<usize>,
    starts: Vec<StartNode>,
    total_starts: usize,
    nodes: Vec<Node>,
    lengths: Vec<usize>,
    total_lengths: usize,

    lrs: bool,
    lrm: bool,
    lre: bool,
    rtf: bool,
}

impl Markov {
    #[allow(dead_code)]
    fn print_node(&self, index: usize, depth: usize) {
        if depth == 12 {
            println!("                        ...");
            return;
        }

        let node = &self.nodes[index];

        println!("{}{} l={} w={} e={} i={}", 
            "  ".repeat(depth),
            self.tokens[node.token],
            node.length,
            node.weight,
            node.ending,
            index,
        );

        for child in node.children.iter() {
            self.print_node(*child, depth + 1);
        }
    }
    
    #[allow(dead_code)]
    pub fn print(&self) {
        for start in self.starts.iter() {
            println!("{}{} l={} w={}", 
                self.tokens[start.tokens.0],
                self.tokens[start.tokens.1],
                start.length,
                start.weight,
            );

            for node_index in start.children.iter().cloned() {
                self.print_node(node_index, 1);
            }
        }
    }

    fn find_next_token(&self, remainder: &str) -> Option<usize> {       
        self.tokens.iter().enumerate().skip(1)
                   .filter(|(_, t)| remainder.starts_with(*t))
                   .map(|(i, _)| i )
                   .next()
    }

    fn pick_length(&self, rng: &mut impl Rng) -> usize {
        let mut random = rng.gen_range(0, self.total_lengths);

        3 + self.lengths.iter().enumerate().filter(|(_, s)| {
            if **s > random {
                true
            } else {
                random -= **s;
                false
            }
        }).map(|(i, _)| i).next().unwrap_or(0)
    }

    fn pick_start(&self, rng: &mut impl Rng) -> usize {
        let mut random = rng.gen_range(0, self.total_starts);

        self.starts.iter().enumerate().filter(|(_, s)| {
            if s.weight > random {
                true
            } else {
                random -= s.weight;
                false
            }
        }).map(|(i, _)| i).next().unwrap_or(0)
    }

    /// Generate a name. You need to provide your own WorkingSet and Rng, which is necessary to save
    /// on allocations. A dependent application should use the full name generator interface instead
    pub fn generate(&self, ws: &mut WorkingSet, rng: &mut impl Rng) {
        if self.starts.len() == 0 {
            return
        }

        let mut length = 1;

        ws.stack_pos.clear();
        ws.result.clear();

        while ws.result.len() < length {
            // Start if the stack is empty.
            if ws.stack_pos.len() == 0 {
                let start_index = self.pick_start(rng);
                let start = &self.starts[start_index];

                ws.result.clear();
                ws.stack.clear();

                ws.result.push(start.tokens.0);
                ws.result.push(start.tokens.1);
                ws.stack.extend(start.children.iter());
                ws.stack_pos.push(0);
                ws.stack_weight.push(start.children.iter().map(|ci| self.nodes[*ci].weight).sum());

                length = if self.lrs { start.length } else { self.pick_length(rng) };
            }

            // Get the last one.
            let pos = *ws.stack_pos.last().unwrap();
            let weight = *ws.stack_weight.last().unwrap();
            if ws.stack.len() == pos {
                ws.stack_pos.pop();
                ws.stack_weight.pop();
                ws.result.pop();
                continue;
            }

            // Pick a available child node.
            let mut r = rng.gen_range(0, weight);
            let mut node_index = pos;
            loop {
                let node = &self.nodes[ws.stack[node_index]];
                if r < node.weight {
                    #[cfg(debug_assertions)]
                    assert!(node_index < ws.stack.len());

                    break;
                }

                r -= node.weight;
                node_index += 1;
            }

            let node = &self.nodes[ws.stack[node_index]];
            ws.stack.swap_remove(node_index);
            *ws.stack_weight.last_mut().unwrap() -= node.weight;

            // Only accept endings at the end.
            let ending = ws.result.len() == length - 1;
            if node.ending != ending {
                continue;
            }
            if self.lre && ending && length != node.length {
                continue;
            }

            // Handle token frequency restriction.
            if self.rtf {
                let count = 1 + ws.result.iter().filter(|c| **c == node.token).count();
                if count > self.max_tokens[node.token] {
                    continue;
                }
            }

            // Push the token
            ws.result.push(node.token);
            ws.stack_pos.push(ws.stack.len());
            ws.stack_weight.push(node.children.iter().map(|ci| self.nodes[*ci].weight).sum());
            ws.stack.extend(node.children.iter());
        };

        ws.result_str.clear();
        for s in ws.result.iter().map(|i| &self.tokens[*i]) {
            ws.result_str.push_str(s);
        }
    }

    /// Learn learns samples from the sample set. The former state is copied and will
    /// be restored upon one of the samples failing to import.
    pub fn learn(&mut self, sample_set: &SampleSet) -> Result<(), LearnError>  {
        let old_state = self.clone();
        for sample in sample_set.samples() {
            if let Err(err) = self.learn_norecalc(sample) {
                *self = old_state;
                return Err(err);
            }
        }

        self.recalculate_weights();

        Ok(())
    }

    /// Learn rules from the sample. The generation is heavily optimized for speed, but `learn` is
    /// paying for that speed.
    pub fn learn_one(&mut self, sample: &Sample) -> Result<(), LearnError> {
        if let Err(err) = self.learn_norecalc(sample) {
            return Err(err)
        }

        self.recalculate_weights();

        Ok(())
    }

    fn learn_norecalc(&mut self, sample: &Sample) -> Result<(), LearnError> {
        let sample_string: &str;
        match sample {
            Sample::Word(s) => sample_string = &s,
            Sample::WordWeighted(s, _) => sample_string = &s,
            _ => {
                return Err(LearnError::new(
                    1,
                    format!("Incorrect sample type. Must be Word"),
                    Some(sample.clone()),
                ));
            },
        }

        let mut remainder = sample_string;
        let mut tokens: Vec<usize> = Vec::with_capacity(sample_string.len());

        // Find and learn new tokens.
        while remainder.len() > 0 {
            let token_index;
            if let Some(index) = self.find_next_token(remainder) {
                token_index = index;
                remainder = &remainder[self.tokens[index].len()..];
            } else {
                token_index = self.tokens.len();
                self.tokens.push(String::from(&remainder[0..1]));
                self.max_tokens.push(0);
                remainder = &remainder[1..];
            }

            tokens.push(token_index);
        }
        if tokens.len() < 3 {
            return Err(LearnError::new(
                0,
                format!("3 or more tokens required ({} provided)", tokens.len()),
                Some(sample.clone()),
            ));
        }

        // Learn token frequencies if that is restricted.
        if self.rtf {
            for token in tokens.iter() {
                let count = tokens.iter().filter(|t| **t == *token).count();

                if self.max_tokens[*token] < count {
                    self.max_tokens[*token] = count;
                }
            }
        }

        // Learn start
        let start_tokens = (tokens[0], tokens[1]);
        let start_length = if self.lrs { tokens.len() } else { 0 };
        let start_index;
        if let Some((i, start)) = self.starts.iter_mut().enumerate().find(|(_, s)| s.tokens == start_tokens && s.length == start_length) {
            start_index = i;
            start.weight += 1;
        } else {
            start_index = self.starts.len();
            self.starts.push(StartNode{
                tokens: start_tokens,
                weight: 1,
                length: if self.lrs { tokens.len() } else { 0 },
                children: Vec::with_capacity(8),
            });
        }
        self.total_starts += 1;

        // Learn length
        let length_index = tokens.len() - 3;
        while self.lengths.len() <= length_index {
            self.lengths.push(0);
        }
        self.lengths[length_index] += 1;
        self.total_lengths += 1;

        // Learn rest of name.
        let mut prev = start_tokens;
        let length_m = if self.lrm { tokens.len() } else { 0 };
        let length_e = if self.lre { tokens.len() } else { 0 };
        for (i, token) in tokens.iter().cloned().enumerate().skip(2) {
            let ending = i == (tokens.len() - 1);
            let length = if ending { length_e } else { length_m };
            let current_index;

            if let Some(node_index) = Node::find_current(&self.nodes, prev, token, length, ending) {
                current_index = node_index;
            } else {
                current_index = self.nodes.len();
                self.nodes.push(Node{
                    prev, token, length, ending,
                    children: Vec::with_capacity(8),
                    weight: 1,
                })
            }

            if i > 2 {
                for (_, node) in Node::list_prev(&mut self.nodes, prev, length_m) {
                    if !node.children.contains(&current_index) {
                        node.children.push(current_index);
                    }
                }
            } else {
                if !self.starts[start_index].children.contains(&current_index) {
                    self.starts[start_index].children.push(current_index);
                }
            }

            prev = (prev.1, token);
        }

        Ok(())
    }

    pub fn recalculate_weights(&mut self) {
        let mut round: Vec<usize> = Vec::with_capacity(64);
        let mut next_round: HashSet<usize> = HashSet::new();
        let mut explored: HashSet<usize> = HashSet::new();

        for i in 0..self.nodes.len() {
            if !self.nodes[i].ending {
                continue;
            }

            self.nodes[i].weight = 1;
            next_round.insert(i);
        }

        while next_round.len() > 0 {
            round.clear();
            round.extend(next_round.iter());
            next_round.clear();

            for i in round.iter().cloned() {
                explored.insert(i);

                for j in 0..self.nodes.len() {
                    if self.nodes[j].has_child(i) {
                        self.nodes[j].weight += 1;

                        if !explored.contains(&j) {
                            next_round.insert(j);
                        }
                    }
                }
            }
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        let total_length: usize = self.lengths.iter().sum();
        if total_length != self.total_lengths {
            return Err(ValidationError::new("parts::Markov", "total_lengths is not accurate."))
        }

        let total_starts: usize = self.starts.iter().map(|s| s.weight).sum();
        if total_starts != self.total_starts {
            return Err(ValidationError::new("parts::Markov", "total_starts is not accurate."))
        }

        for start in self.starts.iter() {
            if start.length == 0 && self.lrs {
                return Err(ValidationError::new("parts::Markov", "start.length cannot be zero if lrs is true."))
            }

            for ch_i in start.children.iter() {
                if *ch_i >= self.nodes.len() {
                    return Err(ValidationError::new("parts::Markov", "start has out of range child."))
                }
            }

            let (st1, st2) = start.tokens;
            if st1 >= self.tokens.len() || st2 >= self.tokens.len() {
                return Err(ValidationError::new("parts::Markov", "start has out of range token."))
            }
        }

        for node in self.nodes.iter() {
            if node.length == 0 && (if node.ending {self.lre} else {self.lrm}) == true {
                return Err(ValidationError::new("parts::Markov", "start.length cannot be zero if lrm/lre is true."))
            }

            for ch_i in node.children.iter() {
                if *ch_i >= self.nodes.len() {
                    return Err(ValidationError::new("parts::Markov", "start has out of range child."))
                }
            }

            if node.token >= self.tokens.len() {
                return Err(ValidationError::new("parts::Markov", "node has out of range token."))
            }

            if node.weight == 0 {
                return Err(ValidationError::new("parts::Markov", "node has zero weight."))
            }

            if node.ending && node.weight != 1 {
                return Err(ValidationError::new("parts::Markov", "ending node cannot have weight <> 1."))
            }

            if node.ending && node.children.len() > 0 {
                return Err(ValidationError::new("parts::Markov", "ending node cannot have children."))
            }

            let (pt1, st2) = node.prev;
            if pt1 >= self.nodes.len() || st2 >= self.nodes.len() {
                return Err(ValidationError::new("parts::Markov", "node has out of range prev."))
            }
        }

        Ok(())
    }

    /// Create a new generator without any pre-defined tokens and constraints.
    pub fn new() -> Markov {
        let tokens: Vec<String> = Vec::new();
        Self::with_constraints(&tokens, false, false, false, false)
    }

    /// Create a new generator with pre-defined tokens and no constraints. The tokens allow you
    /// to define vowel pairs (e.g. ae, ay, ey), digraphs (e.g. th, nth, ng) so that they're treated
    /// as one token.
    pub fn with_tokens<S: AsRef<str>>(tokens: &[S]) -> Markov {
        Self::with_constraints(tokens, false, false, false, false)
    }

    /// Create a new generator with both pre-defined tokens and constraints. The constraints
    /// increases the faithfulness of the generator to the sample material, but at the cost of
    /// variety.
    pub fn with_constraints<S: AsRef<str>>(tokens: &[S], lrs: bool, lrm: bool, lre: bool, rtf: bool) -> Markov {
        Markov{
            tokens: tokens.iter().map(|d| d.as_ref().to_owned()).collect(),
            max_tokens: vec![0usize; tokens.len()],
            nodes: Vec::with_capacity(64),
            starts: Vec::with_capacity(16),
            total_starts: 0,

            lengths: vec![0usize; 8],
            total_lengths: 0,

            lrs, lrm, lre, rtf,
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
struct Node {
    #[cfg_attr(feature = "serde", serde(rename="p"))]
    prev: (usize, usize),
    #[cfg_attr(feature = "serde", serde(rename="t"))]
    token: usize,
    #[cfg_attr(feature = "serde", serde(rename="w"))]
    weight: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(rename="l"))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if="is_zero"))]
    length: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(rename="c"))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if="Vec::is_empty"))]
    children: Vec<usize>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(rename="e"))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if="is_false"))]
    ending: bool,
}

impl Node {
    pub fn has_child(&self, index: usize) -> bool {
        self.children.iter().find(|p| **p == index).is_some()
    }

    fn list_prev(list: &mut [Node], prev: (usize, usize), length: usize) -> impl Iterator<Item=(usize, &mut Node)> {
        list.iter_mut().enumerate().filter(move |(_, n)| n.length == length && n.prev.1 == prev.0 && n.token == prev.1 && n.ending == false)
    }

    fn find_current(list: &[Node], prev: (usize, usize), current: usize, length: usize, ending: bool) -> Option<usize> {
        if length > 0 {
            list.iter().enumerate().filter(|(_, n)| n.length == length && n.ending == ending && n.prev == prev && n.token == current).map(|(i, _)| i).next()        
        } else {
            list.iter().enumerate().filter(|(_, n)| n.ending == ending && n.prev == prev && n.token == current).map(|(i, _)| i).next()                    
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
struct StartNode {
    #[cfg_attr(feature = "serde", serde(rename="t"))]
    tokens: (usize, usize),
    #[cfg_attr(feature = "serde", serde(rename="w"))]
    weight: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(rename="l"))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if="is_zero"))]
    length: usize,
    #[cfg_attr(feature = "serde", serde(rename="c"))]
    children: Vec<usize>,
}

/// This is only used for serialize
#[allow(clippy::trivially_copy_pass_by_ref, dead_code)]
fn is_false(v: &bool) -> bool {
    !*v
}

/// This is only used for serialize
#[allow(clippy::trivially_copy_pass_by_ref, dead_code)]
fn is_zero(v: &usize) -> bool {
    *v == 0
}
