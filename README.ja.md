# PINK-072

ファイルやフォルダをピンク色の画像（.pnk）に偽装して隠すRustクレート。

「072」はカバー画像生成に使う72バイト（8バイト×9）のシードに由来。

## 仕組み

**注意**: これはステガノグラフィではありません。

1. パーリンノイズで72×72のピンク色カバー画像（PNG）を生成
2. PNG終端マーカー（IEND）の後ろにデータを連結
3. 画像ビューアはピンク画像のみを表示し、隠しデータは無視される

出力ファイルの拡張子は `.pnk`（PNG互換だが独自形式であることを明示）。

## インストール

```toml
[dependencies]
pink072 = "1.0"
```

## 使い方

### ファイルをエンコード

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

### ファイルをデコード

```rust
use pink072::decode_file;
use std::path::Path;

let file_name = decode_file(
    Path::new("output.pnk"),
    Path::new("./extracted/"),
)?;
// 元のファイルが ./extracted/secret.jpg に復元される
```

### フォルダをエンコード（ZIP）

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

### 低レベルAPI

```rust
use pink072::{pink072_wrap, pink072_unwrap, encode_pnk, decode_pnk};

// 生データをラップ
let frame = pink072_wrap(b"secret", 0, &seed, 8)?;

// PNK形式に変換
let pnk = encode_pnk(&frame);

// デコード
let frame = decode_pnk(&pnk)?;
let (payload_type, data) = pink072_unwrap(frame)?;
```

## ペイロードタイプ

| Type | 内容 |
|------|------|
| 0 | 生データ（ファイル名なし） |
| 1 | 単一ファイル（ファイル名 + データ） |
| 2 | ZIPアーカイブ（フォルダ構造を保持） |

## カバー画像

- サイズ: 72×72ピクセル（RGBA）
- 見た目: ピンク〜白のグラデーションによる有機的な曲線
- 2Dパーリンノイズでシードから生成
- 同じシードなら常に同じ画像

## ライセンス

GPL-3.0-or-later
