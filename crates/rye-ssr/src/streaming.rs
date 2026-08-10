//! Streaming SSR — sends HTML chunks as async data resolves.
//!
//! Instead of waiting for all async data to load before sending any HTML,
//! streaming SSR sends the static shell immediately, then streams in
//! the resolved content as each `Suspense` boundary completes.
//!
//! ## How it works
//!
//! 1. The server renders the static shell (everything outside `Suspense`)
//!    and sends it immediately.
//! 2. Each `Suspense` boundary emits a placeholder `<template id="r1">`
//!    with fallback content.
//! 3. As async data resolves, the server streams `<script>` chunks that
//!    replace the placeholder with the resolved content.
//! 4. The client hydrates incrementally as chunks arrive.
//!
//! ## Protocol
//!
//! ```text
//! <div id="root">
//!   <h1>My Blog</h1>
//!   <template id="r1"><!--fallback-->Loading...</template>
//! </div>
//! <script>
//!   document.getElementById('r1').replaceWith(
//!     document.createElement('div').innerHTML = '<p>Resolved content</p>'
//!   );
//! </script>
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use rye_ssr::streaming::StreamingRenderer;
//!
//! let mut renderer = StreamingRenderer::new();
//! renderer.push_chunk("<div id=\"root\"><h1>Hello</h1></div>");
//! renderer.push_suspense("r1", "<p>Loading...</p>");
//! // Later, when data resolves:
//! renderer.resolve_suspense("r1", "<p>Resolved!</p>");
//! ```

use std::collections::HashMap;

/// A chunk of streamed HTML.
#[derive(Debug, Clone)]
pub struct HtmlChunk {
    /// The HTML content for this chunk.
    pub html: String,
    /// Whether this chunk is a suspense resolution.
    pub is_resolution: bool,
    /// The suspense ID this chunk resolves (if applicable).
    pub suspense_id: Option<String>,
}

/// Streaming SSR renderer — produces HTML chunks for streaming responses.
///
/// The renderer buffers chunks and yields them as they become available.
/// Static content is emitted immediately; suspense boundaries are emitted
/// as placeholders and resolved later.
pub struct StreamingRenderer {
    /// Buffered chunks ready to be sent.
    chunks: Vec<HtmlChunk>,
    /// Map of suspense ID → whether it has been resolved.
    suspense_state: HashMap<String, SuspenseState>,
    /// Counter for generating unique suspense IDs.
    next_id: usize,
}

/// State of a suspense boundary in the stream.
#[derive(Debug, Clone, PartialEq)]
pub enum SuspenseState {
    /// Placeholder has been emitted, waiting for resolution.
    Pending,
    /// Content has been resolved and streamed.
    Resolved,
}

