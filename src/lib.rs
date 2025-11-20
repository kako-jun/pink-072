use std::fmt;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

const HEADER_LEN: usize = 32;
const COVER_WIDTH: usize = 72;
const COVER_HEIGHT: usize = 72;
const COVER_PIXELS: usize = COVER_WIDTH * COVER_HEIGHT;
const COVER_LEN: usize = COVER_PIXELS * 4;
const BLOCK_SIZE: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinkError {
    SeedLength,
    BufferTooSmall,
    FrameTooSmall,
    PayloadLengthOverflow,
    TruncatedFrame,
}

impl fmt::Display for PinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            PinkError::SeedLength => "seed must be exactly 9 bytes",
            PinkError::BufferTooSmall => "buffer too small",
            PinkError::FrameTooSmall => "frame too small",
            PinkError::PayloadLengthOverflow => "payload length overflow",
            PinkError::TruncatedFrame => "truncated frame",
        };
        f.write_str(msg)
    }
}

#[inline]
fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn seed9_to_u64(seed9: &[u8]) -> u64 {
    let mut first = [0u8; 8];
    first.copy_from_slice(&seed9[..8]);
    let mut state = u64::from_le_bytes(first);
    state ^= (seed9[8] as u64) << 11;
    state
}

fn generate_cover(buf: &mut [u8], seed9: &[u8], strength: u8) {
    let mut state = seed9_to_u64(seed9);

    for pixel in 0..COVER_PIXELS {
        let base = pixel * 4;
        buf[base] = (xorshift64(&mut state) & 0xFF) as u8;
        buf[base + 1] = (xorshift64(&mut state) & 0xFF) as u8;
        buf[base + 2] = (xorshift64(&mut state) & 0xFF) as u8;
        buf[base + 3] = 0xFF;
    }

    let blocks_w = (COVER_WIDTH + BLOCK_SIZE - 1) / BLOCK_SIZE;
    let blocks_h = (COVER_HEIGHT + BLOCK_SIZE - 1) / BLOCK_SIZE;
    let total_blocks = blocks_w * blocks_h;

    let mut indices: Vec<usize> = (0..total_blocks).collect();
    for i in (1..total_blocks).rev() {
        let j = (xorshift64(&mut state) as usize) % (i + 1);
        indices.swap(i, j);
    }

    let mut tmp = vec![0u8; buf.len()];
    for by in 0..blocks_h {
        for bx in 0..blocks_w {
            let src = by * blocks_w + bx;
            let dst = indices[src];
            let src_x = bx * BLOCK_SIZE;
            let src_y = by * BLOCK_SIZE;
            let dst_x = (dst % blocks_w) * BLOCK_SIZE;
            let dst_y = (dst / blocks_w) * BLOCK_SIZE;
            let width = BLOCK_SIZE.min(COVER_WIDTH - src_x);
            let height = BLOCK_SIZE.min(COVER_HEIGHT - src_y);
            for row in 0..height {
                let src_row = ((src_y + row) * COVER_WIDTH + src_x) * 4;
                let dst_row = ((dst_y + row) * COVER_WIDTH + dst_x) * 4;
                let len = width * 4;
                tmp[dst_row..dst_row + len].copy_from_slice(&buf[src_row..src_row + len]);
            }
        }
    }
    buf.copy_from_slice(&tmp);

    let max_r = strength.min(12);
    let max_g = (strength / 2).min(6);
    let max_b = (strength.saturating_mul(5) / 6).min(10);

    for pixel in 0..COVER_PIXELS {
        let base = pixel * 4;
        let dr = (xorshift64(&mut state) & 0xFF) as u8 % (max_r + 1);
        let dg = (xorshift64(&mut state) & 0xFF) as u8 % (max_g + 1);
        let db = (xorshift64(&mut state) & 0xFF) as u8 % (max_b + 1);

        let r = (buf[base] as i16 + dr as i16).clamp(0, 255);
        let g = (buf[base + 1] as i16 - dg as i16).clamp(0, 255);
        let b = (buf[base + 2] as i16 - db as i16).clamp(0, 255);

        buf[base] = r as u8;
        buf[base + 1] = g as u8;
        buf[base + 2] = b as u8;
    }
}

fn validate_seed(seed9: &[u8]) -> Result<(), PinkError> {
    if seed9.len() != 9 {
        Err(PinkError::SeedLength)
    } else {
        Ok(())
    }
}

