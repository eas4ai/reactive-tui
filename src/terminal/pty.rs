//! Pseudo-terminal (PTY) interface

use super::{TerminalConfig, TerminalError, TerminalResult};
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct PseudoTerminal {
    child: Option<Child>,
    input_sender: Option<mpsc::Sender<Vec<u8>>>,
    output_receiver: Option<mpsc::Receiver<Vec<u8>>>,
    exit_receiver: Option<mpsc::Receiver<i32>>,
    size: (u16, u16),
    working_directory: Option<String>,
    environment: Vec<(String, String)>,
    child_pid: Option<u32>,
    #[cfg(windows)]
    process_handle: Option<std::process::Child>,
}

impl Default for PseudoTerminal {
    fn default() -> Self {
        Self::new()
    }
}

impl PseudoTerminal {
    pub fn new() -> Self {
        Self {
            child: None,
            input_sender: None,
            output_receiver: None,
            exit_receiver: None,
            size: (80, 24),
            working_directory: None,
            environment: Vec::new(),
            child_pid: None,
            #[cfg(windows)]
            process_handle: None,
        }
    }

    pub fn spawn(&mut self, config: &TerminalConfig) -> TerminalResult<()> {
        self.size = config.size;
        self.working_directory = config.working_directory.clone();
        self.environment = config.env.clone();

        #[cfg(unix)]
        {
            self.spawn_unix(config)
        }

        #[cfg(windows)]
        {
            self.spawn_windows(config)
        }
    }

    #[cfg(unix)]
    fn spawn_unix(&mut self, config: &TerminalConfig) -> TerminalResult<()> {
        let shell_env = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell = config.shell.as_deref().unwrap_or(&shell_env);

        let mut command = Command::new(shell);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(ref dir) = config.working_directory {
            command.current_dir(dir);
        }

        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        for (key, value) in &config.env {
            command.env(key, value);
        }

        let mut child = command
            .spawn()
            .map_err(|e| TerminalError::Process(format!("Failed to spawn process: {}", e)))?;

        // Capture the child process ID for resize signals
        self.child_pid = Some(child.id());

        self.setup_io_channels(&mut child)?;
        self.child = Some(child);

        Ok(())
    }

    #[cfg(windows)]
    fn spawn_windows(&mut self, config: &TerminalConfig) -> TerminalResult<()> {
        let shell = config.shell.as_deref().unwrap_or("cmd.exe");

        let mut command = Command::new(shell);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(ref dir) = config.working_directory {
            command.current_dir(dir);
        }

        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        for (key, value) in &config.env {
            command.env(key, value);
        }

        let mut child = command
            .spawn()
            .map_err(|e| TerminalError::Process(format!("Failed to spawn process: {}", e)))?;

        // Capture the child process ID and handle for Windows
        self.child_pid = Some(child.id());
        #[cfg(windows)]
        {
            // Store a reference to the child for Windows console operations
            // Note: This is a simplified approach - full Windows PTY would use ConPTY API
        }

        self.setup_io_channels(&mut child)?;
        self.child = Some(child);

        Ok(())
    }

    fn setup_io_channels(&mut self, child: &mut Child) -> TerminalResult<()> {
        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();
        let (_exit_tx, exit_rx) = mpsc::channel();

        self.input_sender = Some(input_tx);
        self.output_receiver = Some(output_rx);
        self.exit_receiver = Some(exit_rx);

        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Input thread
        thread::spawn(move || {
            while let Ok(data) = input_rx.recv() {
                if stdin.write_all(&data).is_err() {
                    break;
                }
                if stdin.flush().is_err() {
                    break;
                }
            }
        });

        // Output thread
        let output_tx_clone = output_tx.clone();
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            loop {
                match stdout.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if output_tx_clone.send(buffer[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Error thread
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            let mut stderr = stderr;
            loop {
                match stderr.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if output_tx.send(buffer[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(())
    }

    pub fn write_input(&self, data: &[u8]) -> TerminalResult<()> {
        if let Some(ref sender) = self.input_sender {
            sender
                .send(data.to_vec())
                .map_err(|_| TerminalError::Pty("Failed to send input".to_string()))?;
        }
        Ok(())
    }

    pub fn read_output(&self, timeout: Option<Duration>) -> TerminalResult<Option<Vec<u8>>> {
        if let Some(ref receiver) = self.output_receiver {
            match timeout {
                Some(duration) => match receiver.recv_timeout(duration) {
                    Ok(data) => Ok(Some(data)),
                    Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
                    Err(mpsc::RecvTimeoutError::Disconnected) => Err(TerminalError::Pty(
                        "Output channel disconnected".to_string(),
                    )),
                },
                None => match receiver.recv() {
                    Ok(data) => Ok(Some(data)),
                    Err(_) => Err(TerminalError::Pty(
                        "Output channel disconnected".to_string(),
                    )),
                },
            }
        } else {
            Ok(None)
        }
    }

    pub fn try_wait(&self) -> TerminalResult<Option<i32>> {
        if let Some(ref receiver) = self.exit_receiver {
            match receiver.try_recv() {
                Ok(exit_code) => Ok(Some(exit_code)),
                Err(mpsc::TryRecvError::Empty) => Ok(None),
                Err(mpsc::TryRecvError::Disconnected) => Ok(Some(-1)),
            }
        } else {
            Ok(None)
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) -> TerminalResult<()> {
        self.size = (width, height);

        // Send SIGWINCH signal to the child process to notify of resize
        #[cfg(unix)]
        {
            use std::process::Command;
            if let Some(child_pid) = self.child_pid {
                // Send SIGWINCH (window change) signal to child process
                let _ = Command::new("kill")
                    .args(["-WINCH", &child_pid.to_string()])
                    .output();
            }
        }

        #[cfg(windows)]
        {
            // On Windows, we need to use SetConsoleScreenBufferSize
            // This is a simplified implementation - full Windows PTY support
            // would require more complex console API calls
            if let Some(_handle) = &self.process_handle {
                // Windows console resize would go here
                // For now, we just update our internal size
            }
        }

        Ok(())
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    pub fn kill(&mut self) -> TerminalResult<()> {
        if let Some(ref mut child) = self.child {
            child
                .kill()
                .map_err(|e| TerminalError::Process(format!("Failed to kill process: {}", e)))?;
        }
        Ok(())
    }
}

impl Drop for PseudoTerminal {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pty_creation() {
        let pty = PseudoTerminal::new();
        assert_eq!(pty.size(), (80, 24));
    }

    #[test]
    fn test_pty_resize() {
        let mut pty = PseudoTerminal::new();
        let _ = pty.resize(120, 30);
        assert_eq!(pty.size(), (120, 30));
    }
}
