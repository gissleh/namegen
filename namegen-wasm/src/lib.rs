#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde;

use namegen::{Name};
use std::error::Error;

#[derive(Deserialize, Serialize)]
pub struct WasmNameGenerator {
    name: Name
}

impl WasmNameGenerator {
    pub fn greet() -> String {
        String::from("Greetings")
    }
}

#[js_export]
pub fn new_generator() -> WasmNameGenerator {
    WasmNameGenerator{name: Name::new()}
}

#[js_export]
pub fn load_generator(json: &str) -> Option<WasmNameGenerator> {
    match serde_json::from_str(json) {
        Ok(name) => Some(WasmNameGenerator{ name }),
        Err(_) => None,
    }
}

js_serializable!(WasmNameGenerator);

#[js_export]
pub fn hello() -> String {
    String::from("Hello World")
}