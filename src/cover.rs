use crate::constants::{COVER_HEIGHT, COVER_WIDTH};
use crate::error::PinkError;
use crate::rng::{generate_permutation, perlin2d, seed9_to_u64, xorshift64};

/// ピンク系のカラーパレット（ピンク〜白）
const PINK_PALETTE: [(u8, u8, u8); 5] = [
    (255, 182, 193), // light pink (ベース)
    (255, 200, 210), // lighter pink
    (255, 218, 225), // very light pink
    (255, 235, 240), // almost white pink
    (255, 250, 252), // near white
];

pub fn generate_cover(buf: &mut [u8], seed9: &[u8], strength: u8) {
    let mut state = seed9_to_u64(seed9);
    let perm = generate_permutation(&mut state);

    // strengthに応じたノイズのスケール（大きいほど細かい模様）
    let base_scale = 0.03 + (strength as f32) * 0.005;

    // オフセット（シードごとに異なる位置から開始）
    let offset_x = (xorshift64(&mut state) % 1000) as f32;
    let offset_y = (xorshift64(&mut state) % 1000) as f32;

    // 2つの周波数を重ねて有機的な曲線を作る
    for y in 0..COVER_HEIGHT {
        for x in 0..COVER_WIDTH {
            let pixel = y * COVER_WIDTH + x;
            let base = pixel * 4;

            let fx = x as f32 + offset_x;
            let fy = y as f32 + offset_y;

            // 低周波（大きな曲線）+ 高周波（細部）
            let n1 = perlin2d(fx * base_scale, fy * base_scale, &perm);
            let n2 = perlin2d(fx * base_scale * 2.5, fy * base_scale * 2.5, &perm);
            let noise = n1 * 0.7 + n2 * 0.3;

            // ノイズ値をパレットインデックスに変換
            let idx = ((noise * 4.99) as usize).min(4);
            let (r, g, b) = PINK_PALETTE[idx];

            // strengthに応じて微細なバリエーションを追加
            let var = ((strength as i16) * 2).min(20);
            let vr = ((xorshift64(&mut state) % (var as u64 * 2 + 1)) as i16 - var) as i16;
            let vg = ((xorshift64(&mut state) % (var as u64 * 2 + 1)) as i16 - var) as i16;
            let vb = ((xorshift64(&mut state) % (var as u64 * 2 + 1)) as i16 - var) as i16;

            buf[base] = (r as i16 + vr).clamp(0, 255) as u8;
            buf[base + 1] = (g as i16 + vg).clamp(0, 255) as u8;
            buf[base + 2] = (b as i16 + vb).clamp(0, 255) as u8;
            buf[base + 3] = 0xFF;
        }
    }
}

pub fn validate_seed(seed9: &[u8]) -> Result<(), PinkError> {
    if seed9.len() != 9 {
        Err(PinkError::SeedLength)
    } else {
        Ok(())
    }
}
