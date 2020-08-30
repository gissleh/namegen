use rand::Rng;

use crate::{Markov, CFGrammar, FormattingRule, WorkingSet, SampleSet, LearnError, WordList};
use crate::formatting::format_ws;
use crate::core::ValidationError;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
enum PartGenerator {
    #[cfg_attr(feature = "serde", serde(rename="markov"))]
    Markov(Markov),
    #[cfg_attr(feature = "serde", serde(rename="cfgrammar"))]
    CFGrammar(CFGrammar),
    #[cfg_attr(feature = "serde", serde(rename="wordlist"))]
    WordList(WordList),
}

impl PartGenerator {
    fn generate(&self, ws: &mut WorkingSet, rng: &mut impl Rng) {
        match self {
            PartGenerator::Markov(m) => m.generate(ws, rng),
            PartGenerator::CFGrammar(c) => c.generate(ws, rng),
            PartGenerator::WordList(wl) => wl.generate(ws, rng),
        }
    }

    fn learn(&mut self, sample_set: &SampleSet) -> Result<(), LearnError> {
        match self {
            PartGenerator::Markov(m) => m.learn(sample_set),
            PartGenerator::CFGrammar(c) => c.learn(sample_set),
            PartGenerator::WordList(wl) => wl.learn(sample_set),
        }
    }

    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            PartGenerator::Markov(m) => m.validate(),
            PartGenerator::CFGrammar(c) => c.validate(),
            PartGenerator::WordList(wl) => wl.validate(),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Clone)]
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

    pub fn learn(&mut self, sample_set: &SampleSet) -> Result<(), LearnError> {
        self.generator.learn(sample_set)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.generator.validate().map_err(|err| err.with_name(&self.name))
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

    pub fn new_wordlist(name: &str, format_rules: &[FormattingRule]) -> NamePart {
        NamePart {
            name: name.to_owned(),
            format_rules: format_rules.to_vec(),
            generator: PartGenerator::WordList(
                WordList::new(),
            )
        }
    }
}