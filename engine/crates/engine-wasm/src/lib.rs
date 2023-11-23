use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn make_config(schema: &str) -> Vec<u8> {
    // let registry = parse_sdl(schema).unwrap();
    b"meowmeowmeow".to_vec()
}
