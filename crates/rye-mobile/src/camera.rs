//! Goal 200: Native camera & photo gallery.
//!
//! `use_camera()` hook for capturing photos/video. `use_photo_gallery()` for picking from gallery.

use std::sync::Mutex;

/// The camera direction (front/back).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraDirection {
    /// Back/rear camera.
    Back,
    /// Front-facing camera.
    Front,
}

/// The media type to capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureType {
    /// Capture a photo.
    Photo,
    /// Capture a video.
    Video,
    /// Capture audio (not commonly used with camera).
    Audio,
}

/// Configuration for a camera capture request.
#[derive(Debug, Clone)]
pub struct CameraConfig {
    /// The capture type (photo or video).
    pub capture_type: CaptureType,
    /// The camera direction.
    pub direction: CameraDirection,
    /// Whether to allow editing after capture.
    pub allows_editing: bool,
    /// Maximum video duration in seconds (for video capture).
    pub max_video_duration: Option<u64>,
    /// Whether to save the captured media to the gallery.
    pub save_to_gallery: bool,
    /// The desired image quality (0-1).
    pub quality: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            capture_type: CaptureType::Photo,
            direction: CameraDirection::Back,
            allows_editing: false,
            max_video_duration: None,
            save_to_gallery: true,
            quality: 0.8,
        }
    }
}

impl CameraConfig {
    /// Create a photo capture config.
    pub fn photo() -> Self {
        Self {
            capture_type: CaptureType::Photo,
            ..Default::default()
        }
    }

    /// Create a video capture config.
    pub fn video() -> Self {
        Self {
            capture_type: CaptureType::Video,
            ..Default::default()
        }
    }

    /// Set the camera direction.
    pub fn with_direction(mut self, direction: CameraDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Allow editing after capture.
    pub fn allow_editing(mut self) -> Self {
        self.allows_editing = true;
        self
    }

    /// Set max video duration.
    pub fn with_max_duration(mut self, seconds: u64) -> Self {
        self.max_video_duration = Some(seconds);
        self
    }

    /// Set the quality (0-1).
    pub fn with_quality(mut self, quality: f32) -> Self {
        self.quality = quality.clamp(0.0, 1.0);
        self
    }

    /// Don't save to gallery.
    pub fn dont_save(mut self) -> Self {
        self.save_to_gallery = false;
        self
    }
}

/// The result of a camera capture.
#[derive(Debug, Clone, PartialEq)]
pub struct CapturedMedia {
    /// The file path of the captured media.
    pub path: String,
    /// The MIME type.
    pub mime_type: String,
    /// The media type.
    pub media_type: CaptureType,
    /// The file size in bytes.
    pub size: u64,
    /// The width (for photo/video).
    pub width: Option<u32>,
    /// The height (for photo/video).
    pub height: Option<u32>,
    /// The duration in seconds (for video/audio).
    pub duration: Option<f64>,
}

impl CapturedMedia {
    /// Create a new captured photo.
    pub fn photo(path: &str, size: u64, width: u32, height: u32) -> Self {
        Self {
            path: path.to_string(),
            mime_type: "image/jpeg".to_string(),
            media_type: CaptureType::Photo,
            size,
            width: Some(width),
            height: Some(height),
            duration: None,
        }
    }

    /// Create a new captured video.
    pub fn video(path: &str, size: u64, width: u32, height: u32, duration: f64) -> Self {
        Self {
            path: path.to_string(),
            mime_type: "video/mp4".to_string(),
            media_type: CaptureType::Video,
            size,
            width: Some(width),
            height: Some(height),
            duration: Some(duration),
        }
    }

    /// Check if this is a photo.
    pub fn is_photo(&self) -> bool {
        self.media_type == CaptureType::Photo
    }

    /// Check if this is a video.
    pub fn is_video(&self) -> bool {
        self.media_type == CaptureType::Video
    }
}

/// The result of a camera operation.
#[derive(Debug, Clone, PartialEq)]
pub enum CameraResult {
    /// Capture succeeded with the captured media.
    Success(CapturedMedia),
    /// User cancelled the capture.
    Cancelled,
    /// Camera permission was denied.
    PermissionDenied,
    /// Camera is not available.
    NotAvailable,
    /// An error occurred.
    Error(String),
}

impl CameraResult {
    /// Check if capture succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, CameraResult::Success(_))
    }
}

