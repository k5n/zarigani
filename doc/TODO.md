# TODO

## フェーズ1：最小限のオウム返し＆LLM対話（基礎の確立）

まずはアクターのライフサイクルとメッセージパッシングに慣れます。

作成するアクター: Channel (Discord), Provider (LLM), Workflow

やること:

1. Discordで発言すると Channel が受け取り、Workflow にメッセージ（例: UserMessage）を送る。
2. Workflow が Provider にメッセージ（例: ChatRequest）を送る。
3. Provider が Codex(OpenAI) を叩き、結果を Workflow に返す。
4. Workflow が Channel に返答を指示し、Discordに投稿される。

## フェーズ2：記憶の導入（ステートの永続化）

AIに文脈を持たせます。

追加するアクター: Memory (SQLite)

やること: Workflow がLLMにリクエストを送る前に Memory から過去の会話履歴を引き出し、LLMからの返答も Memory に保存するようにします。

## フェーズ3：ツールの追加（外部との相互作用）

エージェントとしての能力を拡張します。

追加するアクター: WebSearch, Browser

やること: Provider (LLM) にFunction Calling（またはツール使用のプロンプト）を実装し、LLMが「検索したい」と判断した際に、Workflow がそれを解釈して WebSearch や Browser に処理を委譲するようにします。

## フェーズ4：定期実行（スケジューリング）

トリガーを外部（ユーザー）だけでなく、内部からも生み出します。

追加するアクター: Scheduler (Cron)

やること: Scheduler アクターが定期的に Workflow にメッセージを送り、Workflow が特定のタスク（例: 毎朝のニュースチェック）を実行するようにします。

## フェーズ5：定期的な内部記憶改善（自己学習）

AIが自分の記憶を定期的に見直し、改善する機能を追加します。

追加するアクター: Heartbeat

やること: Heartbeat アクターが定期的に Workflow にメッセージを送り、Workflow が Memory から過去の会話を引き出して分析し、必要に応じて記憶を更新します。