impl StreamingRenderer {
    /// Create a new streaming renderer.
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            suspense_state: HashMap::new(),
            next_id: 0,
        }
    }

    /// Generate a unique suspense ID.
    pub fn next_suspense_id(&mut self) -> String {
        let id = format!("r{}", self.next_id);
        self.next_id += 1;
        id
    }

    /// Push a static HTML chunk (no suspense).
    pub fn push_chunk(&mut self, html: impl Into<String>) {
        self.chunks.push(HtmlChunk {
            html: html.into(),
            is_resolution: false,
            suspense_id: None,
        });
    }

    /// Push a suspense boundary with fallback content.
    ///
    /// Emits a `<template>` placeholder that will be replaced when
    /// the suspense resolves.
    pub fn push_suspense(&mut self, id: &str, fallback: impl Into<String>) {
        let fallback_html = fallback.into();
        let placeholder = format!(r#"<template id="{}">{}</template>"#, id, fallback_html);

        self.chunks.push(HtmlChunk {
            html: placeholder,
            is_resolution: false,
            suspense_id: Some(id.to_string()),
        });

        self.suspense_state
            .insert(id.to_string(), SuspenseState::Pending);
    }

    /// Resolve a suspense boundary with the final content.
    ///
    /// Emits a `<script>` chunk that replaces the placeholder with
    /// the resolved content on the client.
    pub fn resolve_suspense(&mut self, id: &str, content: impl Into<String>) {
        let content_html = content.into();
        let escaped = content_html
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n");

        let script = format!(
            r#"<script>(function(){{var t=document.getElementById('{}');if(t){{var d=document.createElement('div');d.innerHTML='{}';t.replaceWith(d.firstChild||d);}}}})();</script>"#,
            id, escaped
        );

        self.chunks.push(HtmlChunk {
            html: script,
            is_resolution: true,
            suspense_id: Some(id.to_string()),
        });

        self.suspense_state
            .insert(id.to_string(), SuspenseState::Resolved);
    }

    /// Check if a suspense boundary has been resolved.
    pub fn is_resolved(&self, id: &str) -> bool {
        self.suspense_state.get(id) == Some(&SuspenseState::Resolved)
    }

    /// Check if there are pending (unresolved) suspense boundaries.
    pub fn has_pending(&self) -> bool {
        self.suspense_state
            .values()
            .any(|s| s == &SuspenseState::Pending)
    }

    /// Drain all buffered chunks.
    ///
    /// Returns all chunks and clears the internal buffer.
    pub fn drain_chunks(&mut self) -> Vec<HtmlChunk> {
        std::mem::take(&mut self.chunks)
    }

    /// Get all buffered chunks without draining.
    pub fn chunks(&self) -> &[HtmlChunk] {
        &self.chunks
    }

    /// Render all chunks to a single HTML string.
    ///
    /// This is useful for non-streaming contexts or testing.
    pub fn to_html(&self) -> String {
        self.chunks
            .iter()
            .map(|c| c.html.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Number of chunks buffered.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

impl Default for StreamingRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// A streaming SSR response.
///
/// Wraps a `StreamingRenderer` and provides an iterator-like interface
/// for consuming chunks as they become available.
pub struct StreamingResponse {
    /// The streaming renderer.
    renderer: StreamingRenderer,
    /// Whether the response has been fully flushed.
    completed: bool,
}

impl StreamingResponse {
    /// Create a new streaming response.
    pub fn new() -> Self {
        Self {
            renderer: StreamingRenderer::new(),
            completed: false,
        }
    }

    /// Create a streaming response from a renderer.
    pub fn from_renderer(renderer: StreamingRenderer) -> Self {
        Self {
            renderer,
            completed: false,
        }
    }

    /// Push a static HTML chunk.
    pub fn push_chunk(&mut self, html: impl Into<String>) {
        self.renderer.push_chunk(html);
    }

    /// Push a suspense boundary.
    pub fn push_suspense(&mut self, id: &str, fallback: impl Into<String>) {
        self.renderer.push_suspense(id, fallback);
    }

    /// Resolve a suspense boundary.
    pub fn resolve_suspense(&mut self, id: &str, content: impl Into<String>) {
        self.renderer.resolve_suspense(id, content);
    }

    /// Drain all available chunks.
    pub fn drain_chunks(&mut self) -> Vec<HtmlChunk> {
        self.renderer.drain_chunks()
    }

    /// Mark the response as complete (no more chunks will be produced).
    pub fn complete(&mut self) {
        self.completed = true;
    }

    /// Check if the response is complete.
    pub fn is_complete(&self) -> bool {
        self.completed && !self.renderer.has_pending()
    }

    /// Get the full HTML (for testing or non-streaming fallback).
    pub fn to_html(&self) -> String {
        self.renderer.to_html()
    }

    /// Get the underlying renderer.
    pub fn renderer(&self) -> &StreamingRenderer {
        &self.renderer
    }

    /// Get a mutable reference to the underlying renderer.
    pub fn renderer_mut(&mut self) -> &mut StreamingRenderer {
        &mut self.renderer
    }
}

impl Default for StreamingResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_basic() {
        let mut r = StreamingRenderer::new();
        r.push_chunk("<div>Hello</div>");
        r.push_chunk("<div>World</div>");

        assert_eq!(r.chunk_count(), 2);
        assert_eq!(r.to_html(), "<div>Hello</div>\n<div>World</div>");
    }

    #[test]
    fn test_streaming_suspense() {
        let mut r = StreamingRenderer::new();
        let id = r.next_suspense_id();
        r.push_chunk("<div id=\"root\">");
        r.push_suspense(&id, "<p>Loading...</p>");
        r.push_chunk("</div>");

        assert!(r.has_pending());
        assert!(!r.is_resolved(&id));

        r.resolve_suspense(&id, "<p>Resolved!</p>");

        assert!(!r.has_pending());
        assert!(r.is_resolved(&id));
    }

    #[test]
    fn test_streaming_suspense_id_unique() {
        let mut r = StreamingRenderer::new();
        let id1 = r.next_suspense_id();
        let id2 = r.next_suspense_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_streaming_drain() {
        let mut r = StreamingRenderer::new();
        r.push_chunk("a");
        r.push_chunk("b");

        let chunks = r.drain_chunks();
        assert_eq!(chunks.len(), 2);
        assert_eq!(r.chunk_count(), 0);
    }

    #[test]
    fn test_streaming_resolution_content() {
        let mut r = StreamingRenderer::new();
        r.push_suspense("r0", "Loading");
        r.resolve_suspense("r0", "<p>Done</p>");

        let chunks = r.drain_chunks();
        assert_eq!(chunks.len(), 2);
        assert!(!chunks[0].is_resolution);
        assert!(chunks[1].is_resolution);
        assert_eq!(chunks[1].suspense_id, Some("r0".to_string()));
    }

    #[test]
    fn test_streaming_response_complete() {
        let mut resp = StreamingResponse::new();
        resp.push_chunk("<html>");
        resp.complete();

        assert!(resp.is_complete());
    }

    #[test]
    fn test_streaming_response_not_complete_with_pending() {
        let mut resp = StreamingResponse::new();
        resp.push_suspense("r0", "Loading");
        resp.complete();

        // Still has pending suspense
        assert!(!resp.is_complete());

        resp.resolve_suspense("r0", "Resolved");
        assert!(resp.is_complete());
    }

    #[test]
    fn test_streaming_to_html() {
        let mut r = StreamingRenderer::new();
        r.push_chunk("<h1>Title</h1>");
        r.push_chunk("<p>Body</p>");

        let html = r.to_html();
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<p>Body</p>"));
    }

    #[test]
    fn test_streaming_suspense_placeholder_format() {
        let mut r = StreamingRenderer::new();
        r.push_suspense("r42", "Fallback");

        let html = r.to_html();
        assert!(html.contains(r#"id="r42""#));
        assert!(html.contains("Fallback"));
        assert!(html.contains("<template"));
    }

    #[test]
    fn test_streaming_resolution_script_format() {
        let mut r = StreamingRenderer::new();
        r.resolve_suspense("r42", "<p>Content</p>");

        let html = r.to_html();
        assert!(html.contains("getElementById('r42')"));
        assert!(html.contains("<script>"));
    }
}
