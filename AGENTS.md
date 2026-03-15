# Zarigani

以下の機能を提供するRust製の自律型AIチャットボットです。

- Discord などを経由してチャット可能 (Channel)
- OpenAI 互換 API を利用した自然言語処理 (Provider)
- OpenAI Codex などを利用した AI エージェント実行 (Agent)
- SQLite3 を利用した記憶の永続化 (Memory)
    - RAG を利用した情報検索
        - キーワードベース
        - 特徴量ベクトルによる類似度検索
- 定期的な内部記憶改善 (Heartbeat)
- Cron を用いた定期実行 (Scheduler)
- インターネットからの情報検索・収集 (WebSearch)
- ブラウザ操作 (Browser)
- 全体の制御 (Workflow)

Provider はユーザーとの間の対話を処理し、AI エージェントに処理させる必要があるタスクは Agent を利用して実行します。
Provider は OpenAI 互換 API を利用しますが、実際には Ollama や llama.cpp などで動かすローカルモデルや、OpenRouter を通じて、無料もしくは低価格のモデルを利用する想定です。

このプロジェクトでは actix を利用したアクターモデルによるプログラミングを採用しています。各機能は独立したアクターとして実装されており、メッセージパッシングによって通信します。
各機能を呼び出すメッセージ型を抽象化することで、将来的に他の Channel や Provider を追加する際の拡張性を確保します。