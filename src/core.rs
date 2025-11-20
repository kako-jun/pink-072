use crate::constants::{BLOCK_SIZE, COVER_LEN, HEADER_LEN};
use crate::cover::{generate_cover, validate_seed};
use crate::error::PinkError;

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
