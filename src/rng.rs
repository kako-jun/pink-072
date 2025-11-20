#[inline]
pub fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

pub fn seed9_to_u64(seed9: &[u8]) -> u64 {
    let mut first = [0u8; 8];
    first.copy_from_slice(&seed9[..8]);
    let mut state = u64::from_le_bytes(first);
    state ^= (seed9[8] as u64) << 11;
    state
}
