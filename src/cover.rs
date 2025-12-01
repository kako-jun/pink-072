use crate::constants::{COVER_HEIGHT, COVER_WIDTH};
use crate::error::PinkError;
use crate::noise::{generate_permutation, perlin2d, seed9_to_u64, xorshift64};

pub fn generate_cover(buf: &mut [u8], seed9: &[u8]) {
    let mut state = seed9_to_u64(seed9);
    let perm = generate_permutation(&mut state);

    // 大きめの模様が2〜3個見える程度のスケール
    let base_scale = 0.04;

    // オフセット（シードごとに異なる位置から開始）
    let offset_x = (xorshift64(&mut state) % 1000) as f32;
    let offset_y = (xorshift64(&mut state) % 1000) as f32;

    for y in 0..COVER_HEIGHT {
        for x in 0..COVER_WIDTH {
            let pixel = y * COVER_WIDTH + x;
            let base = pixel * 4;

            let fx = x as f32 + offset_x;
            let fy = y as f32 + offset_y;

            // 2層のノイズで滑らかな丸い光のような模様
            let n1 = perlin2d(fx * base_scale, fy * base_scale, &perm);
            let n2 = perlin2d(fx * base_scale * 0.6, fy * base_scale * 0.6, &perm);
            let noise = n1 * 0.5 + n2 * 0.5;

            // 滑らかな補間でピンク〜白のグラデーション
            let t = noise.clamp(0.0, 1.0);
            let t_smooth = t * t * (3.0 - 2.0 * t); // smoothstep

            // ピンク (255, 170, 185) → 薄ピンク (255, 235, 240)
            let r = 255;
            let g = (170.0 + (235.0 - 170.0) * t_smooth) as u8;
            let b = (185.0 + (240.0 - 185.0) * t_smooth) as u8;

            buf[base] = r;
            buf[base + 1] = g;
            buf[base + 2] = b;
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