pub fn pink072_wrap(
    payload: &[u8],
    payload_type: u8,
    seed9: &[u8],
    strength: u8,
) -> Result<Vec<u8>, PinkError> {
    let total_len = HEADER_LEN + COVER_LEN + payload.len();
    let mut frame = vec![0u8; total_len];
    pink072_wrap_into(payload, payload_type, seed9, strength, &mut frame)?;
    Ok(frame)
}

pub fn pink072_wrap_into(
    payload: &[u8],
    payload_type: u8,
    seed9: &[u8],
    strength: u8,
    out_frame: &mut [u8],
) -> Result<usize, PinkError> {
    validate_seed(seed9)?;
    let clamped_strength = strength.min(12);
    let total_len = HEADER_LEN + COVER_LEN + payload.len();
    if out_frame.len() < total_len {
        return Err(PinkError::BufferTooSmall);
    }

    out_frame[0] = 1;
    out_frame[1] = payload_type;
    out_frame[2] = BLOCK_SIZE as u8;
    out_frame[3] = clamped_strength;

    let payload_len = payload.len() as u64;
    out_frame[4..12].copy_from_slice(&payload_len.to_le_bytes());
    out_frame[12..HEADER_LEN].fill(0);

    let cover_range = HEADER_LEN..HEADER_LEN + COVER_LEN;
    generate_cover(&mut out_frame[cover_range.clone()], seed9, clamped_strength);

    let payload_range = cover_range.end..cover_range.end + payload.len();
    out_frame[payload_range].copy_from_slice(payload);

    Ok(total_len)
}

pub fn pink072_unwrap(frame: &[u8]) -> Result<(u8, Vec<u8>), PinkError> {
    if frame.len() < HEADER_LEN + COVER_LEN {
        return Err(PinkError::FrameTooSmall);
    }

    let payload_type = frame[1];
    let mut len_bytes = [0u8; 8];
    len_bytes.copy_from_slice(&frame[4..12]);
    let payload_len = usize::try_from(u64::from_le_bytes(len_bytes))
        .map_err(|_| PinkError::PayloadLengthOverflow)?;

    let payload_start = HEADER_LEN + COVER_LEN;
    let payload_end = payload_start
        .checked_add(payload_len)
        .ok_or(PinkError::PayloadLengthOverflow)?;

    if frame.len() < payload_end {
        return Err(PinkError::TruncatedFrame);
    }

    Ok((payload_type, frame[payload_start..payload_end].to_vec()))
}

#[cfg(feature = "wasm")]
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

#[cfg(feature = "wasm")]
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

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_pink072_unwrap(frame: &[u8]) -> Result<(u8, Vec<u8>), JsValue> {
    pink072_unwrap(frame).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed() -> [u8; 9] {
        [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11]
    }

    #[test]
    fn round_trip_preserves_payload_and_type() {
        let payload = b"hello pink072";
        let frame = pink072_wrap(payload, 2, &seed(), 8).expect("wrap");
        let (ptype, out) = pink072_unwrap(&frame).expect("unwrap");
        assert_eq!(ptype, 2);
        assert_eq!(out, payload);
    }

    #[test]
    fn header_and_cover_are_correctly_sized() {
        let payload = vec![0xAA; 10];
        let total = HEADER_LEN + COVER_LEN + payload.len();
        let mut buf = vec![0u8; total];
        let written = pink072_wrap_into(&payload, 4, &seed(), 12, &mut buf).expect("wrap_into");
        assert_eq!(written, total);
        assert_eq!(buf[0], 1);
        assert_eq!(buf[1], 4);
        assert_eq!(buf[2], BLOCK_SIZE as u8);
        assert_eq!(buf[3], 12);
        assert_eq!(buf.len(), HEADER_LEN + COVER_LEN + payload.len());
    }

    #[test]
    fn strength_is_clamped_to_12() {
        let payload = [0u8; 1];
        let frame = pink072_wrap(&payload, 0, &seed(), 200).expect("wrap");
        assert_eq!(frame[3], 12);
    }

    #[test]
    fn invalid_seed_errors() {
        let payload = [0u8; 1];
        let mut out = vec![0u8; HEADER_LEN + COVER_LEN + payload.len()];
        let err =
            pink072_wrap_into(&payload, 0, &[0xAA], 1, &mut out).expect_err("expected seed error");
        assert_eq!(err, PinkError::SeedLength);
    }
}
