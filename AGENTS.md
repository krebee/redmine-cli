# AGENTS.md

このリポジトリを触る agent 向けの作業ルールです。

## 先に読むもの

- 入口と開発コマンド: [README.md](README.md)
- 詳細設計: [docs/architecture.md](docs/architecture.md)

大きな設計変更をする場合は、実装だけでなく設計ドキュメントも更新してください。

## 守ること

- 実装本体は Rust の単一バイナリという前提を維持する。
- agent 向けの安定した JSON 出力を壊さない。
- write 系 command は可能な範囲で `--dry-run` を用意する。
- destructive な command は `--confirm` を必須にする。
- Redmine instance ごとに異なる ID や custom field を固定値として決め打ちしない。
- 秘密情報を標準出力に出さない。
- command code で直接表示を組み立てず、output layer を通す。
- コミットメッセージは Conventional Commits に準拠し、説明文は日本語で書く。

## 変更時の確認

通常の変更では次を確認してください。

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

未関係な refactor は避けてください。
