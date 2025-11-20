use crate::constants::{BLOCK_SIZE, COVER_HEIGHT, COVER_PIXELS, COVER_WIDTH};
use crate::error::PinkError;
use crate::rng::{seed9_to_u64, xorshift64};

pub fn generate_cover(buf: &mut [u8], seed9: &[u8], strength: u8) {
    let mut state = seed9_to_u64(seed9);

    for pixel in 0..COVER_PIXELS {
        let base = pixel * 4;
        buf[base] = (xorshift64(&mut state) & 0xFF) as u8;
        buf[base + 1] = (xorshift64(&mut state) & 0xFF) as u8;
        buf[base + 2] = (xorshift64(&mut state) & 0xFF) as u8;
        buf[base + 3] = 0xFF;
    }

    let blocks_w = COVER_WIDTH.div_ceil(BLOCK_SIZE);
    let blocks_h = COVER_HEIGHT.div_ceil(BLOCK_SIZE);
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
            let width = BLOCK_SIZE.min(COVER_WIDTH - src_x).min(COVER_WIDTH - dst_x);
            let height = BLOCK_SIZE
                .min(COVER_HEIGHT - src_y)
                .min(COVER_HEIGHT - dst_y);
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

pub fn validate_seed(seed9: &[u8]) -> Result<(), PinkError> {
    if seed9.len() != 9 {
        Err(PinkError::SeedLength)
    } else {
        Ok(())
    }
}
