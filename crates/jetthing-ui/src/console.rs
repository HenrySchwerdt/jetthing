//! Console model for the Build/Run tool window.
//!
//! Process output arrives in arbitrary chunks. This turns it into display
//! lines the way a terminal would show them: `\r` returns to the start of the
//! line (progress counters overwrite themselves), ANSI escape sequences are
//! removed, and lines that end up empty only because their content was
//! progress output erased by escapes or `\r` are dropped. Real blank lines in
//! program output are kept.

use jetthing_runner::{OutputChunk, OutputStream};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleLine {
    pub text: String,
    pub stream: OutputStream,
}

#[derive(Default)]
struct LineBuilder {
    text: String,
    /// Whether the line contained `\r` or escape sequences (progress output).
    rewritten: bool,
    stderr: bool,
}

impl LineBuilder {
    fn finish(&mut self, lines: &mut Vec<ConsoleLine>) {
        let line = std::mem::take(self);
        if line.text.is_empty() && line.rewritten {
            return;
        }
        lines.push(ConsoleLine {
            text: line.text,
            stream: if line.stderr {
                OutputStream::Stderr
            } else {
                OutputStream::Stdout
            },
        });
    }
}

/// Converts raw output chunks into console lines.
pub fn console_lines(chunks: &[OutputChunk]) -> Vec<ConsoleLine> {
    let mut lines = Vec::new();
    let mut line = LineBuilder::default();
    let mut escape = Escape::None;
    for chunk in chunks {
        let stderr = matches!(chunk.stream, OutputStream::Stderr);
        let mut chars = chunk.text.chars().peekable();
        while let Some(ch) = chars.next() {
            match escape {
                Escape::None => {}
                Escape::Start => {
                    escape = match ch {
                        '[' => Escape::Csi,
                        ']' => Escape::Osc,
                        _ => Escape::None,
                    };
                    continue;
                }
                Escape::Csi => {
                    // Parameters and intermediates until a final byte @..~.
                    if ('@'..='~').contains(&ch) {
                        escape = Escape::None;
                    }
                    continue;
                }
                Escape::Osc => {
                    if ch == '\u{7}' {
                        escape = Escape::None;
                    } else if ch == '\u{1b}' {
                        escape = Escape::OscEsc;
                    }
                    continue;
                }
                Escape::OscEsc => {
                    escape = Escape::None;
                    continue;
                }
            }
            match ch {
                '\u{1b}' => {
                    escape = Escape::Start;
                    line.rewritten = true;
                }
                '\n' => {
                    line.stderr |= stderr;
                    line.finish(&mut lines);
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        continue;
                    }
                    line.text.clear();
                    line.rewritten = true;
                }
                '\t' => line.text.push_str("    "),
                ch if ch.is_control() => {}
                ch => {
                    line.text.push(ch);
                    line.stderr |= stderr;
                }
            }
        }
    }
    if !line.text.is_empty() {
        line.finish(&mut lines);
    }
    lines
}

#[derive(Clone, Copy)]
enum Escape {
    None,
    Start,
    Csi,
    Osc,
    OscEsc,
}

/// Height for the console viewport that holds whole rows only, so a view
/// stuck to the bottom never shows a half-cut first line under the header.
pub fn whole_rows_height(available: f32, row_height: f32) -> f32 {
    if row_height <= 0.0 {
        return available.max(0.0);
    }
    (available / row_height).floor().max(1.0) * row_height
}

/// Caches the converted lines; output only grows while a task runs.
#[derive(Default)]
pub struct ConsoleView {
    key: (usize, usize),
    lines: Vec<ConsoleLine>,
}

impl ConsoleView {
    pub fn lines(&mut self, chunks: &[OutputChunk]) -> &[ConsoleLine] {
        let key = (chunks.len(), chunks.last().map_or(0, |c| c.text.len()));
        if key != self.key {
            self.lines = console_lines(chunks);
            self.key = key;
        }
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn out(text: &str) -> OutputChunk {
        OutputChunk {
            stream: OutputStream::Stdout,
            text: text.into(),
        }
    }

    fn texts(chunks: &[OutputChunk]) -> Vec<String> {
        console_lines(chunks).into_iter().map(|l| l.text).collect()
    }

    #[test]
    fn carriage_returns_overwrite_progress() {
        assert_eq!(
            texts(&[out("progress 10%\rprogress 50%\rprogress 100%\ndone\n")]),
            ["progress 100%", "done"]
        );
        assert_eq!(texts(&[out("windows\r\nline\r\n")]), ["windows", "line"]);
    }

    #[test]
    fn lines_split_across_chunks_are_joined() {
        assert_eq!(
            texts(&[out("hel"), out("lo\nwor"), out("ld")]),
            ["hello", "world"]
        );
    }

    #[test]
    fn escape_sequences_are_stripped_and_erased_progress_dropped() {
        let gradle = "\u{1b}[2K\r\u{1b}[1A\n<===> 80% EXECUTING\u{1b}[0K\r\n\u{1b}[2K\n> Task :app:build\n\nBUILD SUCCESSFUL\n\u{1b}]0;title\u{7}";
        assert_eq!(
            texts(&[out(gradle)]),
            [
                "<===> 80% EXECUTING",
                "> Task :app:build",
                "",
                "BUILD SUCCESSFUL"
            ],
            "real blank lines stay, erased progress lines go"
        );
        assert_eq!(texts(&[out("\u{1b}[32mgreen\u{1b}[0m\n")]), ["green"]);
    }

    #[test]
    fn stderr_lines_keep_their_stream() {
        let lines = console_lines(&[
            out("ok\n"),
            OutputChunk {
                stream: OutputStream::Stderr,
                text: "error: x\n".into(),
            },
        ]);
        assert_eq!(lines[0].stream, OutputStream::Stdout);
        assert_eq!(lines[1].stream, OutputStream::Stderr);
    }

    #[test]
    fn viewport_holds_whole_rows() {
        assert_eq!(whole_rows_height(250.0, 18.0), 234.0);
        assert_eq!(whole_rows_height(10.0, 18.0), 18.0);
        assert_eq!(whole_rows_height(100.0, 0.0), 100.0);
    }
}
