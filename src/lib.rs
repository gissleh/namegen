#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

pub use crate::sample::{Sample, SampleSet};
pub use crate::core::{WorkingSet, LearnError};
pub use crate::formatting::{FormattingRule, format_string};
pub use crate::cfgrammar::CFGrammar;
pub use crate::markov::Markov;
pub use crate::wordlist::WordList;
pub use crate::name::{Name, NameFormat};
pub use crate::part::{NamePart};

mod formatting;
mod core;
mod cfgrammar;
mod wordlist;
mod markov;
mod name;
mod part;
mod sample;
