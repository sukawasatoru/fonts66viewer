# Instructions for AI Agents

This guideline provides AI agents working on this codebase.

このコードベースは Fonts66 の GUI ビューアー アプリケーションです。

## Do and Don'ts

- Do: commit message は英語で書く
- Do: コードを変更したらコードフォーマットと lint と unit test と E2E test を実行する
- Do: テストにおける Fake 実装の方針は FontListDataSource のように trait と `#[cfg(test)]` による切り替えパターンを使用する

## Project Structure and Module Organization

- assets/ アプリケーションにバンドルするファイル
- src/ アプリケーションのソースコード
- snapshot/ snapshot テストで使用する hash ファイル
- tests/\*.ice E2E テストで使用する .ice ファイル

## Build, Test, and Development Commands

- build: `cargo build`
- unit test: `cargo test`
- E2E/snapshot test: `cargo test -- --ignored`
- snapshot の更新: `snapshots/` ディレクトリ内の該当する `.sha256` ファイルを削除してから `cargo test -- --ignored`
  を実行すると、新しいハッシュファイルが自動生成される
- lint: `cargo clippy`
- format: `cargo fmt`

## Architecture Patterns

### Feature 間通信

- `XMessage` を使って app レイヤーから各 feature (view) にメッセージを配信する
- app の `broadcast_xmessage` メソッドで全 view に XMessage を clone して配信する
- feature 内部のコマンド（例: `SettingsViewCommand`）は app の `update` で `map` を使い `AppCommand` に変換する
