use std::error::Error;

/// A WorkingSet is a crucial part of this generator's performance. It is all local state required
/// to generate a name and get the output without performing additional allocations per generation
/// once the WorkingSet's underlying vectors have grown.
/// 
/// This also means that if you have a working set per thread, then generating names is completely
/// thread safe.
///
/// WARNING: While the fields are public, they're not guaranteed in any way. You should always
/// use [`WorkingSet::new()`] to create new working sets.
pub struct WorkingSet {
    pub result: Vec<usize>,
    pub result_str: String,
    pub result_chars: Vec<char>,
    pub result_total: String,
    pub stack: Vec<usize>,
    pub stack_pos: Vec<usize>,
    pub subtokens: Vec<usize>,
}

impl WorkingSet {
    /// Get the results from the last generator call.
    /// If you need to keep it around, copy it to another
    /// string.
    pub fn get_result(&self) -> &str {
        &self.result_str
    }

    pub fn new() -> WorkingSet {
        WorkingSet{
            result: Vec::with_capacity(16),
            result_str: String::with_capacity(16),
            result_chars: Vec::with_capacity(64),
            result_total: String::with_capacity(32),
            stack: Vec::with_capacity(128),
            stack_pos: Vec::with_capacity(16),
            subtokens: Vec::new(),
        }
    }
}

#[derive(Clone, std::fmt::Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Sample {
    Word(String),
    Tokens(Vec<String>),
}

#[derive(Clone, std::fmt::Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SampleSet {
    labels: Vec<String>,
    samples: Vec<Sample>,
}

impl SampleSet {
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    pub fn add_sample(&mut self, sample: Sample) {
        self.samples.push(sample);
    }

    pub fn new(labels: &[&str]) -> SampleSet {
        SampleSet{
            samples: Vec::new(),
            labels: labels.iter().map(|s| (*s).to_owned()).collect(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LearnError {
    sample: Option<Sample>,
    code: usize,
    desc: String,
}

impl LearnError {
    pub fn new(code: usize, desc: String, sample: Option<Sample>) -> LearnError {
        LearnError{code, desc, sample}
    }
}

impl std::fmt::Display for LearnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.sample {
            Some(sample) => write!(f, "LearError {:?}: {} (code: {})", sample, self.desc, self.code),
            None => write!(f, "LearError: {} (code: {})", self.desc, self.code),
        }
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