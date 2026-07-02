# redmine-cli アーキテクチャ

## 目的

Redmine 操作を LLM agent から予測可能に呼び出せる CLI にします。
人間が直接使う場合もありますが、安定した JSON 出力と安全な書き込み操作を優先します。

## 前提

- Rust の単一バイナリとして配布します。
- Redmine API key は環境変数から読み、標準出力には出しません。
- Redmine instance ごとに異なる ID や custom field は固定値として決め打ちしません。
- コマンド本体は直接表示を組み立てず、output layer を通します。

## 構成

```text
crates/redmine-cli/src/
  main.rs
  cli.rs
  commands/
  config.rs
  error.rs
  output.rs
  redmine_client.rs
```

- `cli.rs`: clap による引数定義。
- `commands/`: command ごとの入力検証と実行。
- `config.rs`: TOML config の読み書きと profile 選択。
- `redmine_client.rs`: Redmine HTTP API 呼び出し。
- `output.rs`: agent 向けの結果 JSON を生成。
- `error.rs`: エラーコード、retry 可否、hint の正規化。

## 出力契約

成功時:

```json
{
  "ok": true,
  "operation": "issues.get",
  "data": {},
  "meta": {
    "profile": "default",
    "redmineUrl": "https://redmine.example.com",
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
    "message": "Redmine rejected the request.",
    "status": 422,
    "retryable": false,
    "details": ["Status is invalid"],
    "hint": "Fetch Redmine metadata and retry with instance-specific IDs."
  },
  "meta": {
    "profile": null,
    "redmineUrl": null,
    "truncated": false
  }
}
```

`--format text` と `--format table` は現在も JSON と同じ形を出力します。
人間向け表示を追加する場合も、この JSON 契約は壊しません。

## Safety model

- read: Redmine から取得するだけの command。
- write: `issues create/update/comment` など。可能な範囲で `--dry-run` を用意します。
- destructive: 削除など。追加する場合は `--confirm` を必須にします。
- update: GitHub Release から現在 target 用の binary と SHA256 checksum を取得し、checksum 検証後に実行ファイルを差し替えます。`--dry-run` で対象 release と asset を確認でき、対話端末では yes/no 確認後に実更新します。CI や agent などの非対話環境では `--confirm` を必須にします。

## CI / Release

CI は `master` branch への push と手動実行で、次を確認します。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Release は `v*.*.*` tag の push で実行します。
Linux、Windows、macOS 向けに release binary を作り、checksum と合わせて GitHub Release に添付します。
`update` はこの release binary と `.sha256` asset を利用します。
