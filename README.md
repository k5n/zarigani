# Zarigani

Zarigani は、Rust と Actix のアクターモデルで構築する自律型 AI チャットボットです。Channel、Provider、Agent、Memory、Workflow などの機能を独立したアクターとして分離し、メッセージパッシングで連携する構成を前提にしています。

現時点の実装では、CLI から入力したメッセージを Workflow が受け取り、OpenAI 互換 API を話す Provider に転送し、その応答を CLI に返す最小構成が動作します。

## 特徴

- Rust + Actix によるアクターモデル実装
- OpenAI 互換 API を利用する Provider
- CLI Channel による対話インターフェース
- TOML 設定ファイルによる起動時設定
- 将来の Discord、Memory、RAG、Scheduler などを見据えた責務分離

## 現在の実装範囲

現在の Zarigani で利用できる主な機能は次の通りです。

- `CLI Channel`
  - 標準入力からメッセージを受け取り、標準出力へ応答を返します
- `Workflow`
  - 入力メッセージを受け取り、Provider に推論を依頼し、応答を Channel へ返します
- `OpenAI Compatible Provider`
  - `rig-core` を利用して OpenAI 互換の Chat Completions API に接続します
- `Config Loader`
  - `~/.zarigani/config.toml` を読み込み、起動時に設定を検証します

未実装または今後拡張予定のもの:

- Discord などの追加 Channel
- 長期記憶の永続化
- RAG
- Heartbeat
- Scheduler
- Browser / WebSearch
- 自律的な Agent 実行フロー

## アーキテクチャ概要

現在の最小構成は次の流れで動作します。

1. `main.rs` が設定ファイルを読み込みます
2. OpenAI 互換 Provider を初期化します
3. CLI Channel、ChannelDispatcher、Workflow を起動します
4. CLI から入力されたメッセージを Workflow が受け取ります
5. Workflow が Provider に応答生成を依頼します
6. Provider の応答を ChannelDispatcher 経由で CLI Channel に返します

役割の分離は大まかに次の通りです。

- `Channel`
  - 外部入出力を扱います
- `Workflow`
  - 各アクター間の制御を担います
- `Provider`
  - LLM への問い合わせを担当します
- `Config`
  - 設定ファイルの探索、読み込み、検証を担います

## 必要環境

- Rust
- Cargo
- OpenAI 互換 API サーバー

OpenAI 互換 API サーバーとしては、たとえば次のようなものを想定しています。

- `llama.cpp` の OpenAI 互換サーバー
- Ollama を OpenAI 互換エンドポイント経由で利用する構成
- OpenRouter などの OpenAI 互換サービス

## 起動方法

まず設定ファイルを作成し、`~/.zarigani/config.toml` に設定を書きます。

```bash
mkdir -p ~/.zarigani
```

```toml
[provider.openai_compatible]
base_url = "http://127.0.0.1:8080/v1"
model = "local-model"
api_key = "dummy"
system_prompt = "あなたはZariganiという名前のAIアシスタントです。"
temperature = 0.7
max_tokens = 1024
```

その後、プロジェクトルートで起動します。

```bash
cargo run
```

起動後は CLI で対話できます。

```text
user> こんにちは

zarigani> こんにちは。何を手伝いましょうか。
```

終了するには `/exit` を入力するか、`Ctrl-D` を送ってください。

## 設定ファイル仕様

Zarigani は起動時に一度だけ設定ファイルを読み込みます。設定ファイルの既定パスは次の通りです。

```text
~/.zarigani/config.toml
```

設定ファイルが存在しない、読めない、TOML として不正、または値の検証に失敗した場合、アプリケーションは起動に失敗します。設定不備時に別の Provider へフォールバックする挙動はありません。

### 設定項目

現時点では `provider.openai_compatible` セクションのみを使用します。

#### `provider.openai_compatible.base_url`

- 型: `string`
- 必須: はい
- 用途: OpenAI 互換 API のベース URL
- 例: `http://127.0.0.1:8080/v1`

空文字は許可されません。

#### `provider.openai_compatible.api_key`

- 型: `string`
- 必須: いいえ
- 用途: OpenAI 互換 API の認証キー

ローカルサーバーなどで API キーが不要な場合は省略できます。省略時は内部で `"dummy"` が補われます。

#### `provider.openai_compatible.model`

- 型: `string`
- 必須: はい
- 用途: 利用するモデル名
- 例: `gpt-4o-mini`, `local-model`, `qwen2.5`

空文字は許可されません。

#### `provider.openai_compatible.system_prompt`

- 型: `string`
- 必須: いいえ
- 用途: Provider の既定システムプロンプト

Provider へのリクエスト時に明示的な `system_prompt` が渡されない場合、この値が使われます。

#### `provider.openai_compatible.temperature`

- 型: `float`
- 必須: いいえ
- 用途: 生成時の温度パラメータ

許容範囲は `0.0..=2.0` です。`NaN` や無限大は許可されません。

#### `provider.openai_compatible.max_tokens`

- 型: `integer`
- 必須: いいえ
- 用途: 生成時の最大トークン数

`0` は許可されません。

### バリデーション仕様

起動時に次の検証が行われます。

- `base_url` が空文字ではないこと
- `model` が空文字ではないこと
- `temperature` が `0.0..=2.0` の範囲内であること
- `max_tokens` が `1` 以上であること

### 最小設定例

```toml
[provider.openai_compatible]
base_url = "http://127.0.0.1:8080/v1"
model = "local-model"
```

## ログ

ログは `tracing` と `tracing-subscriber` を使って出力します。既定では `zarigani=debug,actix=info` が使われ、`RUST_LOG` を設定すると上書きできます。

例:

```bash
RUST_LOG=zarigani=info,actix=info cargo run
```

## 主要ファイル

- [src/main.rs](/home/nakatani/Projects/k5n/zarigani/src/main.rs)
  - 起動処理、設定ロード、各アクター起動
- [src/config/types.rs](/home/nakatani/Projects/k5n/zarigani/src/config/types.rs)
  - アプリケーション設定構造体と検証
- [src/config/loader.rs](/home/nakatani/Projects/k5n/zarigani/src/config/loader.rs)
  - 設定ファイル読み込み
- [src/config/path.rs](/home/nakatani/Projects/k5n/zarigani/src/config/path.rs)
  - 既定設定パスの解決
- [src/providers/openai.rs](/home/nakatani/Projects/k5n/zarigani/src/providers/openai.rs)
  - OpenAI 互換 Provider 実装
- [src/core/workflow.rs](/home/nakatani/Projects/k5n/zarigani/src/core/workflow.rs)
  - Channel と Provider を接続する制御

## 注意点

- 現状の CLI 実装では会話履歴は 1 メッセージ分のみを Provider に渡します
- Workflow 側で固定のシステムプロンプトを指定しているため、現在は設定ファイルの `system_prompt` より Workflow 側の値が優先されます
- OpenAI 互換 API の具体的な互換性は接続先実装に依存します

## ライセンス

Apache-2.0 License
