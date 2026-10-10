# AI Assistant parity

Clean-room equivalent of the observable features of IntelliJ AI Assistant and Android
Studio Gemini (issue #52). Rows use the same statuses as `platform.md`.

The AI Chat tool window, inline completion, AI actions and commit-message generation are
tracked on their IntelliJ rows: IJ-TW-045, IJ-ED-156 and IJ-ULT-052. Agent mode and the
terminal assistant have no IntelliJ row and stay here.

| ID | Reference | Feature | Status | Commands | Tests | Limitations / evidence |
|---|---|---|---|---|---|---|
| AI-005 | IntelliJ AI Assistant | Agent mode with approvals | Partial | `ai.agent.start`, `ai.agent.approve`, `ai.agent.reject`, `ai.agent.stop`, `ai.agent.clear`, `ai.agent.step` | strict tool-schema and loop-bookkeeping tests (one result message per turn, `max_tokens`/invalid-JSON/turn-cap handling), command tests for auto reads, approved/undoable edits, rejection feedback, file creation, approved commands with exit code and output, Stop; demo-agent end-to-end test; Xvfb mock-provider walkthrough | Agent tab in AI Chat with a visible plan and step list; list/read/search run automatically, every edit, new file and command waits for Approve/Reject with a diff or command preview; edits go through editor documents (one undo step each) and are saved; 30-turn cap; `evidence/ai-assistant.md` |
| AI-006 | IntelliJ AI Assistant | Terminal natural language to command | Partial | `ai.terminal.suggest`, `ai.terminal.run`, `ai.terminal.insert`, `ai.terminal.dismiss` | strict single-command parsing, risk-warning and request tests, real-PTY test that nothing runs before confirmation, Xvfb mock-provider walkthrough | sparkle toggle in the Terminal header opens a fixed-height bar; the suggestion (with a warning for deleting, privileged, history-rewriting, download-and-run or disk commands) runs only on Run / ⌘⏎, Insert types it without Enter; OS, shell, working directory and (if sharing is allowed) the last 30 screen lines are sent; `evidence/ai-assistant.md` |

Statuses are intentionally conservative; see `evidence/ai-assistant.md`.
