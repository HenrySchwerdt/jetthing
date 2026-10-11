//! Debug Adapter Protocol transport: `Content-Length` framed JSON over a child
//! process's stdio, read on a background thread.

use crate::DapError;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

/// Reads one framed message. Returns `Ok(None)` at end of stream.
pub fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, DapError> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.trim().eq_ignore_ascii_case("content-length")
        {
            length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| DapError::Protocol(format!("bad Content-Length `{value}`")))?,
            );
        }
    }
    let length = length.ok_or_else(|| DapError::Protocol("missing Content-Length".into()))?;
    if length > 64 * 1024 * 1024 {
        return Err(DapError::Protocol(format!(
            "message of {length} bytes is too large"
        )));
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|error| DapError::Protocol(format!("invalid JSON: {error}")))
}

pub fn write_message(writer: &mut impl Write, message: &Value) -> Result<(), DapError> {
    let body = serde_json::to_vec(message)
        .map_err(|error| DapError::Protocol(format!("cannot encode message: {error}")))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

/// A running adapter process.
pub struct AdapterProcess {
    child: Child,
    stdin: ChildStdin,
    incoming: Receiver<Result<Value, String>>,
    stderr: Receiver<String>,
}

impl AdapterProcess {
    pub fn spawn(program: &str, args: &[String], cwd: &Path) -> Result<Self, DapError> {
        let mut child = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| DapError::Spawn {
                program: program.to_owned(),
                error: error.to_string(),
            })?;
        let stdin = child.stdin.take().ok_or(DapError::Closed)?;
        let stdout = child.stdout.take().ok_or(DapError::Closed)?;
        let stderr_pipe = child.stderr.take().ok_or(DapError::Closed)?;
        let (sender, incoming) = mpsc::channel();
        thread::Builder::new()
            .name("jetthing-dap-reader".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    match read_message(&mut reader) {
                        Ok(Some(message)) => {
                            if sender.send(Ok(message)).is_err() {
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(error) => {
                            let _ = sender.send(Err(error.to_string()));
                            break;
                        }
                    }
                }
            })?;
        let (err_sender, stderr) = mpsc::channel();
        thread::Builder::new()
            .name("jetthing-dap-stderr".into())
            .spawn(move || {
                let mut pipe = stderr_pipe;
                let mut buffer = [0_u8; 4096];
                while let Ok(read) = pipe.read(&mut buffer) {
                    if read == 0
                        || err_sender
                            .send(String::from_utf8_lossy(&buffer[..read]).into_owned())
                            .is_err()
                    {
                        break;
                    }
                }
            })?;
        Ok(Self {
            child,
            stdin,
            incoming,
            stderr,
        })
    }

    pub fn send(&mut self, message: &Value) -> Result<(), DapError> {
        write_message(&mut self.stdin, message)
    }

    /// Messages received since the last call; an error ends the stream.
    pub fn receive(&mut self) -> Vec<Result<Value, String>> {
        self.incoming.try_iter().collect()
    }

    pub fn stderr(&mut self) -> String {
        self.stderr.try_iter().collect()
    }

    pub fn has_exited(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(Some(_)))
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for AdapterProcess {
    fn drop(&mut self) {
        self.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn frames_round_trip_and_reject_garbage() {
        let mut buffer = Vec::new();
        write_message(
            &mut buffer,
            &json!({"seq": 1, "type": "request", "command": "initialize"}),
        )
        .unwrap();
        write_message(&mut buffer, &json!({"seq": 2})).unwrap();
        let mut reader = std::io::Cursor::new(buffer);
        assert_eq!(
            read_message(&mut reader).unwrap().unwrap()["command"],
            "initialize"
        );
        assert_eq!(read_message(&mut reader).unwrap().unwrap()["seq"], 2);
        assert!(read_message(&mut reader).unwrap().is_none());
        let mut bad = std::io::Cursor::new(b"Content-Length: x\r\n\r\n".to_vec());
        assert!(read_message(&mut bad).is_err());
        let mut missing = std::io::Cursor::new(b"Other: 1\r\n\r\n{}".to_vec());
        assert!(read_message(&mut missing).is_err());
    }
}
