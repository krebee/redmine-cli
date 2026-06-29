# redmine-cli

LLM agent が扱いやすいことを第一にした Redmine CLI です。

目指すインストール体験:

```bash
npm install -g redmine-cli
redmine-cli --version
redmine-cli issues get 123 --json
```

実装本体は Rust の単一バイナリにし、npm package はそのバイナリを呼び出す入口として使います。

## 現在の状態

初期設計と開発環境 skeleton の段階です。

今あるもの:

- Rust workspace
- `redmine-cli` CLI skeleton
- npm wrapper skeleton
- platform package skeleton
- formatter / linter / test の基本コマンド

## 必要なもの

- Rust stable
- Node.js
- npm

Windows PowerShell で `npm` が実行ポリシーに止められる場合は、代わりに `npm.cmd` を使ってください。

```powershell
npm.cmd --version
```

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
npm.cmd run check:npm
node npm/redmine-cli/bin/redmine-cli.js --version
node npm/redmine-cli/bin/redmine-cli.js issues get 123 --json
```

## リポジトリ構成

```text
redmine-cli/
  crates/redmine-cli/   # Rust CLI 本体
  npm/redmine-cli/      # npm wrapper
  npm/redmine-cli-*/    # platform-specific package skeleton
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
