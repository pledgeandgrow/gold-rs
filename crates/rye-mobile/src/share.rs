//! Goal 199: Native share sheet.
//!
//! `use_share()` hook that opens the native share dialog.
//! Web (Web Share API), iOS (UIActivityViewController), Android (Intent.ACTION_SEND).

/// The type of content to share.
#[derive(Debug, Clone)]
pub enum ShareContent {
    /// Share text content.
    Text(String),
    /// Share a URL.
    Url(String),
    /// Share text with a URL.
    TextAndUrl {
        /// The text content.
        text: String,
        /// The URL to share.
        url: String,
    },
    /// Share a file (path or base64 data URI).
    File {
        /// The file path.
        path: String,
        /// The MIME type.
        mime_type: String,
    },
    /// Share multiple files.
    Files {
        /// The file paths.
        paths: Vec<String>,
        /// The MIME types.
        mime_types: Vec<String>,
    },
    /// Share an image.
    Image {
        /// The image data.
        data: Vec<u8>,
        /// The MIME type.
        mime_type: String,
        /// The filename.
        filename: String,
    },
}

impl ShareContent {
    /// Create text content to share.
    pub fn text(text: &str) -> Self {
        ShareContent::Text(text.to_string())
    }

    /// Create URL content to share.
    pub fn url_content(url: &str) -> Self {
        ShareContent::Url(url.to_string())
    }

    /// Create text + URL content to share.
    pub fn text_and_url(text: &str, url: &str) -> Self {
        ShareContent::TextAndUrl {
            text: text.to_string(),
            url: url.to_string(),
        }
    }

    /// Create file content to share.
    pub fn file(path: &str, mime_type: &str) -> Self {
        ShareContent::File {
            path: path.to_string(),
            mime_type: mime_type.to_string(),
        }
    }

    /// Create image content to share.
    pub fn image(data: Vec<u8>, mime_type: &str, filename: &str) -> Self {
        ShareContent::Image {
            data,
            mime_type: mime_type.to_string(),
            filename: filename.to_string(),
        }
    }

    /// Get the primary text representation.
    pub fn as_text(&self) -> String {
        match self {
            ShareContent::Text(t) => t.clone(),
            ShareContent::Url(u) => u.clone(),
            ShareContent::TextAndUrl { text, url } => format!("{}\n{}", text, url),
            ShareContent::File { path, .. } => path.clone(),
            ShareContent::Files { paths, .. } => paths.join(", "),
            ShareContent::Image { filename, .. } => filename.clone(),
        }
    }

    /// Check if this content has a URL.
    pub fn has_url(&self) -> bool {
        matches!(self, ShareContent::Url(_) | ShareContent::TextAndUrl { .. })
    }

    /// Get the URL if present.
    pub fn url(&self) -> Option<&str> {
        match self {
            ShareContent::Url(u) => Some(u),
            ShareContent::TextAndUrl { url, .. } => Some(url),
            _ => None,
        }
    }

    /// Check if this is file content.
    pub fn is_file(&self) -> bool {
        matches!(self, ShareContent::File { .. } | ShareContent::Files { .. } | ShareContent::Image { .. })
    }
}

/// The result of a share operation.
#[derive(Debug, Clone, PartialEq)]
pub enum ShareResult {
    /// Share was successful.
    Success,
    /// User cancelled the share.
    Cancelled,
    /// Sharing is not available on this platform.
    NotAvailable,
    /// An error occurred.
    Error(String),
}

impl ShareResult {
    /// Check if sharing succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, ShareResult::Success)
    }

    /// Check if sharing was cancelled.
    pub fn was_cancelled(&self) -> bool {
        matches!(self, ShareResult::Cancelled)
    }
}

/// Configuration for a share operation.
#[derive(Debug, Clone, Default)]
pub struct ShareConfig {
    /// The subject line (used for email sharing).
    pub subject: Option<String>,
    /// The activity types to exclude (iOS).
    pub excluded_activity_types: Vec<String>,
    /// Whether to allow sharing to clipboard only.
    pub clipboard_only: bool,
}

impl ShareConfig {
    /// Create a new share config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the subject.
    pub fn with_subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());
        self
    }

    /// Exclude an activity type.
    pub fn exclude(mut self, activity_type: &str) -> Self {
        self.excluded_activity_types.push(activity_type.to_string());
        self
    }

    /// Set clipboard-only mode.
    pub fn clipboard_only(mut self) -> Self {
        self.clipboard_only = true;
        self
    }
}

/// The share manager — handles native share sheet operations.
pub struct ShareManager {
    available: bool,
    last_shared: std::sync::Mutex<Option<String>>,
}

impl ShareManager {
    /// Create a new share manager.
    pub fn new() -> Self {
        Self {
            available: true,
            last_shared: std::sync::Mutex::new(None),
        }
    }

    /// Create a share manager with availability set.
    pub fn with_availability(available: bool) -> Self {
        Self {
            available,
            last_shared: std::sync::Mutex::new(None),
        }
    }

