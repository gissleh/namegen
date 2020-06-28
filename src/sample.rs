#[derive(Clone, std::fmt::Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Sample {
    Word(String),
    Tokens(Vec<String>),
}

#[derive(Clone, std::fmt::Debug)]
#[cfg_attr(target = "wasm32-unknown-unknown", wasm_bindgen)]
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
