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
- tests/ E2E テストで使用する .ice ファイル

## Build, Test, and Development Commands

- build: cargo build
- unit test: cargo test
- E2E/snapshot test: cargo test -- --ignored
- lint: cargo clippy
- format: cargo fmt