/// Configuration for photo gallery picking.
#[derive(Debug, Clone)]
pub struct GalleryConfig {
    /// The maximum number of photos to pick (None = unlimited).
    pub max_selection: Option<usize>,
    /// The allowed MIME types (empty = all).
    pub allowed_types: Vec<String>,
    /// Whether to allow editing after selection.
    pub allows_editing: bool,
}

impl Default for GalleryConfig {
    fn default() -> Self {
        Self {
            max_selection: Some(1),
            allowed_types: Vec::new(),
            allows_editing: false,
        }
    }
}

impl GalleryConfig {
    /// Create a single-selection gallery config.
    pub fn single() -> Self {
        Self::default()
    }

    /// Create a multi-selection gallery config.
    pub fn multiple(max: usize) -> Self {
        Self {
            max_selection: Some(max),
            ..Default::default()
        }
    }

    /// Set allowed MIME types.
    pub fn allow_only(mut self, types: &[&str]) -> Self {
        self.allowed_types = types.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Allow editing.
    pub fn allow_editing(mut self) -> Self {
        self.allows_editing = true;
        self
    }
}

/// The result of a gallery pick.
#[derive(Debug, Clone, PartialEq)]
pub enum GalleryResult {
    /// Picking succeeded with the selected media.
    Success(Vec<CapturedMedia>),
    /// User cancelled.
    Cancelled,
    /// Gallery permission was denied.
    PermissionDenied,
    /// Gallery is not available.
    NotAvailable,
    /// An error occurred.
    Error(String),
}

impl GalleryResult {
    /// Check if picking succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, GalleryResult::Success(_))
    }
}

/// The camera manager — handles camera capture and gallery picking.
pub struct CameraManager {
    available: bool,
    has_permission: Mutex<bool>,
    capture_count: Mutex<u32>,
}

impl CameraManager {
    /// Create a new camera manager.
    pub fn new() -> Self {
        Self {
            available: true,
            has_permission: Mutex::new(false),
            capture_count: Mutex::new(0),
        }
    }

    /// Create a camera manager with availability.
    pub fn with_availability(available: bool) -> Self {
        Self {
            available,
            has_permission: Mutex::new(false),
            capture_count: Mutex::new(0),
        }
    }

    /// Check if the camera is available.
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Request camera permission.
    pub fn request_permission(&self) -> bool {
        *self.has_permission.lock().unwrap() = true;
        true
    }

    /// Check if permission is granted.
    pub fn has_permission(&self) -> bool {
        *self.has_permission.lock().unwrap()
    }

    /// Capture photo/video (simulated).
    pub fn capture(&self, config: &CameraConfig) -> CameraResult {
        if !self.available {
            return CameraResult::NotAvailable;
        }

        if !*self.has_permission.lock().unwrap() {
            return CameraResult::PermissionDenied;
        }

        *self.capture_count.lock().unwrap() += 1;

        match config.capture_type {
            CaptureType::Photo => CameraResult::Success(CapturedMedia::photo(
                "/tmp/captured.jpg",
                1024 * 768,
                1920,
                1080,
            )),
            CaptureType::Video => CameraResult::Success(CapturedMedia::video(
                "/tmp/captured.mp4",
                1024 * 1024 * 10,
                1920,
                1080,
                30.0,
            )),
            CaptureType::Audio => CameraResult::Error("Audio capture not supported".to_string()),
        }
    }

    /// Pick from gallery (simulated).
    pub fn pick_from_gallery(&self, config: &GalleryConfig) -> GalleryResult {
        if !*self.has_permission.lock().unwrap() {
            return GalleryResult::PermissionDenied;
        }

        let max = config.max_selection.unwrap_or(1);
        let media: Vec<CapturedMedia> = (0..max)
            .map(|i| CapturedMedia::photo(&format!("/tmp/photo_{}.jpg", i), 500_000, 1080, 1080))
            .collect();

        GalleryResult::Success(media)
    }

    /// Get the number of captures.
    pub fn capture_count(&self) -> u32 {
        *self.capture_count.lock().unwrap()
    }
}

