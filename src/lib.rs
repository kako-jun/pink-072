mod constants;
mod core;
mod cover;
mod error;
mod file;
mod noise;
mod png;

#[cfg(feature = "wasm")]
mod wasm;

pub use constants::*;
pub use core::{pink072_unwrap, pink072_wrap, pink072_wrap_into};
pub use error::PinkError;
pub use file::{
    decode_auto, decode_file, decode_folder, decode_raw, encode_auto, encode_file, encode_folder,
    encode_raw, PAYLOAD_TYPE_FILE, PAYLOAD_TYPE_RAW, PAYLOAD_TYPE_ZIP,
};
pub use png::{decode_pnk, encode_pnk};

#[cfg(feature = "wasm")]
pub use wasm::{wasm_pink072_unwrap, wasm_pink072_wrap, wasm_pink072_wrap_into};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

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

    #[test]
    fn pnk_encode_decode_round_trip() {
        let payload = b"test pnk format";
        let frame = pink072_wrap(payload, 0, &seed(), 8).expect("wrap");
        let pnk = encode_pnk(&frame);

        // PNKはPNGシグネチャで始まる
        assert_eq!(
            &pnk[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );

        // デコード
        let decoded_frame = decode_pnk(&pnk).expect("decode_pnk");
        let (ptype, out) = pink072_unwrap(decoded_frame).expect("unwrap");
        assert_eq!(ptype, 0);
        assert_eq!(out, payload);
    }

    #[test]
    fn file_encode_decode_round_trip() {
        let test_dir = Path::new("/tmp/pink072_test_file");
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        // テストファイル作成
        let input_file = test_dir.join("test_input.txt");
        let pnk_file = test_dir.join("output.pnk");
        let output_dir = test_dir.join("extracted");

        fs::write(&input_file, b"Hello, Pink072!").unwrap();

        // エンコード
        encode_file(&input_file, &pnk_file, &seed(), 8).unwrap();
        assert!(pnk_file.exists());

        // デコード
        let file_name = decode_file(&pnk_file, &output_dir).unwrap();
        assert_eq!(file_name, "test_input.txt");

        // 内容確認
        let restored = fs::read(output_dir.join(&file_name)).unwrap();
        assert_eq!(restored, b"Hello, Pink072!");

        // クリーンアップ
        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn folder_encode_decode_round_trip() {
        let test_dir = Path::new("/tmp/pink072_test_folder");
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        // テストフォルダ作成
        let input_folder = test_dir.join("input_folder");
        let sub_folder = input_folder.join("sub");
        fs::create_dir_all(&sub_folder).unwrap();

        fs::write(input_folder.join("file1.txt"), b"content1").unwrap();
        fs::write(input_folder.join("file2.txt"), b"content2").unwrap();
        fs::write(sub_folder.join("file3.txt"), b"content3").unwrap();

        let pnk_file = test_dir.join("folder.pnk");
        let output_dir = test_dir.join("extracted");

        // エンコード
        encode_folder(&input_folder, &pnk_file, &seed(), 8).unwrap();
        assert!(pnk_file.exists());

        // デコード
        let files = decode_folder(&pnk_file, &output_dir).unwrap();
        assert!(files.len() >= 3);

        // 内容確認
        assert_eq!(fs::read(output_dir.join("file1.txt")).unwrap(), b"content1");
        assert_eq!(fs::read(output_dir.join("file2.txt")).unwrap(), b"content2");
        assert_eq!(
            fs::read(output_dir.join("sub/file3.txt")).unwrap(),
            b"content3"
        );

        // クリーンアップ
        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn auto_encode_decode_file() {
        let test_dir = Path::new("/tmp/pink072_test_auto");
        let _ = fs::remove_dir_all(test_dir);
        fs::create_dir_all(test_dir).unwrap();

        let input_file = test_dir.join("auto_test.txt");
        let pnk_file = test_dir.join("auto.pnk");
        let output_dir = test_dir.join("extracted");

        fs::write(&input_file, b"auto test content").unwrap();

        // 自動エンコード（ファイル）
        encode_auto(&input_file, &pnk_file, &seed(), 8).unwrap();

        // 自動デコード
        let files = decode_auto(&pnk_file, &output_dir).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "auto_test.txt");

        let _ = fs::remove_dir_all(test_dir);
    }
}
