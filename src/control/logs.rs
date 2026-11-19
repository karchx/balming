use std::sync::{LazyLock, Mutex};

use heapless::String;

const LOG_LINES: usize = 40;
const LOG_LINE_LEN: usize = 128;

type LogLine = String<LOG_LINE_LEN>;

pub struct LogsBuffer {
    buf: [LogLine; LOG_LINES],
    head: usize,
}

impl LogsBuffer {
    const fn new() -> Self {
        Self {
            buf: [const { String::new() }; LOG_LINES],
            head: 0,
        }
    }

    fn push(&mut self, line: &str) {
        let slot = &mut self.buf[self.head];
        slot.clear();
        for ch in line.chars() {
            if slot.push(ch).is_err() {
                break;
            }
        }
        self.head = (self.head + 1) % LOG_LINES;
    }

    fn text(&self) -> std::string::String {
        let mut out = std::string::String::with_capacity(1024);
        for i in 0..LOG_LINES {
            let idx = (self.head + i) % LOG_LINES;
            let line = &self.buf[idx];
            if line.is_empty() {
                continue;
            }

            if out.len() + line.len() + 1 > 1024 {
                break;
            }

            out.push_str(line.as_str());
            out.push('\n')
        }
        if out.is_empty() {
            out.push_str("(no logs yet)\n");
        }
        out
    }
}

static LOGS: Mutex<LogsBuffer> = Mutex::new(LogsBuffer::new());

// create new log
pub fn log_line(msg: &str) {
    if let Ok(mut guard) = LOGS.lock() {
        guard.push(msg);
    }
}

pub fn logs_text() -> std::string::String {
    LOGS.lock()
        .map(|g| g.text())
        .unwrap_or_else(|_| std::string::String::from("(log lock failed)\n"))
}
