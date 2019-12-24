use crate::{WorkingSet, NamePart};
use rand::Rng;

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
                        if part.name == path_name {
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

    pub fn generate<'a, T>(&'a self, rng: &'a mut T, format_name: &str) -> Option<GeneratorIter<'a, T>> where T: Rng {
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
    rng: &'a mut T,
    format_index: usize,
    ws: WorkingSet,
}

impl<'a, T> Iterator for GeneratorIter<'a, T> where T: Rng {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.ws.result_total.clear();
        self.name.run_generate(&mut self.ws, self.rng, self.format_index);

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