#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

pub use crate::sample::{Sample, SampleSet};
pub use crate::markov::Markov;
pub use crate::core::{WorkingSet, LearnError};
pub use crate::formatting::{FormattingRule, format_string};
pub use crate::cfgrammar::CFGrammar;
pub use crate::name::{Name};
pub use crate::part::{NamePart};

mod formatting;
mod core;
mod cfgrammar;
mod markov;
mod name;
mod part;
mod sample;
