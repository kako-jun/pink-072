# PINK-072

A Rust crate for concealing files and folders as pink-colored images (.pnk).

The "072" refers to the 72-byte (8 bytes × 9) seed used for cover image generation.

## How It Works

**Note**: This is NOT steganography.

1. A 72×72 pink cover image (PNG) is generated using Perlin noise
2. Your data is appended after the PNG's end marker (IEND)
3. Image viewers display only the pink image; the hidden data is ignored

Output files use the `.pnk` extension (PNG-compatible but indicates custom format).

## Installation

```toml
[dependencies]
pink072 = "0.1"
```

## Usage

### Encode a File

```rust
use pink072::encode_file;
use std::path::Path;

let seed: [u8; 9] = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11];
encode_file(
    Path::new("secret.jpg"),
    Path::new("output.pnk"),
    &seed,
    8,  // strength (0-12)
)?;
```

### Decode a File

```rust
use pink072::decode_file;
use std::path::Path;

let file_name = decode_file(
    Path::new("output.pnk"),
    Path::new("./extracted/"),
)?;
// Original file restored to ./extracted/secret.jpg
```

### Encode a Folder (ZIP)

```rust
use pink072::encode_folder;
use std::path::Path;

encode_folder(
    Path::new("secret_folder/"),
    Path::new("output.pnk"),
    &seed,
    8,
)?;
```

### Low-level API

```rust
use pink072::{pink072_wrap, pink072_unwrap, encode_pnk, decode_pnk};

// Wrap raw data
let frame = pink072_wrap(b"secret", 0, &seed, 8)?;

// Convert to PNK format
let pnk = encode_pnk(&frame);

// Decode
let frame = decode_pnk(&pnk)?;
let (payload_type, data) = pink072_unwrap(frame)?;
```

## Payload Types

| Type | Description |
|------|-------------|
| 0 | Raw data (no filename) |
| 1 | Single file (filename + data) |
| 2 | ZIP archive (preserves folder structure) |

## Cover Image

- Size: 72×72 pixels (RGBA)
- Style: Organic curves with pink-to-white gradient
- Generated from seed using 2D Perlin noise
- Same seed always produces the same image

## License

GPL-3.0-or-later
