# AI Assistant parity

Clean-room equivalent of the observable features of IntelliJ AI Assistant and Android
Studio Gemini (issue #52). Rows use the same statuses as `platform.md`.

| ID | Reference | Feature | Status | Commands | Tests | Limitations / evidence |
|---|---|---|---|---|---|---|
| AI-001 | IntelliJ AI Assistant | AI providers and AI Chat tool window | Partial | `ai.configure`, `ai.disable`, `ai.chat.send`, `ai.chat.stop`, `ai.chat.new`, `ai.chat.copyCode`, `ai.chat.insertCode`, `ai.chat.previewApply`, `ai.chat.applyConfirm`, `ai.chat.applyCancel`, `ai.chat.setContext`, `toolwindow.activate.aiChat` | canned-stream provider tests (Anthropic SSE incl. tool use/refusal/fallback, OpenAI-compatible, Ollama), cancellation, chat/apply/policy command tests, Xvfb mock-provider walkthrough | off until configured; Anthropic (default `claude-opus-5-5` / `claude-haiku-5-5`), OpenAI-compatible and Ollama; streaming Markdown, file/selection chips, slash commands, per-project history, Copy/Insert/Apply with diff preview, status bar widget; no real-provider recording yet, no layout reference; `evidence/ai-assistant.md` |
| AI-002 | IntelliJ AI Assistant | Inline code completion (ghost text) | Missing | — | — | planned slice 2 |
| AI-003 | IntelliJ AI Assistant | AI actions: Explain, Rewrite, Generate tests/docs, Fix with AI | Missing | — | — | planned slice 3; reference: `Fix with AI` link in Android Studio sync/build output (scene a06) |
| AI-004 | IntelliJ AI Assistant | Commit message generation | Missing | — | — | planned slice 4; reference: AI-generate icon in the Commit window (scene 12) |
| AI-005 | IntelliJ AI Assistant | Agent mode with approvals | Missing | — | — | planned slice 5 |
| AI-006 | IntelliJ AI Assistant | Terminal natural language to command | Missing | — | — | planned slice 6 |

Statuses are intentionally conservative; see `evidence/ai-assistant.md`.
