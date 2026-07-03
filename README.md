# redmine-cli

LLM agent が扱いやすいことを第一にした Redmine CLI です。

Rust の単一バイナリとして実装します。

## 必要なもの

- Rust stable
- Windows で MSVC toolchain を使う場合:
  - Visual Studio 2022 Build Tools
  - `Desktop development with C++`
  - Windows 10/11 SDK

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
redmine-cli config init --url URL [--api-key-env NAME] [--profile NAME] [--default-project ID] [--ssl-no-revoke] [--dry-run]
redmine-cli config show

redmine-cli projects list [--limit N]
redmine-cli projects get PROJECT_ID

redmine-cli issues list [--project PROJECT_ID] [--status STATUS_ID] [--limit N]
redmine-cli issues get ISSUE_ID
redmine-cli issues create --project PROJECT_ID --subject SUBJECT [--description TEXT | --description-file PATH] [--dry-run]
redmine-cli issues update ISSUE_ID [--subject TEXT] [--description TEXT] [--status-id ID] [--priority-id ID] [--assigned-to-id ID] [--notes TEXT] [--dry-run]
redmine-cli issues comment ISSUE_ID --notes TEXT [--dry-run]

redmine-cli update [--repo OWNER/REPO] [--tag TAG] [--force] [--dry-run] [--confirm]
```

共通 option:

```bash
--json
--format json|text|table
--profile PROFILE
--timeout-ms N
--ssl-no-revoke
```

`--ssl-no-revoke` または config の `ssl_no_revoke = true` を指定した場合、Redmine API 通信用の TLS backend を通常時の native-tls から rustls に切り替えます。
Windows 環境で TLS 証明書失効確認が原因の接続失敗を回避するための option です。
TLS 失敗時に HTTPS から HTTP へ自動 fallback することはありません。

## アップデート

GitHub Release から現在の OS / architecture に合う binary を取得して、インストール済みの `redmine-cli` を更新できます。
実行前に `--dry-run` で対象 release と asset を確認できます。

```bash
redmine-cli update --dry-run
redmine-cli update
```

`update` は実行前に yes/no 確認を表示します。
CI や agent などの非対話環境では `--confirm` を指定します。
`Cargo.toml` の `repository` が実際の GitHub repository を指していない場合は、`--repo OWNER/REPO` を指定します。

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

Windows で MSVC toolchain を使う場合は、`pwsh` から Visual Studio 2022 Build Tools の環境を明示して実行できます。
ローカルの Visual Studio installation path は環境ごとに異なるため、repository 側では固定しません。

```powershell
$vs = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'

cmd /d /c "`"$vs`" && cargo check --workspace"
cmd /d /c "`"$vs`" && cargo clippy --workspace --all-targets -- -D warnings"
cmd /d /c "`"$vs`" && cargo test --workspace"
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
