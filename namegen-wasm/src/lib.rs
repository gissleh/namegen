#[macro_use]
extern crate serde;

use namegen::{Name, SampleSet, NamePart, FormattingRule};
use std::error::Error;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
pub struct WasmNameGenerator {
    name: Name,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPartOptions {
    name: String,
    kind: String,
    format_rules: Vec<FormattingRule>,
    #[serde(default)]
    initial_tokens: Vec<String>,
    #[serde(default)]
    initial_subtokens: Vec<String>,
    #[serde(default)]
    rlf: bool,
    #[serde(default)]
    ral: bool,
    #[serde(default)]
    lrs: bool,
    #[serde(default)]
    lrm: bool,
    #[serde(default)]
    lre: bool,
}

#[wasm_bindgen]
impl WasmNameGenerator {
    pub fn generate_one(&self, format_name: &str, seed: Option<u64>) -> Option<String> {
        match seed {
            Some(seed) => self.name.generate_seeded(seed, format_name)?.next(),
            None => self.name.generate(format_name)?.next(),
        }
    }

    pub fn generate_many(&self, format_name: &str, amount: usize, seed: Option<u64>) -> JsValue {
        let gen = match seed {
            Some(seed) => self.name.generate_seeded(seed, format_name),
            None => self.name.generate(format_name),
        };

        match gen {
            Some(gen) => {
                let strings: Vec<String> = gen.take(amount).collect();
                JsValue::from_serde(&strings).unwrap()
            }
            None => JsValue::null(),
        }
    }

    pub fn add_part(&mut self, options: JsValue) -> Result<(), JsValue> {
        let mut options: AddPartOptions = match options.into_serde() {
            Ok(options) => options,
            Err(err) => {
                return Err(JsValue::from(format!("{}", err)));
            }
        };

        match options.kind.as_str() {
            "markov" => {
                self.name.add_part(NamePart::new_markov(
                    &options.name, &options.format_rules,
                    options.initial_tokens.as_slice(),
                    options.lrs, options.lrm, options.lre, options.rlf,
                ));
            },
            "cfgrammar" => {
                self.name.add_part(NamePart::new_cfgrammar(
                    &options.name, &options.format_rules,
                    options.initial_subtokens.as_slice(),
                    options.rlf, options.ral,
                ));
            },
            "wordlist" => {
                self.name.add_part(NamePart::new_wordlist(&options.name, &options.format_rules));
            },
            _ => {
                return Err(JsValue::from("AddPartError: Invalid value for options.kind (outdated wasm?)"));
            }
        }

        Ok(())
    }

    pub fn add_format(&mut self, format_name: &str, format_str: &str) {
        self.name.add_format(format_name, format_str)
    }

    pub fn learn(&mut self, part_name: &str, sample_set: JsValue) -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        let sample_set: SampleSet = match sample_set.into_serde() {
            Ok(ss) => ss,
            Err(e) => { return Err(JsValue::from(format!("{}", e))); },
        };

        self.name.learn(part_name, &sample_set).map_err(|e| JsValue::from(format!("{}", e)))
    }

    pub fn data(&self) -> JsValue {
        JsValue::from_serde(&self.name).unwrap()
    }

    pub fn new() -> WasmNameGenerator {
        WasmNameGenerator { name: Name::new() }
    }

    pub fn load(data: &str) -> Result<WasmNameGenerator, JsValue> {
        let name = serde_json::from_slice(data.as_bytes());
        return name
            .map(|n| WasmNameGenerator { name: n })
            .map_err(|e| JsValue::from(format!("{:?}", e)));
    }
}
