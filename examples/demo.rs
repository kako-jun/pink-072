use pink072::{pink072_unwrap, pink072_wrap};
use std::fs::File;
use std::io::Write;

fn main() {
    let payload = b"Hello, PINK-072!";
    let seed: [u8; 9] = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11];
    let payload_type = 2;

    println!("=== PINK-072 動作確認 ===\n");
    println!("入力: {:?}", String::from_utf8_lossy(payload));
    println!("シード: {:02X?}", seed);
    println!("タイプ: {}\n", payload_type);

    let frame = pink072_wrap(payload, payload_type, &seed).unwrap();
    println!("フレーム生成: {} bytes", frame.len());

    // Cover部分を抽出 (Header 32B の後、72x72x4 bytes)
    let cover = &frame[32..32 + 72 * 72 * 4];

    // PNG出力
    write_png("/tmp/pink_cover.png", 72, 72, cover).unwrap();
    println!("カバー画像を保存: /tmp/pink_cover.png");

    let (t, p) = pink072_unwrap(&frame).unwrap();
    println!(
        "\n抽出: タイプ={}, ペイロード={:?}",
        t,
        String::from_utf8_lossy(&p)
    );

    if payload == p.as_slice() && payload_type == t {
        println!("\n✅ 成功!");
    }
}

fn write_png(path: &str, width: u32, height: u32, rgba: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])?;

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    write_chunk(&mut file, b"IHDR", &ihdr)?;

    let mut raw = Vec::new();
    for y in 0..height as usize {
        raw.push(0);
        let s = y * width as usize * 4;
        raw.extend_from_slice(&rgba[s..s + width as usize * 4]);
    }

    let mut zlib = vec![0x78, 0x01];
    let mut pos = 0;
    while pos < raw.len() {
        let sz = (raw.len() - pos).min(65535);
        let last = pos + sz >= raw.len();
        zlib.push(if last { 1 } else { 0 });
        zlib.extend_from_slice(&(sz as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(sz as u16)).to_le_bytes());
        zlib.extend_from_slice(&raw[pos..pos + sz]);
        pos += sz;
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &raw {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    zlib.extend_from_slice(&((b << 16) | a).to_be_bytes());
    write_chunk(&mut file, b"IDAT", &zlib)?;
    write_chunk(&mut file, b"IEND", &[])?;
    Ok(())
}

fn write_chunk(f: &mut File, t: &[u8; 4], d: &[u8]) -> std::io::Result<()> {
    f.write_all(&(d.len() as u32).to_be_bytes())?;
    f.write_all(t)?;
    f.write_all(d)?;
    let c = [t.to_vec(), d.to_vec()].concat();
    let mut crc = 0xFFFFFFFFu32;
    for &b in &c {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB88320
            } else {
                crc >> 1
            };
        }
    }
    f.write_all(&(!crc).to_be_bytes())?;
    Ok(())
}
