# pink-072

PINK-072仕様を実装したRustクレート。ペイロードをピンク色のカバー画像に連結して隠蔽します。

## 仕組み

**注意**: これはステガノグラフィ（画像ピクセル内にデータを埋め込む手法）ではありません。カバー画像の後ろにペイロードをそのまま連結する方式です。PNG/WebP等で保存すると、画像ビューアはカバー部分のみを表示し、後続のペイロードは無視されます。

- **カバー画像**: 常に72×72 RGBA固定（シードが同じなら見た目も同一）
- **ペイロード**: テキスト、画像、動画、ZIP等あらゆるバイナリを格納可能
- **見た目**: 2Dパーリンノイズによる有機的な曲線模様（ピンク〜白のグラデーション）

## インストール

```toml
[dependencies]
pink072 = "0.1"
```

## 使い方

### Rust API

```rust
use pink072::{pink072_wrap, pink072_unwrap};

// ペイロードを埋め込み
let payload = b"secret data";
let seed = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11];
let frame = pink072_wrap(payload, 2, &seed, 8)?;

// ペイロードを抽出
let (payload_type, extracted) = pink072_unwrap(&frame)?;
```

### WASM API

```bash
wasm-pack build --features wasm --target web
```

```javascript
import init, { wasm_pink072_wrap, wasm_pink072_unwrap } from './pkg/pink072.js';

await init();
const frame = wasm_pink072_wrap(payload, 2, seed, 8);
const result = wasm_pink072_unwrap(frame);
```

## API

### `pink072_wrap(payload, payload_type, seed9, strength) -> Vec<u8>`

ペイロードを埋め込んだフレームを生成。

- `payload`: 任意長のバイト列
- `payload_type`: 0〜4のペイロード種別
- `seed9`: 9バイトのシード（乱数生成用）
- `strength`: 0〜12のピンク強度

### `pink072_unwrap(frame) -> (payload_type, payload)`

フレームからペイロードを抽出。

## フレーム構造

```
Header (32B) | Cover (72×72 RGBA = 20,736B) | Payload
```

- **Header**: Version, PayloadType, BlockSize, Strength, PayloadLength, Reserved
- **Cover**: シード由来の乱数ノイズ + Fisher-Yatesシャッフル + ピンクバイアス
- **Payload**: 加工なしの生データ

## ライセンス

GPL-3.0-or-later
