use wasm_bindgen::prelude::*;

use crate::core::{pink072_unwrap, pink072_wrap, pink072_wrap_into};

#[wasm_bindgen]
pub fn wasm_pink072_wrap(
    payload: &[u8],
    payload_type: u8,
    seed9: &[u8],
    strength: u8,
) -> Result<Vec<u8>, JsValue> {
    pink072_wrap(payload, payload_type, seed9, strength)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn wasm_pink072_wrap_into(
    payload: &[u8],
    payload_type: u8,
    seed9: &[u8],
    strength: u8,
    out_frame: &mut [u8],
) -> Result<usize, JsValue> {
    pink072_wrap_into(payload, payload_type, seed9, strength, out_frame)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn wasm_pink072_unwrap(frame: &[u8]) -> Result<(u8, Vec<u8>), JsValue> {
    pink072_unwrap(frame).map_err(|e| JsValue::from_str(&e.to_string()))
}
