use wasm_bindgen::prelude::*;

use crate::core::{pink072_unwrap, pink072_wrap, pink072_wrap_into};

#[wasm_bindgen]
pub struct UnwrapResult {
    payload_type: u8,
    payload: Vec<u8>,
}

#[wasm_bindgen]
impl UnwrapResult {
    #[wasm_bindgen(getter)]
    pub fn payload_type(&self) -> u8 {
        self.payload_type
    }

    #[wasm_bindgen(getter)]
    pub fn payload(&self) -> Vec<u8> {
        self.payload.clone()
    }
}

#[wasm_bindgen]
pub fn wasm_pink072_wrap(
    payload: &[u8],
    payload_type: u8,
    seed9: &[u8],
) -> Result<Vec<u8>, JsValue> {
    pink072_wrap(payload, payload_type, seed9).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn wasm_pink072_wrap_into(
    payload: &[u8],
    payload_type: u8,
    seed9: &[u8],
    out_frame: &mut [u8],
) -> Result<usize, JsValue> {
    pink072_wrap_into(payload, payload_type, seed9, out_frame)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn wasm_pink072_unwrap(frame: &[u8]) -> Result<UnwrapResult, JsValue> {
    let (payload_type, payload) =
        pink072_unwrap(frame).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(UnwrapResult {
        payload_type,
        payload,
    })
}
