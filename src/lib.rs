#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

use rand::Rng;
use std::error::Error;

pub use crate::markov::Markov;
pub use crate::core::{WorkingSet, Sample, SampleSet, LearnError};
pub use crate::formatting::{FormattingRule, format_ws};
pub use crate::cfgrammar::CFGrammar;

mod formatting;
mod core;
mod cfgrammar;
mod markov;

// TODO: Figure out how to make this serializable with a trait instead of this manual stuff.

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum PartGenerator {
    #[cfg_attr(feature = "serde", serde(rename="markov"))]
    Markov(Markov),
    #[cfg_attr(feature = "serde", serde(rename="cfgrammar"))]
    CFGrammar(CFGrammar),
}

impl PartGenerator {
    fn generate(&self, ws: &mut WorkingSet, rng: &mut impl Rng) {
        match self {
            PartGenerator::Markov(m) => m.generate(ws, rng),
            PartGenerator::CFGrammar(c) => c.generate(ws, rng),
        }
    }

    fn learn(&mut self, sample_set: &SampleSet) -> Result<(), LearnError> {
        match self {
            PartGenerator::Markov(m) => m.learn(sample_set),
            PartGenerator::CFGrammar(c) => c.learn(sample_set),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
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

    pub fn learn(&mut self, sample_set: &SampleSet) -> Result<(), impl Error> {
        self.generator.learn(sample_set)
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

    pub fn new_cfgrammar(name: &str, format_rules: &[FormattingRule], initial_subtokens: &[&str], rlf: bool, ral: bool) -> NamePart {
        NamePart {
            name: name.to_owned(),
            format_rules: format_rules.to_vec(),
            generator: PartGenerator::CFGrammar(
                CFGrammar::new(initial_subtokens, rlf, ral),
            )
        }
    }
}