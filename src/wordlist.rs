use rand::{Rng};
use crate::{LearnError, WorkingSet, Sample, SampleSet};

/// WList is a simple word-list generator. It's probably not what you came here for, but some name
/// parts are best filled with a word-list. It supports weighted words, so that you can make common
/// words appear more often.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct WordList {
    rules: Vec<Rule>,
    one_cutoff_index: usize,
    one_cutoff_weight: u32,
    total_weight: u32,
}

impl WordList {
    /// Generate a name. You need to provide your own WorkingSet and Rng, which is necessary to save
    /// on allocations. A dependent application should use the full name generator interface instead
    pub fn generate(&self, ws: &mut WorkingSet, rng: &mut impl Rng) {
        if self.total_weight == 0 {
            ws.result_str.clear();
            return;
        }

        let roll: u32 = rng.gen_range(0, self.total_weight);

        return self.generate_with_roll(&mut ws.result_str, roll);
    }

    fn generate_with_roll(&self, target: &mut String, roll: u32) {
        let mut roll = roll;

        if roll > self.one_cutoff_weight {
            let rule = &self.rules[self.one_cutoff_index + ((roll - self.one_cutoff_weight) as usize)];

            target.clear();
            target.push_str(&rule.name);
        } else {
            for rule in self.rules.iter() {
                if roll < rule.weight {
                    target.clear();
                    target.push_str(&rule.name);
                    return
                }

                roll -= rule.weight;
            }

            target.clear();
        }
    }

    /// Learn learns samples from the sample set. The former state is copied and will
    /// be restored upon one of the samples failing to import.
    pub fn learn(&mut self, sample_set: &SampleSet) -> Result<(), LearnError>  {
        let old_state = self.clone();
        for sample in sample_set.samples() {
            if let Err(err) = self.learn_one(sample) {
                *self = old_state;
                return Err(err);
            }
        }

        Ok(())
    }

    fn learn_one(&mut self, sample: &Sample) -> Result<(), LearnError> {
        let sample_weight: u32;
        let sample_word: &str;

        match sample {
            Sample::Word(w) => {
                sample_word = &w;
                sample_weight = 1;
            },
            Sample::WordWeighted(w, n) => {
                sample_word = &w;
                sample_weight = *n;
            },
            _ => {
                return Err(LearnError::new(
                    1,
                    format!("Incorrect sample type. Must be Word"),
                    Some(sample.clone()),
                ));
            },
        }

        self.add_rule(Rule{
            name: sample_word.to_owned(),
            weight: sample_weight,
        });

        Ok(())
    }

    fn add_rule(&mut self, new_rule: Rule) {
        let mut index = self.rules.len();
        for (i, rule) in self.rules.iter().enumerate() {
            if rule.name == new_rule.name {
                index = i;
                break;
            }
        }

        self.total_weight += new_rule.weight;

        if index == self.rules.len() {
            if new_rule.weight == 1 {
                self.rules.push(new_rule);
            } else {
                let last_index = self.rules.len();

                self.one_cutoff_weight += new_rule.weight;
                self.rules.push(new_rule);
                if self.one_cutoff_index < last_index {
                    self.rules.swap(self.one_cutoff_index, last_index);
                }
                self.one_cutoff_index += 1;
            }
        } else {
            if self.rules[index].weight > 1 {
                self.rules[index].weight += new_rule.weight;
                self.one_cutoff_weight += new_rule.weight;
            } else {
                let last_index = self.rules.len() - 1;

                self.rules[index].weight += new_rule.weight;
                if self.one_cutoff_index < last_index {
                    self.rules.swap(self.one_cutoff_index, index);
                }
                self.one_cutoff_index += 1;
                self.one_cutoff_weight += new_rule.weight + 1;
            }
        }
    }

    pub fn validate(&mut self) {
        let rules_copy = self.rules.clone();

        self.rules.clear();
        self.one_cutoff_weight = 0;
        self.one_cutoff_index = 0;
        self.total_weight = 0;

        rules_copy.into_iter().for_each(|r| self.add_rule(r));
    }

    pub fn new() -> WordList {
        WordList {
            rules: Vec::with_capacity(16),
            one_cutoff_weight: 0,
            one_cutoff_index: 0,
            total_weight: 0,
        }
    }
}