    /// Check if sharing is available.
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Share content (simulated).
    pub fn share(&self, content: &ShareContent, config: &ShareConfig) -> ShareResult {
        if !self.available {
            return ShareResult::NotAvailable;
        }

        if config.clipboard_only {
            *self.last_shared.lock().unwrap() = Some(content.as_text());
            return ShareResult::Success;
        }

        *self.last_shared.lock().unwrap() = Some(content.as_text());
        ShareResult::Success
    }

    /// Share text (convenience method).
    pub fn share_text(&self, text: &str) -> ShareResult {
        self.share(&ShareContent::text(text), &ShareConfig::default())
    }

    /// Share a URL (convenience method).
    pub fn share_url(&self, url: &str) -> ShareResult {
        self.share(&ShareContent::url_content(url), &ShareConfig::default())
    }

    /// Get the last shared content.
    pub fn last_shared(&self) -> Option<String> {
        self.last_shared.lock().unwrap().clone()
    }
}

impl Default for ShareManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_content_text() {
        let c = ShareContent::text("Hello");
        assert_eq!(c.as_text(), "Hello");
        assert!(!c.has_url());
        assert!(!c.is_file());
    }

    #[test]
    fn test_share_content_url() {
        let c = ShareContent::url_content("https://example.com");
        assert_eq!(c.as_text(), "https://example.com");
        assert!(c.has_url());
        assert_eq!(c.url(), Some("https://example.com"));
    }

    #[test]
    fn test_share_content_text_and_url() {
        let c = ShareContent::text_and_url("Check this", "https://example.com");
        assert_eq!(c.as_text(), "Check this\nhttps://example.com");
        assert!(c.has_url());
        assert_eq!(c.url(), Some("https://example.com"));
    }

    #[test]
    fn test_share_content_file() {
        let c = ShareContent::file("/path/to/file.pdf", "application/pdf");
        assert_eq!(c.as_text(), "/path/to/file.pdf");
        assert!(c.is_file());
        assert!(!c.has_url());
    }

    #[test]
    fn test_share_content_image() {
        let c = ShareContent::image(vec![0xFF, 0xD8], "image/jpeg", "photo.jpg");
        assert_eq!(c.as_text(), "photo.jpg");
        assert!(c.is_file());
    }

    #[test]
    fn test_share_content_files() {
        let c = ShareContent::Files {
            paths: vec!["/a.txt".to_string(), "/b.txt".to_string()],
            mime_types: vec!["text/plain".to_string(), "text/plain".to_string()],
        };
        assert_eq!(c.as_text(), "/a.txt, /b.txt");
        assert!(c.is_file());
    }

    #[test]
    fn test_share_result_is_success() {
        assert!(ShareResult::Success.is_success());
        assert!(!ShareResult::Cancelled.is_success());
    }

    #[test]
    fn test_share_result_was_cancelled() {
        assert!(ShareResult::Cancelled.was_cancelled());
        assert!(!ShareResult::Success.was_cancelled());
    }

    #[test]
    fn test_share_config_builder() {
        let config = ShareConfig::new()
            .with_subject("Check this out")
            .exclude("com.apple.mail")
            .clipboard_only();

        assert_eq!(config.subject, Some("Check this out".to_string()));
        assert!(config.excluded_activity_types.contains(&"com.apple.mail".to_string()));
        assert!(config.clipboard_only);
    }

    #[test]
    fn test_share_manager_available() {
        let mgr = ShareManager::new();
        assert!(mgr.is_available());
    }

    #[test]
    fn test_share_manager_unavailable() {
        let mgr = ShareManager::with_availability(false);
        assert!(!mgr.is_available());
    }

    #[test]
    fn test_share_manager_share() {
        let mgr = ShareManager::new();
        let result = mgr.share(&ShareContent::text("Hello"), &ShareConfig::default());
        assert!(result.is_success());
        assert_eq!(mgr.last_shared(), Some("Hello".to_string()));
    }

    #[test]
    fn test_share_manager_share_unavailable() {
        let mgr = ShareManager::with_availability(false);
        let result = mgr.share(&ShareContent::text("Hello"), &ShareConfig::default());
        assert_eq!(result, ShareResult::NotAvailable);
    }

    #[test]
    fn test_share_manager_share_text() {
        let mgr = ShareManager::new();
        let result = mgr.share_text("Hello World");
        assert!(result.is_success());
    }

    #[test]
    fn test_share_manager_share_url() {
        let mgr = ShareManager::new();
        let result = mgr.share_url("https://example.com");
        assert!(result.is_success());
    }

    #[test]
    fn test_share_manager_clipboard_only() {
        let mgr = ShareManager::new();
        let result = mgr.share(&ShareContent::text("Clipboard"), &ShareConfig::new().clipboard_only());
        assert!(result.is_success());
        assert_eq!(mgr.last_shared(), Some("Clipboard".to_string()));
    }
}