impl Default for CameraManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_direction() {
        assert_ne!(CameraDirection::Back, CameraDirection::Front);
    }

    #[test]
    fn test_camera_config_photo() {
        let config = CameraConfig::photo();
        assert_eq!(config.capture_type, CaptureType::Photo);
        assert_eq!(config.direction, CameraDirection::Back);
    }

    #[test]
    fn test_camera_config_video() {
        let config = CameraConfig::video()
            .with_direction(CameraDirection::Front)
            .with_max_duration(60);
        assert_eq!(config.capture_type, CaptureType::Video);
        assert_eq!(config.direction, CameraDirection::Front);
        assert_eq!(config.max_video_duration, Some(60));
    }

    #[test]
    fn test_camera_config_quality_clamped() {
        let config = CameraConfig::photo().with_quality(2.0);
        assert_eq!(config.quality, 1.0);

        let config2 = CameraConfig::photo().with_quality(-1.0);
        assert_eq!(config2.quality, 0.0);
    }

    #[test]
    fn test_camera_config_builder() {
        let config = CameraConfig::photo()
            .allow_editing()
            .dont_save();
        assert!(config.allows_editing);
        assert!(!config.save_to_gallery);
    }

    #[test]
    fn test_captured_media_photo() {
        let media = CapturedMedia::photo("/test.jpg", 1000, 800, 600);
        assert!(media.is_photo());
        assert!(!media.is_video());
        assert_eq!(media.mime_type, "image/jpeg");
        assert_eq!(media.width, Some(800));
    }

    #[test]
    fn test_captured_media_video() {
        let media = CapturedMedia::video("/test.mp4", 50000, 1920, 1080, 15.5);
        assert!(media.is_video());
        assert!(!media.is_photo());
        assert_eq!(media.mime_type, "video/mp4");
        assert_eq!(media.duration, Some(15.5));
    }

    #[test]
    fn test_camera_result_is_success() {
        assert!(CameraResult::Success(CapturedMedia::photo("/t", 1, 1, 1)).is_success());
        assert!(!CameraResult::Cancelled.is_success());
    }

    #[test]
    fn test_gallery_config_single() {
        let config = GalleryConfig::single();
        assert_eq!(config.max_selection, Some(1));
    }

    #[test]
    fn test_gallery_config_multiple() {
        let config = GalleryConfig::multiple(5);
        assert_eq!(config.max_selection, Some(5));
    }

    #[test]
    fn test_gallery_config_allow_only() {
        let config = GalleryConfig::single().allow_only(&["image/jpeg", "image/png"]);
        assert_eq!(config.allowed_types.len(), 2);
    }

    #[test]
    fn test_gallery_result_is_success() {
        assert!(GalleryResult::Success(vec![]).is_success());
        assert!(!GalleryResult::Cancelled.is_success());
    }

    #[test]
    fn test_manager_available() {
        let mgr = CameraManager::new();
        assert!(mgr.is_available());
    }

    #[test]
    fn test_manager_unavailable() {
        let mgr = CameraManager::with_availability(false);
        assert!(!mgr.is_available());
    }

    #[test]
    fn test_manager_permission() {
        let mgr = CameraManager::new();
        assert!(!mgr.has_permission());
        mgr.request_permission();
        assert!(mgr.has_permission());
    }

    #[test]
    fn test_manager_capture_no_permission() {
        let mgr = CameraManager::new();
        let result = mgr.capture(&CameraConfig::photo());
        assert_eq!(result, CameraResult::PermissionDenied);
    }

    #[test]
    fn test_manager_capture_photo() {
        let mgr = CameraManager::new();
        mgr.request_permission();
        let result = mgr.capture(&CameraConfig::photo());
        assert!(result.is_success());
        assert_eq!(mgr.capture_count(), 1);
    }

    #[test]
    fn test_manager_capture_video() {
        let mgr = CameraManager::new();
        mgr.request_permission();
        let result = mgr.capture(&CameraConfig::video());
        assert!(result.is_success());
        if let CameraResult::Success(media) = result {
            assert!(media.is_video());
        }
    }

    #[test]
    fn test_manager_capture_unavailable() {
        let mgr = CameraManager::with_availability(false);
        mgr.request_permission();
        let result = mgr.capture(&CameraConfig::photo());
        assert_eq!(result, CameraResult::NotAvailable);
    }

    #[test]
    fn test_manager_pick_from_gallery() {
        let mgr = CameraManager::new();
        mgr.request_permission();
        let result = mgr.pick_from_gallery(&GalleryConfig::multiple(3));
        assert!(result.is_success());
        if let GalleryResult::Success(media) = result {
            assert_eq!(media.len(), 3);
        }
    }

    #[test]
    fn test_manager_pick_from_gallery_no_permission() {
        let mgr = CameraManager::new();
        let result = mgr.pick_from_gallery(&GalleryConfig::single());
        assert_eq!(result, GalleryResult::PermissionDenied);
    }
}
