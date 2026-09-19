use async_drop::{AsyncDrop, AsyncDropFuture, Dropper};
use colored::Colorize;
use futures::io::AsyncWriteExt;
use rustyline_async::{Readline, ReadlineEvent};
use std::sync::Mutex as StdMutex;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

// Global Sender for your macros to use
lazy_static::lazy_static! {
    static ref GLOBAL_LOG_TX: StdMutex<Option<mpsc::UnboundedSender<(String, ColorValue)>>> = StdMutex::new(None);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for ColorValue {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }
}

impl ColorValue {
    pub fn is_uncolored(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }
}

/// The inner state of our console that manages the background task
pub struct ConsoleState {
    shutdown_token: CancellationToken,
    task_handle: Mutex<Option<JoinHandle<()>>>,
}

// Implement AsyncDrop to automatically shut down and flush
impl AsyncDrop for ConsoleState {
    fn async_drop(&mut self) -> AsyncDropFuture<'_> {
        Box::pin(async {
            self.shutdown_token.cancel();

            // Wait for the background task to flush queues and shut down
            if let Some(handle) = self.task_handle.lock().await.take() {
                let _ = handle.await;
            }
            Ok(())
        })
    }
}

/// The combined console that handles both READ (user input) and WRITE (logging)
pub type SharedConsole = Dropper<ConsoleState>;

pub enum ReadEvent {
    Line(String),
    Interrupted,
}

/// Initializes the console and returns the `Dropper` (which handles shutdown)
/// and a receiver to process user input commands.
pub async fn init_console(prompt: &str) -> (SharedConsole, mpsc::UnboundedReceiver<ReadEvent>) {
    let (mut rl, mut out) = Readline::new(prompt.to_owned()).expect("Failed to create Readline");

    let (input_tx, input_rx) = mpsc::unbounded_channel::<ReadEvent>();
    let (log_tx, mut log_rx) = mpsc::unbounded_channel::<(String, ColorValue)>();

    *GLOBAL_LOG_TX.lock().unwrap() = Some(log_tx);

    let shutdown_token = CancellationToken::new();
    let token_clone = shutdown_token.clone();

    let handle = tokio::spawn(async move {
        let mut readline = true;

        loop {
            if !readline {
                // Some workaround to ensure the log output is flushed after interruption
                let _ = rl.flush();
            }

            tokio::select! {
                _ = token_clone.cancelled() => {
                    break;
                }

                Some((line, color)) = log_rx.recv() => {
                    let formatted = if !color.is_uncolored() {
                        format!("{}\n", line.truecolor(color.r, color.g, color.b))
                    } else {
                        format!("{}\n", line)
                    };

                    // Write to rustyline. If it fails, fallback to standard print.
                    if out.write_all(formatted.as_bytes()).await.is_err() {
                        print!("{}", formatted);
                    }
                }

                res = rl.readline(), if readline => {
                    match res {
                        Ok(ReadlineEvent::Line(line)) => {
                            rl.add_history_entry(line.clone());

                            let _ = input_tx.send(ReadEvent::Line(line));
                        }
                        Ok(ReadlineEvent::Interrupted | ReadlineEvent::Eof) | Err(_) => {
                            let _ = input_tx.send(ReadEvent::Interrupted);

                            // Workaround on removing the prompt after interruption.
                            send(String::new(), ColorValue::default());
                            let _ = rl.update_prompt("");

                            readline = false;
                        }
                    }
                }
            }
        }

        while let Ok((line, color)) = log_rx.try_recv() {
            let formatted = if !color.is_uncolored() {
                format!("{}\n", line.truecolor(color.r, color.g, color.b))
            } else {
                format!("{}\n", line)
            };

            if out.write_all(formatted.as_bytes()).await.is_err() {
                print!("{}", formatted);
            }
        }

        if out.flush().await.is_err() {
            std::println!("[Debug] Failed to flush log output");
        }
    });

    let state = ConsoleState {
        shutdown_token,
        task_handle: Mutex::new(Some(handle)),
    };

    (Dropper::new(state), input_rx)
}

pub fn send(line: String, color: ColorValue) {
    let guard = GLOBAL_LOG_TX.lock().unwrap();

    if let Some(writer) = guard.as_ref() {
        if writer.send((line.clone(), color)).is_err() {
            std::println!("{}", line.truecolor(color.r, color.g, color.b));
        }
    } else {
        std::println!("{}", line);
    }
}

#[macro_export]
macro_rules! output {
    ($($arg:tt)*) => {{
        log::send(format!($($arg)*), log::ColorValue::default());
    }};
}

#[macro_export]
macro_rules! info {
    () => {{
        log::send(String::new(), log::ColorValue::default());
    }};
    ($($arg:tt)*) => {{
        log::send(format!("[INFO] {}", format!($($arg)*)), log::ColorValue::default());
    }};
}

#[macro_export]
macro_rules! error {
    () => {{
        log::send(String::new(), log::ColorValue { r: 255, g: 0, b: 0 });
    }};
    ($($arg:tt)*) => {{
        log::send(format!("[ERROR] {}", format!($($arg)*)), log::ColorValue { r: 255, g: 0, b: 0 });
    }};
}

#[macro_export]
macro_rules! warn {
    () => {{
        log::send(String::new(), log::ColorValue { r: 255, g: 255, b: 0 });
    }};
    ($($arg:tt)*) => {{
        log::send(format!("[WARN] {}", format!($($arg)*)), log::ColorValue { r: 255, g: 255, b: 0 });
    }};
}
