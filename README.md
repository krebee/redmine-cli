# redmine-cli

LLM agent が扱いやすいことを第一にした Redmine CLI です。

目指す利用体験:

```bash
cargo install --path crates/redmine-cli
redmine-cli --version
redmine-cli issues get 123 --json
```

実装本体は Rust の単一バイナリにします。

## 現在の状態

初期設計と開発環境 skeleton の段階です。

今あるもの:

- Rust workspace
- `redmine-cli` CLI skeleton
- formatter / linter / test の基本コマンド

## 必要なもの

- Rust stable

## 開発コマンド

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p redmine-cli -- --version
```

確認用:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p redmine-cli -- --version
```

## リポジトリ構成

```text
redmine-cli/
  crates/redmine-cli/   # Rust CLI 本体
  docs/architecture.md    # 詳細設計
  AGENTS.md               # agent 向け作業ルール
```

## ドキュメント

- 詳細設計: [docs/architecture.md](docs/architecture.md)
- agent 向け作業ルール: [AGENTS.md](AGENTS.md)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
