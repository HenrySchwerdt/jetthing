# AI Assistant: providers and AI Chat

Issue #52, slice 1. Clean-room equivalent of the observable AI Chat of IntelliJ AI
Assistant / Android Studio Gemini (right-stripe sparkle tool window, scene `04` right
stripe "Agent/Gemini" icon in `reference/android-studio/NOTES.md`). No reference
screenshot of the chat window itself exists yet, so layout parity is **not** claimed;
the behaviour below is what is implemented and tested.

```sh
cargo test -p jetthing-ai          # SSE parsing, providers, service, chat, markdown, diff
cargo test -p jetthing-core ai_    # commands: configure, chat, stop, errors, apply, policy
JETTHING_AI_MOCK=1 cargo run -p jetthing-app -- <project>   # offline demo provider
```

## Behaviour

- **Off until configured.** `AiSettings::enabled` defaults to `false`; a key in the
  environment never enables AI. `ai.configure` validates provider, model and key
  *before* persisting, so a missing key leaves AI off (`01-ai-off-setup.png`).
- **Providers** (`crates/jetthing-ai`): Anthropic Messages API over SSE (default
  `claude-opus-5-5` for chat, `claude-haiku-5-5` for inline completion, both
  configurable; automatic prompt caching; `output_config.effort`; client tools with
  `eager_input_streaming` and strict JSON validation of streamed inputs; refusals
  surfaced as errors with partial output discarded; server-side refusal fallbacks
  (`fallbacks: "default"`) on supported models against the first-party endpoint,
  with fallback-aware history echo), OpenAI-compatible Chat Completions (SSE), and
  Ollama `/api/chat` (NDJSON). All share one `Provider` trait and are tested with
  canned streams; no test performs network I/O.
- **Keys**: stored key in `<config>/secrets.toml` (mode `0600`) wins over
  `ANTHROPIC_API_KEY` / `OPENAI_API_KEY`. `<config>` is `JETTHING_CONFIG_DIR` or the
  platform config directory. OS keychain storage is not implemented yet.
- **Project data sharing**: `.jetthing/ai.toml` (`allow_ai`, `share_file_contents`);
  an unreadable policy fails closed.
- **AI Chat tool window** (right stripe, `toolwindow.activate.aiChat`): streaming
  Markdown (headings, lists, inline code, fenced code with editor syntax colors),
  "Thinking…/Generating…" with Stop (`02-chat-streaming.png`), context chips for the
  current file or selection, slash commands `/explain /tests /docs /fix /refactor`
  with suggestions, per-project history in `.jetthing/ai-chat.json`
  (`03-chat-answer.png`).
- **Code blocks**: Copy, Insert at caret, and Apply… which previews a line diff
  against the attached selection/file (or current selection) and applies it as one
  undoable edit; a preview made stale by later edits is refused
  (`04-apply-diff-preview.png`, `05-applied-in-editor.png`).
- **Status bar widget**: provider/model, "generating…", "off" or "disabled";
  click opens AI Chat.

## Evidence

Linux, Xvfb 1600×1000, `JETTHING_RENDERER=glow`, 2026-10-10. Answers come from the
offline demo provider (`JETTHING_AI_MOCK=1`, labelled "mock-chat (demo)"); no
`ANTHROPIC_API_KEY` was available, so no real-provider walkthrough was recorded.

| Screenshot | Shows |
|---|---|
| `ai/01-ai-off-setup.png` | AI off: setup form, actionable missing-key error |
| `ai/02-chat-streaming.png` | streaming answer with Stop, selection chip `main.rs:3-9` |
| `ai/03-chat-answer.png` | completed Markdown answer, highlighted code block actions |
| `ai/04-apply-diff-preview.png` | Apply… diff preview (+1 −0) |
| `ai/05-applied-in-editor.png` | applied edit in the editor |

## Gaps

Inline ghost-text completion, AI actions (Explain/Rewrite/Generate/Fix with AI),
commit message generation, terminal NL→command, agent mode, `@file`/`@symbol`,
open-tabs/VCS/build-output context and OS keychain storage are follow-up slices.
