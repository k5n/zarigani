# Zarigani

以下の機能を提供するRust製の自律型AIチャットボットです。

- Discord経由でチャット可能 (Channel)
- OpenAI Codex を利用した AI エージェント (Provider)
- SQLite3 を利用した記憶の永続化 (Memory)
- 定期的な内部記憶改善 (Heartbeat)
- Cron を用いた定期実行 (Scheduler)
- インターネットからの情報検索・収集 (WebSearch)
- ブラウザ操作 (Browser)
- 全体の制御 (Workflow)

このプロジェクトではactixを利用したアクターモデルによるプログラミングを採用しています。各機能は独立したアクターとして実装されており、メッセージパッシングによって通信します。
各機能を呼び出すメッセージ型を抽象化することで、将来的に他のChannelやProviderを追加する際の拡張性を確保します。