//! 最小限のPNGエンコーダ/デコーダ（72x72 RGBA専用）

use crate::constants::{COVER_HEIGHT, COVER_LEN, COVER_WIDTH};
use crate::error::PinkError;

const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// CRC32（PNG標準）
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Adler32（zlib用）
fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// PNGチャンクを書き込み
fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let mut crc_data = chunk_type.to_vec();
    crc_data.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}

/// 72x72 RGBAをPNGにエンコード
pub fn encode_png(rgba: &[u8]) -> Vec<u8> {
    debug_assert_eq!(rgba.len(), COVER_LEN);

    let mut out = Vec::new();
    out.extend_from_slice(&PNG_SIGNATURE);

    // IHDR
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(COVER_WIDTH as u32).to_be_bytes());
    ihdr.extend_from_slice(&(COVER_HEIGHT as u32).to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // color type (RGBA)
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace
    write_chunk(&mut out, b"IHDR", &ihdr);

    // IDAT（非圧縮deflate）
    let mut raw = Vec::with_capacity(COVER_HEIGHT * (1 + COVER_WIDTH * 4));
    for y in 0..COVER_HEIGHT {
        raw.push(0); // filter type: none
        let start = y * COVER_WIDTH * 4;
        raw.extend_from_slice(&rgba[start..start + COVER_WIDTH * 4]);
    }

    // zlibラッパー + 非圧縮ブロック
    let mut zlib = vec![0x78, 0x01]; // zlib header (no compression)
    let mut pos = 0;
    while pos < raw.len() {
        let remaining = raw.len() - pos;
        let block_size = remaining.min(65535);
        let is_last = pos + block_size >= raw.len();
        zlib.push(if is_last { 1 } else { 0 });
        zlib.extend_from_slice(&(block_size as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(block_size as u16)).to_le_bytes());
        zlib.extend_from_slice(&raw[pos..pos + block_size]);
        pos += block_size;
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    write_chunk(&mut out, b"IDAT", &zlib);

    // IEND
    write_chunk(&mut out, b"IEND", &[]);

    out
}

/// PNGデータからIEND終端位置を探す（その後にペイロードがある）
pub fn find_png_end(data: &[u8]) -> Result<usize, PinkError> {
    if data.len() < 8 || data[0..8] != PNG_SIGNATURE {
        return Err(PinkError::InvalidFormat);
    }

    let mut pos = 8;
    while pos + 12 <= data.len() {
        let len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let chunk_type = &data[pos + 4..pos + 8];

        // チャンク全体: 4(len) + 4(type) + len(data) + 4(crc)
        let chunk_end = pos + 12 + len;
        if chunk_end > data.len() {
            return Err(PinkError::InvalidFormat);
        }

        if chunk_type == b"IEND" {
            return Ok(chunk_end);
        }

        pos = chunk_end;
    }

    Err(PinkError::InvalidFormat)
}

/// PINK-072フレームをPNK形式にエンコード
/// 出力: [PNG(カバー画像)][PINK-072フレーム全体]
pub fn encode_pnk(frame: &[u8]) -> Vec<u8> {
    let cover = &frame[crate::constants::HEADER_LEN..crate::constants::HEADER_LEN + COVER_LEN];
    let mut out = encode_png(cover);
    out.extend_from_slice(frame);
    out
}

/// PNK形式からPINK-072フレームを抽出
pub fn decode_pnk(data: &[u8]) -> Result<&[u8], PinkError> {
    let png_end = find_png_end(data)?;
    if png_end >= data.len() {
        return Err(PinkError::InvalidFormat);
    }
    Ok(&data[png_end..])
}
