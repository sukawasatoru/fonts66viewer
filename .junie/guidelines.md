# Instructions for AI Agents

This guideline provides AI agents working on this codebase.

このコードベースは Fonts66 の GUI ビューアー アプリケーションです。

## Do and Don'ts

- Do: commit message は英語で書く
- Do: コードを変更したらコードフォーマットと lint と unit test と E2E test を実行する
- Do: テストにおける Fake 実装の方針は FontListDataSource のように trait と `#[cfg(test)]` による切り替えパターンを使用する
- Do: iced 0.14 は macOS のシステム絵文字フォント (Apple Color Emoji) を読み込まないため、絵文字の代わりに SVG アイコンを使用する

## Project Structure and Module Organization

- assets/ アプリケーションにバンドルするファイル
- src/ アプリケーションのソースコード
- snapshot/ snapshot テストで使用する hash ファイル
- tests/\*.ice E2E テストで使用する .ice ファイル

## Build, Test, and Development Commands

- build: `cargo build`
- unit test + End-to-End test + snapshot test: `cargo test -- --include-ignored`
- snapshot の更新: `snapshots/` ディレクトリ内の該当する `.sha256` ファイルを削除してから `cargo test -- --ignored`
  を実行すると、新しいハッシュファイルが自動生成される
- lint: `cargo clippy`
- format: `cargo fmt`

## Architecture Patterns

### Feature 間通信

- `XMessage` を使って app レイヤーから各 feature (view) にメッセージを配信する
- app の `broadcast_xmessage` メソッドで全 view に XMessage を clone して配信する
- feature 内部のコマンド（例: `SettingsViewCommand`）は app の `update` で `map` を使い `AppCommand` に変換する
- 各 feature の `SendXMessage` バリアントは iced の `map` パターンを利用して app レイヤーに XMessage
  を伝搬するための仕組み。  
  `update` 内で `SendXMessage` を受け取り `Task::done(SendXMessage(data))` で再発行すると、app 側の `map` クロージャが
  `AppCommand::XMessage` に変換する。自己再送に見えるが、`map` により必ず `AppCommand::XMessage` に変換されるため無限ループにはならない
