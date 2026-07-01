# redmine-cli

LLM agent が扱いやすいことを第一にした Redmine CLI です。

Rust の単一バイナリとして実装します。

## 必要なもの

- Rust stable

## 使い方

```bash
cargo run -p redmine-cli -- --version
cargo run -p redmine-cli -- config init --url https://redmine.example.com --dry-run
cargo run -p redmine-cli -- projects list --json
cargo run -p redmine-cli -- issues get 123 --json
```

Redmine API key は設定された環境変数から読みます。
既定では `REDMINE_API_KEY` です。

## 主なコマンド

```bash
redmine-cli config init --url URL [--api-key-env NAME] [--profile NAME] [--default-project ID] [--dry-run]
redmine-cli config show

redmine-cli projects list [--limit N]
redmine-cli projects get PROJECT_ID

redmine-cli issues list [--project PROJECT_ID] [--status STATUS_ID] [--limit N]
redmine-cli issues get ISSUE_ID
redmine-cli issues create --project PROJECT_ID --subject SUBJECT [--description TEXT | --description-file PATH] [--dry-run]
redmine-cli issues update ISSUE_ID [--subject TEXT] [--description TEXT] [--status-id ID] [--priority-id ID] [--assigned-to-id ID] [--notes TEXT] [--dry-run]
redmine-cli issues comment ISSUE_ID --notes TEXT [--dry-run]
```

共通 option:

```bash
--json
--format json|text|table
--profile PROFILE
--timeout-ms N
```

## 開発

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

確認用:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## CI / Release

CI は `master` branch への push と手動実行で動きます。
Pull request 前提の運用にはしていません。

`v*.*.*` 形式の tag を push すると、Linux、Windows、macOS 向けの release binary と SHA256 checksum を GitHub Release に添付します。

```bash
git tag v0.1.0
git push origin v0.1.0
```

## ドキュメント

- 詳細設計: [docs/architecture.md](docs/architecture.md)
- agent 向け作業ルール: [AGENTS.md](AGENTS.md)

## License

Licensed under the MIT License ([LICENSE-MIT](LICENSE-MIT)).
