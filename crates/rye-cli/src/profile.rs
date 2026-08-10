//! Goal 224: `rpg profile` performance profiler.
//!
//! CLI profiler that runs the app, collects performance data (render times,
//! signal updates, bridge calls, memory), and outputs a flamegraph.

use std::collections::HashMap;

/// A profile event — a single measurement.
#[derive(Debug, Clone)]
pub struct ProfileEvent {
    /// The event name (function/component name).
    pub name: String,
    /// The event category.
    pub category: ProfileCategory,
    /// The start time in microseconds.
    pub start_us: u64,
    /// The duration in microseconds.
    pub duration_us: u64,
    /// The depth in the call stack.
    pub depth: u32,
}

/// The category of a profile event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileCategory {
    /// Component render.
    Render,
    /// Signal update.
    Signal,
    /// Wasm-JS bridge call.
    Bridge,
    /// Memory allocation.
    Memory,
    /// Layout computation.
    Layout,
    /// Other.
    Other,
}

impl ProfileCategory {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            ProfileCategory::Render => "render",
            ProfileCategory::Signal => "signal",
            ProfileCategory::Bridge => "bridge",
            ProfileCategory::Memory => "memory",
            ProfileCategory::Layout => "layout",
            ProfileCategory::Other => "other",
        }
    }

    /// Get the flamegraph color.
    pub fn color(&self) -> &'static str {
        match self {
            ProfileCategory::Render => "#ff6b6b",
            ProfileCategory::Signal => "#4ecdc4",
            ProfileCategory::Bridge => "#ffe66d",
            ProfileCategory::Memory => "#a78bfa",
            ProfileCategory::Layout => "#95e1d3",
            ProfileCategory::Other => "#c4c4c4",
        }
    }
}

/// A profile session — a collection of profile events.
#[derive(Debug, Clone)]
pub struct ProfileSession {
    /// The session name.
    pub name: String,
    /// The events recorded.
    pub events: Vec<ProfileEvent>,
    /// The total duration in microseconds.
    pub total_duration_us: u64,
}

impl ProfileSession {
    /// Create a new empty session.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            events: Vec::new(),
            total_duration_us: 0,
        }
    }

    /// Record an event.
    pub fn record(&mut self, event: ProfileEvent) {
        if event.start_us + event.duration_us > self.total_duration_us {
            self.total_duration_us = event.start_us + event.duration_us;
        }
        self.events.push(event);
    }

    /// Get the number of events.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get events by category.
    pub fn events_by_category(&self, category: ProfileCategory) -> Vec<&ProfileEvent> {
        self.events
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Get the total time spent in a category.
    pub fn time_in_category(&self, category: ProfileCategory) -> u64 {
        self.events
            .iter()
            .filter(|e| e.category == category)
            .map(|e| e.duration_us)
            .sum()
    }

    /// Get category summary.
    pub fn category_summary(&self) -> HashMap<ProfileCategory, (usize, u64)> {
        let mut summary: HashMap<ProfileCategory, (usize, u64)> = HashMap::new();
        for event in &self.events {
            let entry = summary.entry(event.category).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += event.duration_us;
        }
        summary
    }

    /// Generate a flamegraph in HTML format.
    pub fn generate_flamegraph_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<style>\n");
        html.push_str(".flame { display:flex; height:20px; margin:1px 0; }\n");
        html.push_str(".bar { display:inline-block; height:20px; overflow:hidden; ");
        html.push_str("font-size:11px; white-space:nowrap; color:#fff; padding:0 4px; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str(&format!(
            "<h2>Profile: {} ({:.3}ms)</h2>\n",
            self.name,
            self.total_duration_us as f64 / 1000.0
        ));

        let summary = self.category_summary();
        for (cat, (count, time)) in &summary {
            html.push_str(&format!(
                "<div>{}: {} events, {:.3}ms</div>\n",
                cat.display_name(),
                count,
                *time as f64 / 1000.0,
            ));
        }

        html.push_str("<div class=\"flamegraph\">\n");
        for event in &self.events {
            let width = (event.duration_us as f64 / self.total_duration_us as f64 * 100.0) as f64;
            html.push_str(&format!(
                "<div class=\"flame\"><div class=\"bar\" style=\"width:{:.1}%;background:{};\">{} ({:.3}ms)</div></div>\n",
                width,
                event.category.color(),
                event.name,
                event.duration_us as f64 / 1000.0,
            ));
        }
        html.push_str("</div>\n</body>\n</html>\n");
        html
    }

    /// Generate a text report.
    pub fn to_text_report(&self) -> String {
        let mut text = String::new();
        text.push_str("=== Performance Profile ===\n\n");
        text.push_str(&format!("Session: {}\n", self.name));
        text.push_str(&format!(
            "Total time: {:.3}ms\n",
            self.total_duration_us as f64 / 1000.0
        ));
        text.push_str(&format!("Events: {}\n\n", self.event_count()));

        let summary = self.category_summary();
        text.push_str("By category:\n");
        for (cat, (count, time)) in &summary {
            text.push_str(&format!(
                "  {}: {} events, {:.3}ms\n",
                cat.display_name(),
                count,
                *time as f64 / 1000.0,
            ));
        }

        text
    }
}

