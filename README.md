# pink-072

PINK-072 仕様を忠実に実装した Rust クレートです。Rust からは次の 3 つの関数を利用できます。

- `pink072_wrap(payload, payload_type, seed9, strength) -> Vec<u8>`: 任意長のペイロードを受け取り、Header(32B) + Cover(72×72 RGBA) + Payload で構成されるフレームを新規バッファに生成します。
- `pink072_wrap_into(payload, payload_type, seed9, strength, out_frame) -> usize`: 事前確保した `out_frame` にフレームを書き込みたい場合はこちらを使用します。返り値は書き込まれたバイト長です。
- `pink072_unwrap(frame) -> (payload_type, payload)`：フレームからヘッダー情報を読み出し、埋め込まれているペイロードを抽出します。

`wasm` フィーチャを有効化すると、`wasm-bindgen` で公開される `wasm_pink072_wrap` / `wasm_pink072_wrap_into` / `wasm_pink072_unwrap` を利用できます。Rust 側の API と同じ振る舞いですが、エラーは `JsValue` として返します。`wasm_pink072_unwrap` は `UnwrapResult` オブジェクト（`payload_type` と `payload` プロパティを持つ）を返します。

主な仕様ポイント:

1. フレームは `Header (32B) | Cover (72×72 RGBA = 20736B) | Payload` の並びで固定されます。
2. Header には Version(=1)、Payload Type(0〜4)、Block Size(=16)、Color Strength(0〜12)、Payload Length(u64, LE)、Reserved(20B, 0埋め) を格納します。
3. Cover は 9 バイト鍵から xorshift64 で生成した乱数ノイズを敷き詰め、16×16 ブロック単位で Fisher–Yates シャッフルし、最後に淡いピンクのバイアス（R増/G減/B減）を掛けます。
4. ペイロードは加工せずにコピーされ、`pink072_unwrap` は Header と Cover をスキップして生データを返します。

`wasm-pack build --features wasm` で WebAssembly を生成すれば、JavaScript から上記 API を直接呼び出せます。

## 開発フロー

- Git hooks は `cargo-husky` によって Rust 側だけで管理します。`cargo install cargo-husky` をインストール後、`cargo husky install` を一度実行すると pre-commit フックが登録され、コミット前に `cargo fmt -- --check` と `cargo clippy -- -D warnings` が走ります。
- GitHub Actions (`.github/workflows/release.yml`) はリリース公開時に Rust のフォーマット／Lint／テストを実行し、`wasm-pack build --release --target web --features wasm` で生成した `pkg/` を `pink072-wasm.tar.gz` にまとめてリリースページへ添付します。
