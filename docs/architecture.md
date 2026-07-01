# redmine-cli CLI アーキテクチャ

## 目的

LLM agent が扱いやすいことを第一にした Redmine CLI を作る。
人間が直接使う場合にも便利だが、主な目的は Redmine 操作を予測可能・スクリプト可能・検証可能にすること。

## 設計方針

- agent 向けコマンドは安定した JSON 出力を基本にする。
- 人間向け表示は `--format table` や `--format text` で明示的に選べるようにする。
- 書き込み系コマンドは、Redmine API 的に可能な範囲で `--dry-run` をサポートする。
- コマンドは小さく、組み合わせやすく、Redmine のリソース構造に近づける。
- エラーは構造化し、retry 可能か、HTTP status、Redmine 側のエラー、短い hint を含める。
- 設定は明示的かつ確認しやすくする。
- API key などの秘密情報は環境変数、OS keychain、または config 内の参照として扱い、標準出力には出さない。

## 推奨スタック

初期構成としては次を推奨する。

- 言語: Rust
- CLI framework: `clap`
- HTTP client: `reqwest`
- JSON serialization: `serde`, `serde_json`
- Config file: TOML または JSON
- Error handling: `thiserror`, `anyhow`
- Test: Rust unit test + integration test
- Formatter: `rustfmt`
- Linter: `clippy`

Rust を選ぶ理由は、単一バイナリとして配りやすく、Windows/macOS/Linux で動作を安定させやすく、agent が頻繁に呼び出す CLI として起動速度と堅牢性を確保しやすいから。

## リポジトリ構成案

```text
redmine-cli/
  docs/
    architecture.md
    commands.md
    redmine-api-notes.md
  crates/
    redmine-cli/
      src/
        main.rs
        cli.rs
        output.rs
        error.rs
        commands/
          mod.rs
          config.rs
          issues.rs
          projects.rs
          time_entries.rs
          users.rs
        core/
          mod.rs
          redmine_client.rs
          auth.rs
          pagination.rs
          schemas.rs
          types.rs
        agent/
          mod.rs
          tool_manifest.rs
          safety.rs
          summaries.rs
        config/
          mod.rs
          load_config.rs
          config_schema.rs
      Cargo.toml
  tests/
    integration/
  Cargo.toml
```

## 品質ゲート

Rust 本体は標準 toolchain に寄せ、format、lint、test を CI で必須にする。
具体的な開発コマンドは [README.md](../README.md) に置く。

## CLI の形

resource-oriented なコマンドにする。

```bash
redmine-cli config init
redmine-cli config show --json

redmine-cli projects list --json
redmine-cli projects get PROJECT_ID --json

redmine-cli issues list --project PROJECT_ID --status open --json
redmine-cli issues get ISSUE_ID --json
redmine-cli issues create --project PROJECT_ID --subject "..." --description-file body.md --json
redmine-cli issues update ISSUE_ID --status resolved --notes "..." --json
redmine-cli issues comment ISSUE_ID --notes "..." --json

redmine-cli time list --issue ISSUE_ID --json
redmine-cli time log --issue ISSUE_ID --hours 1.5 --activity "Development" --comments "..." --json
```

agent が使う前提で、各コマンドはできるだけ次の option を共通で持つ。

```bash
--json
--format json|table|text
--profile PROFILE
--dry-run
--confirm
--timeout-ms N
--limit N
```

## Agent 向け出力契約

成功時:

```json
{
  "ok": true,
  "operation": "issues.get",
  "data": {
    "id": 123,
    "subject": "Example"
  },
  "meta": {
    "profile": "default",
    "redmineUrl": "https://redmine.example.com",
    "requestId": "optional",
    "truncated": false
  }
}
```

失敗時:

```json
{
  "ok": false,
  "operation": "issues.update",
  "error": {
    "code": "REDMINE_VALIDATION_ERROR",
    "message": "Redmine rejected the update.",
    "status": 422,
    "retryable": false,
    "details": ["Status is invalid"],
    "hint": "Fetch issue metadata and allowed statuses, then retry with a valid status id."
  }
}
```

この出力契約を固定すると、agent が retry すべきか、ユーザーに確認すべきか、関連データを追加取得すべきか判断しやすくなる。

## 設定

設定ファイルの候補:

```text
~/.config/redmine-cli/config.toml
```

例:

```toml
default_profile = "work"

[profiles.work]
url = "https://redmine.example.com"
api_key_env = "REDMINE_API_KEY"
default_project = "ops"
```

環境変数での上書き:

```bash
REDMINE_URL=https://redmine.example.com
REDMINE_API_KEY=...
REDMINE_PROFILE=work
```

## Core module

### `core/redmine_client.rs`

HTTP 通信を担当する。

- URL 組み立て
- auth header
- timeout
- safe な read 系 request の retry
- Redmine error の正規化
- pagination helper

### `commands/*`

CLI の引数解釈と command-specific な validation を担当する。
各 command は core client を呼び、正規化された result object を返す。

### `output.rs`

出力形式を担当する。
command code は直接 `console.log` せず、この層を通す。

### `agent/*`

agent 向け helper を置く。

- command/tool manifest 生成
- 書き込み系 command の safety classification
- 大きい Redmine payload の短い要約
- noisy な API response を減らす field allowlist

## Safety model

コマンドを次のように分類する。

- read: 安全。確認不要。
- write: 明示的に呼ばれたときだけ実行。`--dry-run` をサポート。
- destructive: `--confirm` 必須。実行前に preview を表示。

例:

```text
read: issues get/list
write: issues create/update/comment, time log
destructive: attachments delete, issue relation delete
```

## MVP scope

Phase 1:

- `config init/show`
- `projects list/get`
- `issues list/get/create/update/comment`
- structured JSON output
- Redmine API key auth
- config、output、error normalization の unit test
- `cargo fmt`, `cargo clippy`, `cargo test` を CI に入れる

Phase 2:

- time entries
- users and memberships
- attachments
- custom field helper
- interactive setup
- Redmine container を使った integration test

Phase 3:

- MCP server wrapper または tool manifest export
- agent 向け planning helper
- status、tracker、priority、custom field などの metadata cache

## 早めに確認したい Redmine API の仕様

- issue create/update の field name と custom field の形。
- status、tracker、priority、activity ID は Redmine instance ごとに異なる。
- pagination response の形と default。
- 対象 Redmine に plugin が入っていて利用可能 field が変わるか。
- 認証方式。API key、basic auth、SSO 制約など。

## 未決定事項

- default output: 常に JSON にするか、人間向けは table にして `--json` で JSON にするか。
- config format: Rust CLI と相性の良い TOML を基本にするか、agent から扱いやすい JSON にするか。
- MCP support を first-class package にするか、CLI の薄い wrapper にするか。