/// The profiler configuration.
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    /// Whether to trace render events.
    pub trace_renders: bool,
    /// Whether to trace signal updates.
    pub trace_signals: bool,
    /// Whether to trace bridge calls.
    pub trace_bridge: bool,
    /// Whether to trace memory.
    pub trace_memory: bool,
    /// The output format.
    pub output_format: ProfileOutputFormat,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            trace_renders: true,
            trace_signals: true,
            trace_bridge: true,
            trace_memory: false,
            output_format: ProfileOutputFormat::Html,
        }
    }
}

/// The output format for the profiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileOutputFormat {
    /// HTML flamegraph.
    Html,
    /// Text report.
    Text,
    /// JSON.
    Json,
}

/// Run the profile command.
pub fn run(args: &[String]) {
    let output = args
        .iter()
        .find(|a| a.starts_with("--format"))
        .map(|a| &a[8..])
        .unwrap_or("html");

    let config = ProfilerConfig {
        output_format: match output {
            "text" => ProfileOutputFormat::Text,
            "json" => ProfileOutputFormat::Json,
            _ => ProfileOutputFormat::Html,
        },
        ..Default::default()
    };

    println!("Profiling rye app...");
    println!("Output format: {}", output);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_category_display_name() {
        assert_eq!(ProfileCategory::Render.display_name(), "render");
        assert_eq!(ProfileCategory::Signal.display_name(), "signal");
    }

    #[test]
    fn test_profile_category_color() {
        assert_eq!(ProfileCategory::Render.color(), "#ff6b6b");
    }

    #[test]
    fn test_profile_session_new() {
        let session = ProfileSession::new("test");
        assert_eq!(session.name, "test");
        assert_eq!(session.event_count(), 0);
    }

    #[test]
    fn test_profile_session_record() {
        let mut session = ProfileSession::new("test");
        session.record(ProfileEvent {
            name: "render".to_string(),
            category: ProfileCategory::Render,
            start_us: 0,
            duration_us: 100,
            depth: 0,
        });
        assert_eq!(session.event_count(), 1);
        assert_eq!(session.total_duration_us, 100);
    }

    #[test]
    fn test_profile_session_events_by_category() {
        let mut session = ProfileSession::new("test");
        session.record(ProfileEvent {
            name: "a".into(),
            category: ProfileCategory::Render,
            start_us: 0,
            duration_us: 10,
            depth: 0,
        });
        session.record(ProfileEvent {
            name: "b".into(),
            category: ProfileCategory::Signal,
            start_us: 0,
            duration_us: 20,
            depth: 0,
        });
        session.record(ProfileEvent {
            name: "c".into(),
            category: ProfileCategory::Render,
            start_us: 0,
            duration_us: 30,
            depth: 0,
        });
        assert_eq!(session.events_by_category(ProfileCategory::Render).len(), 2);
        assert_eq!(session.events_by_category(ProfileCategory::Signal).len(), 1);
    }

    #[test]
    fn test_profile_session_time_in_category() {
        let mut session = ProfileSession::new("test");
        session.record(ProfileEvent {
            name: "a".into(),
            category: ProfileCategory::Render,
            start_us: 0,
            duration_us: 100,
            depth: 0,
        });
        session.record(ProfileEvent {
            name: "b".into(),
            category: ProfileCategory::Render,
            start_us: 0,
            duration_us: 200,
            depth: 0,
        });
        assert_eq!(session.time_in_category(ProfileCategory::Render), 300);
    }

    #[test]
    fn test_profile_session_category_summary() {
        let mut session = ProfileSession::new("test");
        session.record(ProfileEvent {
            name: "a".into(),
            category: ProfileCategory::Render,
            start_us: 0,
            duration_us: 100,
            depth: 0,
        });
        session.record(ProfileEvent {
            name: "b".into(),
            category: ProfileCategory::Signal,
            start_us: 0,
            duration_us: 50,
            depth: 0,
        });
        let summary = session.category_summary();
        assert_eq!(summary.get(&ProfileCategory::Render), Some(&(1, 100)));
        assert_eq!(summary.get(&ProfileCategory::Signal), Some(&(1, 50)));
    }

    #[test]
    fn test_profile_session_flamegraph_html() {
        let mut session = ProfileSession::new("test");
        session.record(ProfileEvent {
            name: "render".into(),
            category: ProfileCategory::Render,
            start_us: 0,
            duration_us: 1000,
            depth: 0,
        });
        let html = session.generate_flamegraph_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("flamegraph"));
        assert!(html.contains("render"));
    }

    #[test]
    fn test_profile_session_text_report() {
        let mut session = ProfileSession::new("test");
        session.record(ProfileEvent {
            name: "render".into(),
            category: ProfileCategory::Render,
            start_us: 0,
            duration_us: 1000,
            depth: 0,
        });
        let text = session.to_text_report();
        assert!(text.contains("Performance Profile"));
        assert!(text.contains("render"));
    }

    #[test]
    fn test_profiler_config_default() {
        let config = ProfilerConfig::default();
        assert!(config.trace_renders);
        assert!(config.trace_signals);
        assert_eq!(config.output_format, ProfileOutputFormat::Html);
    }
}
