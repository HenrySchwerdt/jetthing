use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::{
    io::{Read, Write},
    path::Path,
    sync::mpsc::{self, Receiver},
    thread,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("failed to open pseudo-terminal: {0}")]
    Open(String),
    #[error("failed to spawn terminal process: {0}")]
    Spawn(String),
    #[error("terminal IO failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to resize terminal: {0}")]
    Resize(String),
}

pub struct TerminalSession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    output: Receiver<Vec<u8>>,
    transcript: String,
    exited: bool,
}

impl TerminalSession {
    pub fn spawn_shell(cwd: &Path) -> Result<Self, TerminalError> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        Self::spawn(&shell, &[], cwd)
    }

    pub fn spawn(program: &str, args: &[&str], cwd: &Path) -> Result<Self, TerminalError> {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 100,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| TerminalError::Open(error.to_string()))?;
        let mut command = CommandBuilder::new(program);
        command.args(args);
        command.cwd(cwd);
        command.env("TERM", "xterm-256color");
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| TerminalError::Spawn(error.to_string()))?;
        drop(pair.slave);
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| TerminalError::Open(error.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| TerminalError::Open(error.to_string()))?;
        let (sender, output) = mpsc::channel();
        thread::Builder::new()
            .name("jetthing-terminal-reader".into())
            .spawn(move || {
                let mut buffer = [0_u8; 8192];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => {
                            if sender.send(buffer[..read].to_vec()).is_err() {
                                break;
                            }
                        }
                    }
                }
            })?;
        Ok(Self {
            master: pair.master,
            writer,
            child,
            output,
            transcript: String::new(),
            exited: false,
        })
    }

    pub fn send(&mut self, text: &str) -> Result<(), TerminalError> {
        self.writer.write_all(text.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), TerminalError> {
        self.master
            .resize(PtySize {
                rows: rows.max(1),
                cols: cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| TerminalError::Resize(error.to_string()))
    }

    pub fn poll(&mut self) -> &str {
        while let Ok(bytes) = self.output.try_recv() {
            self.transcript.push_str(&String::from_utf8_lossy(&bytes));
        }
        if !self.exited {
            self.exited = self.child.try_wait().ok().flatten().is_some();
        }
        &self.transcript
    }

    pub fn is_exited(&self) -> bool {
        self.exited
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if !self.exited {
            let _ = self.child.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn captures_output_from_a_real_pty_process() {
        let mut terminal = TerminalSession::spawn(
            "/bin/sh",
            &["-c", "printf 'jetthing-pty-ok\\n'"],
            Path::new("/tmp"),
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if terminal.poll().contains("jetthing-pty-ok") {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("PTY output was not captured: {:?}", terminal.poll());
    }
}
