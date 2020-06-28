use crate::{WorkingSet, NamePart, LearnError, SampleSet};
use rand::{SeedableRng, thread_rng};
use rand::rngs::SmallRng;
use rand::Rng;
use rand::prelude::ThreadRng;

#[derive(Clone, std::fmt::Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
enum FormatPart {
    Text(String),
    Part(usize),
    Format(usize),
    Random(Vec<FormatPart>),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct NameFormat {
    name: String,
    parts: Vec<FormatPart>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Name {
    parts: Vec<NamePart>,
    formats: Vec<NameFormat>,
}

impl Name {
    pub fn add_part(&mut self, part: NamePart) {
        self.parts.push(part);
    }

    pub fn learn(&mut self, part_name: &str, sample_set: &SampleSet) -> Result<(), LearnError> {
        for part in self.parts.iter_mut() {
            if part.name() == part_name {
                return part.learn(sample_set);
            }
        }

        Err(
            LearnError::new(
                100,
                format!("Part {} not found", part_name),
                None
            )
        )
    }

    pub fn add_format(&mut self, name: &str, str: &str) {
        let mut parts: Vec<FormatPart> = Vec::with_capacity(8);
        let mut subparts: Vec<FormatPart> = Vec::with_capacity(8);

        let mut pos = 0;
        while pos < str.len() {
            let remainder = &str[pos..];
            let (bs, start, end) = next_bracket(remainder);

            if start > 0 {
                parts.push(FormatPart::Text(remainder[..start].to_owned()));
            }

            subparts.clear();

            let tokens: Vec<&str> = bs.split("|").collect();
            for token in tokens {
                if token.starts_with('=') {
                    subparts.push(FormatPart::Text(token[1..].to_owned()))
                } else if token.starts_with(':') {
                    let format_name = &token[1..];
                    for (i, format) in self.formats.iter().enumerate() {
                        if format.name == format_name {
                            subparts.push(FormatPart::Format(i));
                        }
                    }
                } else {
                    let path_name = token;
                    for (i, part) in self.parts.iter().enumerate() {
                        if part.name() == path_name {
                            subparts.push(FormatPart::Part(i));
                        }
                    }
                }
            }

            if subparts.len() > 1 {
                parts.push(FormatPart::Random(subparts.clone()));
            } else if subparts.len() == 1 {
                parts.push(subparts[0].clone())
            }

            pos += end + 1;
        }

        self.formats.push(NameFormat{
            name: name.to_owned(),
            parts,
        })
    }

    /// Generate names with a fast RNG (SmallRng). This uses `thread_rng()` to
    /// seed, and may return none.
    pub fn generate(&self, format_name: &str) -> Option<GeneratorIter<SmallRng>> {
        if let Ok(rng) = SmallRng::from_rng(thread_rng()) {
            self.generate_with_rng(rng, format_name)
        } else {
            None
        }
    }

    /// Generate names with a fast RNG (SmallRng) using a seed. This is useful if your rand
    /// version differs and you want it to be dependent on external reproducable random data
    /// (e.g. if namegen is part of a bigger procedural generation pipeline).
    pub fn generate_seeded(&self, seed: u64, format_name: &str) -> Option<GeneratorIter<SmallRng>> {
        self.generate_with_rng(SmallRng::seed_from_u64(seed), format_name)
    }

    /// If you for some reason need a secure random generator....
    pub fn generate_with_thread_rng(&self, format_name: &str) -> Option<GeneratorIter<ThreadRng>> {
        self.generate_with_rng(thread_rng(), format_name)
    }

    fn generate_with_rng<T>(&self, rng: T, format_name: &str) -> Option<GeneratorIter<T>> where T: Rng {
        for (i, format) in self.formats.iter().enumerate() {
            if format.name == format_name {
                return Some(
                    GeneratorIter {
                        name: self,
                        format_index: i,
                        ws: WorkingSet::new(),

                        rng,
                    }
                );
            }
        }

        None
    }

    fn run_generate(&self, ws: &mut WorkingSet, rng: &mut impl Rng, format_index: usize) {
        for mut fp in self.formats[format_index].parts.iter() {
            if let FormatPart::Random(list) = fp {
                fp = &list[rng.gen_range(0, list.len())];
            }

            match fp {
                FormatPart::Text(text) => {
                    ws.result_total.push_str(&text);
                }
                FormatPart::Part(part_index) => {
                    self.parts[*part_index].generate(ws, rng);
                    ws.result_total.push_str(&ws.result_str);
                }
                FormatPart::Format(format_index) => {
                    self.run_generate(ws, rng, *format_index);
                }
                _ => {}
            }
        }
    }

    pub fn new() -> Name {
        Name{
            parts: Vec::new(),
            formats: Vec::new(),
        }
    }
}

pub struct GeneratorIter<'a, T> where T: Rng {
    name: &'a Name,
    rng: T,
    format_index: usize,
    ws: WorkingSet,
}

impl<'a, T> Iterator for GeneratorIter<'a, T> where T: Rng {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.ws.result_total.clear();
        self.name.run_generate(&mut self.ws, &mut self.rng, self.format_index);

        Some(self.ws.result_total.clone())
    }
}

fn next_bracket(s: &str) -> (&str, usize, usize) {
    let mut start = 0usize;
    let mut start_found = false;
    let mut end = 0usize;

    for (i, ch) in s.chars().enumerate() {
        match ch {
            '{' => {
                if !start_found {
                    start = i;
                    start_found = true;
                }
            }
            '}' => {
                if start_found {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }

    if start != end && end > start {
        (&s[start+1..end], start, end)
    } else {
        (&s[0..0], s.len(), s.len())
    }
}