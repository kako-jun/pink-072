mod constants;
mod core;
mod cover;
mod error;
mod rng;

#[cfg(feature = "wasm")]
mod wasm;

pub use constants::*;
pub use core::{pink072_unwrap, pink072_wrap, pink072_wrap_into};
pub use error::PinkError;

#[cfg(feature = "wasm")]
pub use wasm::{wasm_pink072_unwrap, wasm_pink072_wrap, wasm_pink072_wrap_into};

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
