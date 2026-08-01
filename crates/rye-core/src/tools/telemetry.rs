//! Goal 136: Telemetry / tracing.
//! Goal 137: Crash reporting.
//!
//! Structured logging, distributed tracing, and crash reporting for
//! production rye apps. Integrates with OpenTelemetry on server,
//! Sentry-style crash reporting on client.

use std::collections::HashMap;

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Trace (most verbose).
    Trace,
    /// Debug information.
    Debug,
    /// General information.
    Info,
    /// Warning.
    Warn,
    /// Error.
    Error,
    /// Fatal (crash).
    Fatal,
}

impl LogLevel {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }
}

/// A log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp (Unix milliseconds).
    pub timestamp: u64,
    /// Log level.
    pub level: LogLevel,
    /// Message.
    pub message: String,
    /// Target/module name.
    pub target: String,
    /// Structured fields.
    pub fields: HashMap<String, String>,
    /// Trace ID (for distributed tracing).
    pub trace_id: Option<String>,
    /// Span ID.
    pub span_id: Option<String>,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(level: LogLevel, message: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            timestamp: current_timestamp(),
            level,
            message: message.into(),
            target: target.into(),
            fields: HashMap::new(),
            trace_id: None,
            span_id: None,
        }
    }

    /// Add a structured field.
    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Set trace ID.
    pub fn trace(mut self, id: impl Into<String>) -> Self {
        self.trace_id = Some(id.into());
        self
    }

    /// Set span ID.
    pub fn span(mut self, id: impl Into<String>) -> Self {
        self.span_id = Some(id.into());
        self
    }

    /// Format as a structured log line.
    pub fn format(&self) -> String {
        let mut line = format!(
            "[{}] {} [{}] {}",
            self.timestamp,
            self.level.as_str(),
            self.target,
            self.message
        );

        if !self.fields.is_empty() {
            let fields: Vec<String> = self.fields.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            line.push_str(&format!(" {{{}}}", fields.join(", ")));
        }

        if let Some(tid) = &self.trace_id {
            line.push_str(&format!(" trace={}", tid));
        }

        line
    }
}

/// Get current timestamp in milliseconds.
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// A trace span for distributed tracing.
#[derive(Debug, Clone)]
pub struct TraceSpan {
    /// Span ID.
    pub span_id: String,
    /// Trace ID.
    pub trace_id: String,
    /// Parent span ID.
    pub parent_id: Option<String>,
    /// Span name.
    pub name: String,
    /// Start timestamp.
    pub start: u64,
    /// End timestamp (None if still open).
    pub end: Option<u64>,
    /// Span attributes.
    pub attributes: HashMap<String, String>,
}

impl TraceSpan {
    /// Start a new span.
    pub fn start(name: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self {
            span_id: generate_id(),
            trace_id: trace_id.into(),
            parent_id: None,
            name: name.into(),
            start: current_timestamp(),
            end: None,
            attributes: HashMap::new(),
        }
    }

    /// Create a child span.
    pub fn child(&self, name: impl Into<String>) -> Self {
        Self {
            span_id: generate_id(),
            trace_id: self.trace_id.clone(),
            parent_id: Some(self.span_id.clone()),
            name: name.into(),
            start: current_timestamp(),
            end: None,
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute.
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// End the span.
    pub fn finish(&mut self) {
        self.end = Some(current_timestamp());
    }

    /// Duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u64> {
        self.end.map(|e| e - self.start)
    }
}

fn generate_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("span-{}", id)
}

/// Telemetry configuration.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Minimum log level to record.
    pub min_level: LogLevel,
    /// Whether to enable tracing.
    pub tracing: bool,
    /// Whether to send logs to a remote endpoint.
    pub remote_endpoint: Option<String>,
    /// Batch size for remote logging.
    pub batch_size: usize,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            tracing: true,
            remote_endpoint: None,
            batch_size: 100,
        }
    }
}

/// Logger — collects and forwards log entries.
pub struct Logger {
    /// Configuration.
    config: TelemetryConfig,
    /// Buffered log entries.
    buffer: Vec<LogEntry>,
}

impl Logger {
    /// Create a new logger.
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            buffer: Vec::new(),
        }
    }

    /// Log an entry.
    pub fn log(&mut self, entry: LogEntry) {
        if entry.level >= self.config.min_level {
            self.buffer.push(entry);
        }
    }

    /// Drain buffered entries.
    pub fn drain(&mut self) -> Vec<LogEntry> {
        std::mem::take(&mut self.buffer)
    }

    /// Number of buffered entries.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

// ===== Crash Reporting (Goal 137) =====

/// A crash report.
#[derive(Debug, Clone)]
pub struct CrashReport {
    /// Crash ID.
    pub id: String,
    /// Error message.
    pub message: String,
    /// Stack trace (if available).
    pub stack_trace: Option<String>,
    /// Timestamp.
    pub timestamp: u64,
    /// App version.
    pub app_version: String,
    /// Platform (wasm, native, etc.).
    pub platform: String,
    /// Additional context.
    pub context: HashMap<String, String>,
}

