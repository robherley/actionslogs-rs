use wasm_bindgen::prelude::*;

mod ansi;
mod log;
mod parser;

pub use log::{Command, Group, Line};
pub use parser::{Node, Parser};

#[wasm_bindgen]
pub fn parse(raw: &str) -> String {
    let mut parser = Parser::new();
    parser.add_raw(raw);
    parser.to_json().unwrap()
}
