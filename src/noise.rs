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

/// シードからパーリンノイズ用の256要素の順列テーブルを生成
#[inline]
pub fn generate_permutation(state: &mut u64) -> [u8; 256] {
    let mut perm: [u8; 256] = core::array::from_fn(|i| i as u8);
    for i in (1..256).rev() {
        let j = (xorshift64(state) as usize) % (i + 1);
        perm.swap(i, j);
    }
    perm
}

/// パーリンノイズ用の勾配ベクトル（8方向）
const GRADIENTS: [(f32, f32); 8] = {
    const D: f32 = std::f32::consts::FRAC_1_SQRT_2;
    [
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (D, D),
        (-D, D),
        (D, -D),
        (-D, -D),
    ]
};

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

#[inline]
fn grad(hash: u8, x: f32, y: f32) -> f32 {
    let (gx, gy) = GRADIENTS[(hash & 7) as usize];
    gx * x + gy * y
}

/// 2Dパーリンノイズ（0.0〜1.0を返す）
#[inline]
pub fn perlin2d(x: f32, y: f32, perm: &[u8; 256]) -> f32 {
    let xi = (x.floor() as i32) & 255;
    let yi = (y.floor() as i32) & 255;
    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    let aa = perm[(perm[xi as usize] as usize + yi as usize) & 255];
    let ab = perm[(perm[xi as usize] as usize + (yi + 1) as usize) & 255];
    let ba = perm[(perm[((xi + 1) & 255) as usize] as usize + yi as usize) & 255];
    let bb = perm[(perm[((xi + 1) & 255) as usize] as usize + (yi + 1) as usize) & 255];

    let x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1.0, yf), u);
    let x2 = lerp(grad(ab, xf, yf - 1.0), grad(bb, xf - 1.0, yf - 1.0), u);

    (lerp(x1, x2, v) + 1.0) * 0.5
}
