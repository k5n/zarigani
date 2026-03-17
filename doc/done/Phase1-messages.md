# フェーズ1：最小限のオウム返し＆LLM対話（基礎の確立）

## 基盤となるメッセージ定義

`actix` では、アクター間で送信するデータ構造に `Message` トレイトを実装し、戻り値の型（`rtype`）を定義するのが基本になります。将来の拡張性（別のチャットツールや別のLLMの導入）を見据えて、**「Discord依存」や「OpenAI依存」のデータ構造をアクター間のメッセージに含めない** ことがポイントです。

以下が、基盤となるメッセージ定義のドラフトです。

### 1. 共通のドメインモデル (Shared Domain Models)

まずは、複数のアクターで使い回す「会話」の共通フォーマットを定義します。OpenAIのAPIレスポンスをそのまま使い回すのではなく、Zarigani専用の型に抽象化します。

```rust
use actix::prelude::*;

// AIとの会話における役割
#[derive(Debug, Clone)]
pub enum Role {
    System,
    User,
    Assistant,
    // 将来的に Tool(Function) などを追加可能
}

// 抽象化された1つのメッセージ単位
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

```

---

### 2. Workflow が受け取るメッセージ (Channel -> Workflow)

Discordでユーザーが発言した際、`Channel` アクターがその内容を抽象化して `Workflow` に伝えます。

```rust
// Discordなどの発信元からWorkflowへ送られるイベント
#[derive(Message, Debug)]
#[rtype(result = "Result<(), WorkflowError>")] // 処理の成功/失敗だけを返す
pub struct HandleIncomingMessage {
    pub source_channel_id: String, // どこに返信すべきか（DiscordのチャンネルIDなど）
    pub user_id: String,           // 誰が発言したか
    pub content: String,           // 発言内容
}

#[derive(Debug)]
pub struct WorkflowError(pub String);

```

**設計の意図:** `Workflow` は「どこから来たか (source_channel_id)」だけを文字列で受け取ります。これがDiscordのIDであれ、将来追加されるSlackのIDであれ、Workflowは気にせず処理を続行できます。

---

### 3. Provider へ要求するメッセージ (Workflow -> Provider)

`Workflow` が文脈（履歴）を組み立てて、LLMに推論を依頼します。

```rust
// WorkflowからLLM(Provider)への生成依頼
#[derive(Message, Debug)]
#[rtype(result = "Result<ProviderResponse, ProviderError>")]
pub struct GenerateCompletion {
    pub history: Vec<ChatMessage>, // これまでの文脈
    pub system_prompt: Option<String>, // 「あなたはZariganiです」などの指示
}

// LLMからの返答の抽象化
#[derive(Debug)]
pub struct ProviderResponse {
    pub content: String,
    // フェーズ3でツール(Function Calling)を追加する際、ここに情報を追加します
    // pub tool_calls: Option<Vec<ToolCall>>, 
}

#[derive(Debug)]
pub struct ProviderError(pub String);

```

**設計の意図:** `GenerateCompletion` にはOpenAI特有のパラメータ（`temperature` や `model` など）は含めません。それらの固有設定は `Provider` アクター自身の内部状態として持たせる（初期化時に設定する）ことで、Workflow側をクリーンに保ちます。

---

### 4. Channel へ要求するメッセージ (Workflow -> Channel)

`Provider` から返答を受け取った `Workflow` が、ユーザーに返信を届けるために `Channel` に指示を出します。

```rust
// WorkflowからChannelへの送信指示
#[derive(Message, Debug)]
#[rtype(result = "Result<(), ChannelError>")]
pub struct SendReply {
    pub target_channel_id: String, // 送信先（HandleIncomingMessageで受け取ったID）
    pub content: String,           // 送信するテキスト
}

#[derive(Debug)]
pub struct ChannelError(pub String);

```

**設計の意図:** シンプルに「どこに」「何を」送るかだけを定義しています。画像送信などが必要になった場合は、この構造体に `attachments: Vec<File>` などを追加して拡張できます。

---

### 一連の流れ（Workflow内での処理イメージ）

このメッセージ定義を使うと、Workflowアクターの内部（`Handler<HandleIncomingMessage>` の実装）は以下のようなシンプルな流れになります。

1. `HandleIncomingMessage` を受け取る。
2. 内容を `ChatMessage` に変換し、履歴 (`history`) を組み立てる（※最初は履歴なしでもOK）。
3. `Provider` アクターに対して `GenerateCompletion { history, ... }` を送信（`send().await`）する。
4. `ProviderResponse` を受け取る。
5. `Channel` アクターに対して `SendReply { target_channel_id, content: response.content }` を送信する。
