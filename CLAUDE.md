# PINK-072 開発者向けドキュメント

PINK-072仕様を実装したRustクレート。ファイルやフォルダをピンク色の画像（.pnk）に偽装して隠蔽する。

「072」はカバー画像生成に使う72バイト（8バイト×9）のシードに由来。

## 技術的背景

**注意**: これはステガノグラフィ（画像ピクセル内にデータを埋め込む手法）ではない。

仕組み:
1. カバー画像（72×72 PNG）を生成
2. PNG終端（IEND）の後ろにPINK-072フレームを連結
3. 画像ビューアはPNG部分のみ表示し、後続データを無視

出力ファイルの拡張子は `.pnk`（PNG互換だが独自形式であることを明示）。

## プロジェクト構造

```
src/
├── lib.rs        # エントリーポイント、公開API
├── core.rs       # wrap/unwrapのコア実装
├── cover.rs      # カバー画像生成（パーリンノイズ）
├── noise.rs      # xorshift64乱数生成器、2Dパーリンノイズ
├── png.rs        # PNG/PNKエンコード・デコード
├── file.rs       # ファイル/フォルダ操作
├── constants.rs  # 定数定義
├── error.rs      # エラー型
└── wasm.rs       # WASM用ラッパー（feature="wasm"）
```

## PNKファイル構造

```
[PNG画像 (カバー72×72)][PINK-072フレーム]
```

### PINK-072フレーム構造

```
[Header 32B][Cover 20,736B][Payload 可変長]
```

### Header (32バイト)

| オフセット | サイズ | 内容 |
|-----------|-------|------|
| 0 | 1 | Version (=1) |
| 1 | 1 | Payload Type |
| 2 | 1 | Block Size (=16) |
| 3 | 1 | Reserved (=0) |
| 4 | 8 | Payload Length (u64 LE) |
| 12 | 20 | Reserved (0埋め) |

### Payload Type

| Type | 内容 | 用途 |
|------|------|------|
| 0 | 生データ | ファイル名なしのバイト列 |
| 1 | 単一ファイル | ファイル名 + データ |
| 2 | ZIP | フォルダ構造を保持 |
| 3-4 | 予約 | 将来の拡張用 |

### 単一ファイル形式 (Type=1)

```
[ファイル名長 2B (LE)][ファイル名 UTF-8][データ]
```

### ZIP形式 (Type=2)

非圧縮ZIPアーカイブ（Stored）をペイロードとして格納。フォルダ構造、ファイル名を保持。圧縮しないため高速。

## Cover画像生成

1. 9バイトシードからxorshift64で順列テーブル生成
2. 2Dパーリンノイズで滑らかな曲線模様を生成
3. ピンク(255,170,185)〜薄ピンク(255,235,240)のグラデーション
4. カバー画像はペイロードと無関係（シードのみで決定）

## 主要定数

```rust
HEADER_LEN: usize = 32
COVER_LEN: usize = 72 * 72 * 4  // 20,736
COVER_WIDTH: usize = 72
COVER_HEIGHT: usize = 72
```

## 依存クレート

- `wasm-bindgen` 0.2 (optional): WASM連携
- `zip` 2: フォルダのZIPアーカイブ（非圧縮）

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
