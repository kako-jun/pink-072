# pink-072 開発者向けドキュメント

PINK-072仕様を実装したRustクレート。ペイロードをピンク色のカバー画像に連結して隠蔽する。

## 技術的背景

**注意**: これはステガノグラフィ（画像ピクセル内にデータを埋め込む手法）ではない。カバー画像の後ろにペイロードをそのまま連結する方式。PNG/WebP等の画像フォーマットで保存すると、画像ビューアはカバー部分のみを表示し、後続のペイロードは無視される。

- カバー画像: 常に72×72 RGBA固定（シードが同じなら見た目も同一）
- ペイロード: テキスト、画像、動画、ZIP等あらゆるバイナリを格納可能
- ファイルサイズ: `32B (Header) + 20,736B (Cover) + ペイロード長`

## プロジェクト構造

```
src/
├── lib.rs        # エントリーポイント、公開API
├── core.rs       # wrap/unwrapのコア実装
├── cover.rs      # カバー画像生成
├── rng.rs        # xorshift64乱数生成器、パーリンノイズ
├── constants.rs  # 定数定義
├── error.rs      # エラー型
└── wasm.rs       # WASM用ラッパー（feature="wasm"）
```

## PINK-072仕様

### フレーム構造

```
[Header 32B][Cover 20,736B][Payload 可変長]
```

### Header (32バイト)

| オフセット | サイズ | 内容 |
|-----------|-------|------|
| 0 | 1 | Version (=1) |
| 1 | 1 | Payload Type (0-4) |
| 2 | 1 | Block Size (=16) |
| 3 | 1 | Color Strength (0-12) |
| 4 | 8 | Payload Length (u64 LE) |
| 12 | 20 | Reserved (0埋め) |

### Cover (72×72 RGBA = 20,736バイト)

1. 9バイトシードからxorshift64で順列テーブル生成
2. **2Dパーリンノイズ**で有機的な曲線模様を生成（低周波 + 高周波の重ね合わせ）
3. ピンク〜白のグラデーションパレットでマッピング
4. strengthに応じた微細なバリエーション追加

カバー画像はペイロードとは無関係に、シードのみから決定論的に生成される。

### 主要定数

```rust
HEADER_LEN: usize = 32
COVER_LEN: usize = 72 * 72 * 4  // 20,736
BLOCK_SIZE: usize = 16
MAX_STRENGTH: u8 = 12
```

## 依存クレート

- `wasm-bindgen` 0.2 (optional): WASM連携

## ビルド

```bash
cargo build              # ライブラリビルド
cargo test               # テスト実行
cargo clippy             # Lint

# WASM
wasm-pack build --features wasm --target web
```

## CI/CD

- **pre-commit**: cargo-husky（fmt, clippy）
- **release.yml**: WASMビルドをGitHub Releaseに添付
