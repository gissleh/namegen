#[macro_use]
extern crate serde_derive;

use rand::Rng;
use crate::markov::Markov;
use crate::core::{WorkingSet, Sample};
use crate::formatting::{FormattingRule, format_ws};
use std::error::Error;

pub mod formatting;
pub mod core;
pub mod cfgrammar;
pub mod markov;

enum PartGenerator {
    Markov(Markov),
}

impl PartGenerator {
    fn generate(&self, ws: &mut WorkingSet, rng: &mut impl Rng) {
        match self {
            PartGenerator::Markov(m) => m.generate(ws, rng)
        }
    }

    fn learn(&mut self, sample: &Sample) -> Result<(), impl Error> {
        match self {
            PartGenerator::Markov(m) => m.learn(sample)
        }
    }
}

pub struct NamePart {
    name: String,
    generator: PartGenerator,
    format_rules: Vec<FormattingRule>,
}

impl NamePart {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn generate(&self, ws: &mut WorkingSet, rng: &mut impl Rng) {
        self.generator.generate(ws, rng);
        format_ws(ws, &self.format_rules);
    }

    pub fn learn(&mut self, sample: &Sample) -> Result<(), impl Error> {
        self.generator.learn(sample)
    }

    pub fn new_markov(name: &str, format_rules: &[FormattingRule], initial_tokens: &[&str], lrs: bool, lrm: bool, lre: bool, rlf: bool) -> NamePart {
        NamePart {
            name: name.to_owned(),
            format_rules: format_rules.to_vec(),
            generator: PartGenerator::Markov(
                Markov::with_constraints(initial_tokens, lrs, lrm, lre, rlf),
            ),
        }
    }
}