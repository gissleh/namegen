use serde::export::fmt::{Display};
use serde::export::Formatter;
use std::error::Error;

/// A WorkingSet is a crucial part of this generator's performance. It is all local state required
/// to generate a name and get the output without performing additional allocations per generation
/// once the WorkingSet's underlying vectors have grown.
/// 
/// This also means that if you have a working set per thread, then generating names is completely
/// thread safe.
pub struct WorkingSet {
    pub result: Vec<usize>,
    pub result_str: String,
    pub result_chars: Vec<char>,
    pub stack: Vec<usize>,
    pub stack_pos: Vec<usize>,
}

impl WorkingSet {
    /// Get the results from the last generator call.
    /// If you need to keep it around, copy it to another
    /// string.
    #[inline]
    pub fn get_result(&self) -> &str {
        &self.result_str
    }

    pub fn new() -> WorkingSet {
        WorkingSet{
            result: Vec::with_capacity(16),
            result_str: String::with_capacity(16),
            result_chars: Vec::with_capacity(64),
            stack: Vec::with_capacity(128),
            stack_pos: Vec::with_capacity(16),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, std::fmt::Debug)]
#[serde(tag = "t", content = "p")]
pub enum Sample {
    Word(String),
    Tokens(Vec<String>),
    LabeledTokens{labels: Vec<String>, tokens: Vec<String>}
}

#[derive(Debug)]
pub struct LearnError {
    sample: Sample,
    code: usize,
    desc: String,
}

impl LearnError {
    pub fn new(code: usize, desc: String, sample: Sample) -> LearnError {
        LearnError{code, desc, sample}
    }
}

impl Display for LearnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LearError {:?}: {} (code: {})", self.sample, self.desc, self.code)
    }
}

impl Error for LearnError {
    fn description(&self) -> &str {
        &self.desc
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}