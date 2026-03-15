# フェーズ1：最小限のオウム返し＆LLM対話（基礎の確立）

## アクター実装と連携

### 1. アクター構造体の定義と `Actor` トレイトの実装

まずは、フェーズ1で登場する3つのアクターの「箱」を作ります。それぞれに状態（State）を持たせ、`actix::Actor` トレイトを実装します。

* **Workflow アクター**: 他のアクターのアドレス（`Addr<Channel>`, `Addr<Provider>`）を状態として保持し、メッセージを投げられるようにします。
* **Provider アクター**: OpenAIのAPIキーや、HTTPクライアント（`reqwest` や `async-openai` など）を状態として保持します。
* **Channel アクター**: Discordのクライアントやトークンを保持します。

### 2. 定義したメッセージの `Handler` トレイトの実装

作成したメッセージ定義ファイル に基づき、各アクターがメッセージを受け取った際の具体的な振る舞い（`Handler`）を実装します。

* **Workflow**: `HandleIncomingMessage` を受け取ったら、`ChatMessage` を組み立てて Provider に `GenerateCompletion` を送信し、その結果を Channel に `SendReply` として送る処理を書きます。
* **Provider**: `GenerateCompletion` を受け取ったら、実際にOpenAI互換APIを叩き、結果を `ProviderResponse` として返す処理を書きます。
* **Channel**: `SendReply` を受け取ったら、DiscordのAPIを叩いて特定のチャンネルにテキストを投稿する処理を書きます。

### 3. API通信のモック化（オウム返しの実現）

最初からDiscordやOpenAIのAPIを繋ぎこむと、エラーの原因が「Actixのメッセージパッシングの問題」なのか「APIの使い方の問題」なのか分からなくなります。
まずは、ProviderとChannelの内部処理を**「受け取った文字をそのまま返す（またはターミナルに標準出力する）だけのモック」**として実装し、アクター間の通信が設計通りに流れるか（オウム返しができるか）をテストするのが安全です。

### 4. `main.rs` での起動処理

最後に、Actixのシステム（`actix::System` または `#[actix::main]`）を立ち上げ、各アクターを `start()` してアドレス（`Addr`）を取得・結合するエントリーポイントを作成します。