impl CrashReport {
    /// Create a new crash report.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            id: generate_id(),
            message: message.into(),
            stack_trace: None,
            timestamp: current_timestamp(),
            app_version: String::new(),
            platform: detect_platform(),
            context: HashMap::new(),
        }
    }

    /// Set stack trace.
    pub fn stack(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }

    /// Set app version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.app_version = version.into();
        self
    }

    /// Add context.
    pub fn context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> String {
        let mut json = format!(
            r#"{{"id":"{}","message":"{}","timestamp":{},"platform":"{}","app_version":"{}""#,
            self.id, escape_json(&self.message), self.timestamp, self.platform, self.app_version
        );

        if let Some(trace) = &self.stack_trace {
            json.push_str(&format!(r#","stack_trace":"{}""#, escape_json(trace)));
        }

        if !self.context.is_empty() {
            let ctx: Vec<String> = self.context.iter()
                .map(|(k, v)| format!(r#""{}":"{}""#, escape_json(k), escape_json(v)))
                .collect();
            json.push_str(&format!(r#","context":{{{}}}"#, ctx.join(",")));
        }

        json.push('}');
        json
    }
}

fn detect_platform() -> String {
    #[cfg(target_arch = "wasm32")]
    { "wasm".to_string() }
    #[cfg(not(target_arch = "wasm32"))]
    { "native".to_string() }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

/// Crash reporter — collects and sends crash reports.
pub struct CrashReporter {
    /// Pending crash reports.
    reports: Vec<CrashReport>,
    /// Whether to send reports to a remote endpoint.
    pub remote_endpoint: Option<String>,
}

impl CrashReporter {
    /// Create a new crash reporter.
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
            remote_endpoint: None,
        }
    }

    /// Set remote endpoint.
    pub fn endpoint(mut self, url: impl Into<String>) -> Self {
        self.remote_endpoint = Some(url.into());
        self
    }

    /// Report a crash.
    pub fn report(&mut self, crash: CrashReport) {
        self.reports.push(crash);
    }

    /// Drain pending reports.
    pub fn drain(&mut self) -> Vec<CrashReport> {
        std::mem::take(&mut self.reports)
    }

    /// Number of pending reports.
    pub fn len(&self) -> usize {
        self.reports.len()
    }
}

impl Default for CrashReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate the JS for global error handling and crash reporting.
pub fn crash_handler_script() -> &'static str {
    r#"<script>
(function() {
  window.addEventListener('error', function(event) {
    var report = {
      message: event.message || 'Unknown error',
      stack: event.error && event.error.stack,
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      timestamp: Date.now()
    };
    if (window.__rye_crash_report) {
      window.__rye_crash_report(report);
    }
  });

  window.addEventListener('unhandledrejection', function(event) {
    var reason = event.reason;
    var report = {
      message: 'Unhandled promise rejection: ' + (reason && reason.message || reason),
      stack: reason && reason.stack,
      timestamp: Date.now()
    };
    if (window.__rye_crash_report) {
      window.__rye_crash_report(report);
    }
  });
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }

    #[test]
    fn test_log_entry() {
        let entry = LogEntry::new(LogLevel::Info, "User logged in", "auth")
            .field("user_id", "123")
            .field("method", "password");
        let formatted = entry.format();
        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("User logged in"));
        assert!(formatted.contains("user_id=123"));
    }

    #[test]
    fn test_log_entry_trace() {
        let entry = LogEntry::new(LogLevel::Error, "Database error", "db")
            .trace("trace-abc");
        let formatted = entry.format();
        assert!(formatted.contains("trace=trace-abc"));
    }

    #[test]
    fn test_logger() {
        let config = TelemetryConfig { min_level: LogLevel::Warn, ..Default::default() };
        let mut logger = Logger::new(config);

        logger.log(LogEntry::new(LogLevel::Info, "Info message", "test"));
        logger.log(LogEntry::new(LogLevel::Error, "Error message", "test"));

        assert_eq!(logger.len(), 1); // Only Error passes the Warn filter
    }

    #[test]
    fn test_trace_span() {
        let parent = TraceSpan::start("request", "trace-1");
        let mut child = parent.child("db_query");
        child = child.attr("query", "SELECT * FROM users");
        child.finish();

        assert!(child.duration_ms().is_some());
        assert_eq!(child.parent_id, Some(parent.span_id));
        assert_eq!(child.trace_id, "trace-1");
    }

    #[test]
    fn test_crash_report() {
        let crash = CrashReport::new("Panic: index out of bounds")
            .stack("at /src/main.rs:42")
            .version("1.0.0")
            .context("user", "alice");
        let json = crash.to_json();
        assert!(json.contains("Panic: index out of bounds"));
        assert!(json.contains("stack_trace"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_crash_reporter() {
        let mut reporter = CrashReporter::new().endpoint("https://crash.example.com/report");
        reporter.report(CrashReport::new("Error 1"));
        reporter.report(CrashReport::new("Error 2"));
        assert_eq!(reporter.len(), 2);
        let drained = reporter.drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(reporter.len(), 0);
    }

    #[test]
    fn test_crash_handler_script() {
        let script = crash_handler_script();
        assert!(script.contains("addEventListener('error'"));
        assert!(script.contains("unhandledrejection"));
        assert!(script.contains("__rye_crash_report"));
    }
}
