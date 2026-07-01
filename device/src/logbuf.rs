/// In-memory ring buffer of recent WARN/ERROR log lines, for the dashboard's
/// error log panel. Plugs into `tracing` as an extra `Layer` alongside the
/// normal stdout formatter (see `main.rs`), so nothing about existing logging
/// changes — this just also copies warnings/errors into memory.
use std::collections::VecDeque;
use std::sync::Mutex;

use tracing::field::{Field, Visit};
use tracing::Event;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// Millis since the Unix epoch.
    pub time_ms: i64,
    pub level: String,
    pub target: String,
    pub message: String,
}

pub struct LogBuffer {
    entries: Mutex<VecDeque<LogEntry>>,
    cap: usize,
}

impl LogBuffer {
    pub fn new(cap: usize) -> Self {
        LogBuffer {
            entries: Mutex::new(VecDeque::with_capacity(cap)),
            cap,
        }
    }

    fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.lock().unwrap();
        if entries.len() == self.cap {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    /// Most-recent-first snapshot of the buffered entries.
    pub fn snapshot(&self) -> Vec<LogEntry> {
        self.entries.lock().unwrap().iter().rev().cloned().collect()
    }
}

/// A `tracing_subscriber::Layer` that copies each event it sees into a
/// `LogBuffer`. Compose with a `LevelFilter` (see `main.rs`) to capture only
/// WARN/ERROR — this layer itself has no opinion on level.
pub struct CaptureLayer {
    buffer: std::sync::Arc<LogBuffer>,
}

impl CaptureLayer {
    pub fn new(buffer: std::sync::Arc<LogBuffer>) -> Self {
        CaptureLayer { buffer }
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else if self.message.is_empty() {
            self.message = format!("{}={:?}", field.name(), value);
        }
    }
}

impl<S> Layer<S> for CaptureLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        self.buffer.push(LogEntry {
            time_ms: chrono::Utc::now().timestamp_millis(),
            level: event.metadata().level().to_string(),
            target: event.metadata().target().to_string(),
            message: visitor.message,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_caps_and_orders_most_recent_first() {
        let buf = LogBuffer::new(3);
        for i in 0..5 {
            buf.push(LogEntry {
                time_ms: i,
                level: "WARN".to_string(),
                target: "t".to_string(),
                message: format!("msg {i}"),
            });
        }
        let snap = buf.snapshot();
        assert_eq!(snap.len(), 3);
        // Most recent (4, 3, 2) first, oldest (0, 1) dropped.
        assert_eq!(snap[0].message, "msg 4");
        assert_eq!(snap[1].message, "msg 3");
        assert_eq!(snap[2].message, "msg 2");
    }

    #[test]
    fn empty_buffer_snapshot_is_empty() {
        let buf = LogBuffer::new(10);
        assert!(buf.snapshot().is_empty());
    }
}