#[derive(Clone, std::fmt::Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct Rule {
    #[cfg_attr(feature = "serde", serde(rename="n"))]
    name: String,
    #[cfg_attr(feature = "serde", serde(rename="w"))]
    weight: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate() {
        let mut wlist = WordList::new();
        let mut res = String::with_capacity(8);

        wlist.learn_one(&sw("stuff", 1)).unwrap();
        wlist.learn_one(&sw("things", 2)).unwrap();
        wlist.learn_one(&sw("objects", 4)).unwrap();
        wlist.learn_one(&sw("items", 16)).unwrap();
        wlist.learn_one(&sw("artifact", 1)).unwrap();

        for i in 0..2 {
            wlist.generate_with_roll(&mut res, i);
            assert_eq!(&res, "things", "things ({}/2)", i + 1);
        }
        for i in 2..6 {
            wlist.generate_with_roll(&mut res, i);
            assert_eq!(&res, "objects", "objects ({}/4)", i + 1);
        }
        for i in 6..22 {
            wlist.generate_with_roll(&mut res, i);
            assert_eq!(&res, "items", "items ({}/16)", i + 1);
        }

        wlist.generate_with_roll(&mut res, 22);
        assert_eq!(&res, "stuff");
        wlist.generate_with_roll(&mut res, 23);
        assert_eq!(&res, "artifact");
    }

    #[test]
    fn test_cutoff() {
        let mut wlist = WordList::new();
        assert_eq!(wlist.rules.as_slice(), &[]);
        assert_eq!(wlist.one_cutoff_index, 0);
        assert_eq!(wlist.one_cutoff_weight, 0);
        assert_eq!(wlist.total_weight, 0);

        wlist.learn_one(&s("stuff")).unwrap();
        assert_eq!(wlist.rules.as_slice(), &[r("stuff")]);
        assert_eq!(wlist.one_cutoff_index, 0);
        assert_eq!(wlist.one_cutoff_weight, 0);
        assert_eq!(wlist.total_weight, 1);

        wlist.learn_one(&s("stuff")).unwrap();
        assert_eq!(wlist.rules.as_slice(), &[rw("stuff", 2)]);
        assert_eq!(wlist.one_cutoff_index, 1);
        assert_eq!(wlist.one_cutoff_weight, 2);
        assert_eq!(wlist.total_weight, 2);

        wlist.learn_one(&s("things")).unwrap();
        assert_eq!(wlist.rules.as_slice(), &[rw("stuff", 2), r("things")]);
        assert_eq!(wlist.one_cutoff_index, 1);
        assert_eq!(wlist.one_cutoff_weight, 2);
        assert_eq!(wlist.total_weight, 3);

        wlist.learn_one(&s("items")).unwrap();
        assert_eq!(wlist.rules.as_slice(), &[rw("stuff", 2), r("things"), r("items")]);
        assert_eq!(wlist.one_cutoff_index, 1);
        assert_eq!(wlist.one_cutoff_weight, 2);
        assert_eq!(wlist.total_weight, 4);

        wlist.learn_one(&s("artifacts")).unwrap();
        assert_eq!(wlist.rules.as_slice(), &[rw("stuff", 2), r("things"), r("items"), r("artifacts")]);
        assert_eq!(wlist.one_cutoff_index, 1);
        assert_eq!(wlist.one_cutoff_weight, 2);
        assert_eq!(wlist.total_weight, 5);

        wlist.learn_one(&sw("objects", 13)).unwrap();
        assert_eq!(wlist.rules.as_slice(), &[rw("stuff", 2), rw("objects", 13), r("items"), r("artifacts"), r("things")]);
        assert_eq!(wlist.one_cutoff_index, 2);
        assert_eq!(wlist.one_cutoff_weight, 15);
        assert_eq!(wlist.total_weight, 18);

        wlist.learn_one(&sw("artifacts", 6)).unwrap();
        assert_eq!(wlist.rules.as_slice(), &[rw("stuff", 2), rw("objects", 13), rw("artifacts", 7), r("items"), r("things")]);
        assert_eq!(wlist.one_cutoff_index, 3);
        assert_eq!(wlist.one_cutoff_weight, 22);
        assert_eq!(wlist.total_weight, 24);

        wlist.learn_one(&sw("objects",2)).unwrap();
        assert_eq!(wlist.rules.as_slice(), &[rw("stuff", 2), rw("objects", 15), rw("artifacts", 7), r("items"), r("things")]);
        assert_eq!(wlist.one_cutoff_index, 3);
        assert_eq!(wlist.one_cutoff_weight, 24);
        assert_eq!(wlist.total_weight, 26);
    }

    fn s(s: &str) -> Sample {
        Sample::Word(s.to_owned())
    }

    fn sw(s: &str, w: u32) -> Sample {
        Sample::WordWeighted(s.to_owned(), w)
    }

    fn r(s: &str) -> Rule {
        Rule{name: s.to_owned(), weight: 1}
    }

    fn rw(s: &str, w: u32) -> Rule {
        Rule{name: s.to_owned(), weight: w}
    }
}