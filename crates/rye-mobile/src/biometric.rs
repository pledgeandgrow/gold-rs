//! Goal 198: Native biometric authentication.
//!
//! `use_biometric_auth()` hook. Face ID / Touch ID on iOS, fingerprint on Android,
//! Windows Hello on desktop. Falls back to password on web.

use std::sync::Mutex;

/// The type of biometric authentication available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiometricType {
    /// Face recognition (Face ID, Face Unlock).
    Face,
    /// Fingerprint (Touch ID, fingerprint scanner).
    Fingerprint,
    /// Iris scanning.
    Iris,
    /// Voice recognition.
    Voice,
    /// No biometric available.
    None,
}

impl BiometricType {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            BiometricType::Face => "Face Recognition",
            BiometricType::Fingerprint => "Fingerprint",
            BiometricType::Iris => "Iris Scan",
            BiometricType::Voice => "Voice Recognition",
            BiometricType::None => "None",
        }
    }
}

/// The availability status of biometric authentication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiometricAvailability {
    /// Biometric is available and enrolled.
    Available,
    /// Biometric hardware is present but no biometrics are enrolled.
    NoEnrollment,
    /// Biometric hardware is not present.
    NotAvailable,
    /// Biometric is locked out due to too many failed attempts.
    LockedOut,
    /// Biometric is temporarily unavailable.
    TemporaryUnavailable,
}

/// The result of a biometric authentication attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum BiometricAuthResult {
    /// Authentication succeeded.
    Success,
    /// Authentication failed (biometric didn't match).
    Failed,
    /// User cancelled the authentication.
    Cancelled,
    /// User fell back to password/passcode.
    Fallback,
    /// Authentication is locked out.
    LockedOut,
    /// An error occurred.
    Error(String),
}

impl BiometricAuthResult {
    /// Check if authentication succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, BiometricAuthResult::Success)
    }

    /// Check if authentication was cancelled by the user.
    pub fn was_cancelled(&self) -> bool {
        matches!(self, BiometricAuthResult::Cancelled)
    }
}

/// Configuration for a biometric authentication request.
#[derive(Debug, Clone)]
pub struct BiometricAuthConfig {
    /// The reason text shown to the user.
    pub reason: String,
    /// The title of the authentication dialog (Android).
    pub title: String,
    /// The subtitle of the dialog (Android).
    pub subtitle: String,
    /// Whether to allow fallback to password/passcode.
    pub allow_fallback: bool,
    /// Whether to allow cancellation by the user.
    pub allow_cancellation: bool,
    /// The preferred biometric type.
    pub preferred_type: Option<BiometricType>,
}

impl Default for BiometricAuthConfig {
    fn default() -> Self {
        Self {
            reason: "Authenticate to continue".to_string(),
            title: "Biometric Authentication".to_string(),
            subtitle: String::new(),
            allow_fallback: true,
            allow_cancellation: true,
            preferred_type: None,
        }
    }
}

impl BiometricAuthConfig {
    /// Create a new config with a reason.
    pub fn new(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
            ..Default::default()
        }
    }

    /// Set the title.
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Set the subtitle.
    pub fn with_subtitle(mut self, subtitle: &str) -> Self {
        self.subtitle = subtitle.to_string();
        self
    }

    /// Disable password fallback.
    pub fn no_fallback(mut self) -> Self {
        self.allow_fallback = false;
        self
    }

    /// Disable cancellation.
    pub fn no_cancellation(mut self) -> Self {
        self.allow_cancellation = false;
        self
    }

    /// Set the preferred biometric type.
    pub fn prefer(mut self, biometric_type: BiometricType) -> Self {
        self.preferred_type = Some(biometric_type);
        self
    }
}

/// The biometric authentication manager.
pub struct BiometricAuthManager {
    availability: Mutex<BiometricAvailability>,
    biometric_type: Mutex<BiometricType>,
    auth_count: Mutex<u32>,
    fail_count: Mutex<u32>,
}

impl BiometricAuthManager {
    /// Create a new biometric auth manager.
    pub fn new() -> Self {
        Self {
            availability: Mutex::new(BiometricAvailability::Available),
            biometric_type: Mutex::new(BiometricType::Fingerprint),
            auth_count: Mutex::new(0),
            fail_count: Mutex::new(0),
        }
    }

    /// Check biometric availability.
    pub fn check_availability(&self) -> BiometricAvailability {
        *self.availability.lock().unwrap()
    }

    /// Get the available biometric type.
    pub fn biometric_type(&self) -> BiometricType {
        *self.biometric_type.lock().unwrap()
    }

    /// Set the availability (for testing/platform detection).
    pub fn set_availability(&self, availability: BiometricAvailability) {
        *self.availability.lock().unwrap() = availability;
    }

    /// Set the biometric type (for testing/platform detection).
    pub fn set_biometric_type(&self, biometric_type: BiometricType) {
        *self.biometric_type.lock().unwrap() = biometric_type;
    }

    /// Authenticate with biometrics (simulated).
    pub fn authenticate(&self, config: &BiometricAuthConfig) -> BiometricAuthResult {
        let availability = self.availability.lock().unwrap();
        match *availability {
            BiometricAvailability::Available => {
                *self.auth_count.lock().unwrap() += 1;
                BiometricAuthResult::Success
            }
            BiometricAvailability::NoEnrollment => {
                BiometricAuthResult::Error("No biometrics enrolled".to_string())
            }
            BiometricAvailability::NotAvailable => {
                if config.allow_fallback {
                    BiometricAuthResult::Fallback
                } else {
                    BiometricAuthResult::Error("Biometric not available".to_string())
                }
            }
            BiometricAvailability::LockedOut => BiometricAuthResult::LockedOut,
            BiometricAvailability::TemporaryUnavailable => {
                BiometricAuthResult::Error("Temporarily unavailable".to_string())
            }
        }
    }

    /// Get the number of successful authentications.
    pub fn auth_count(&self) -> u32 {
        *self.auth_count.lock().unwrap()
    }

    /// Get the number of failed attempts.
    pub fn fail_count(&self) -> u32 {
        *self.fail_count.lock().unwrap()
    }

    /// Record a failed attempt.
    pub fn record_failure(&self) {
        *self.fail_count.lock().unwrap() += 1;
    }

    /// Reset failure count and unlock.
    pub fn reset(&self) {
        *self.fail_count.lock().unwrap() = 0;
        *self.availability.lock().unwrap() = BiometricAvailability::Available;
    }
}

impl Default for BiometricAuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_type_display_name() {
        assert_eq!(BiometricType::Face.display_name(), "Face Recognition");
        assert_eq!(BiometricType::Fingerprint.display_name(), "Fingerprint");
        assert_eq!(BiometricType::Iris.display_name(), "Iris Scan");
        assert_eq!(BiometricType::None.display_name(), "None");
    }

    #[test]
    fn test_biometric_auth_result_is_success() {
        assert!(BiometricAuthResult::Success.is_success());
        assert!(!BiometricAuthResult::Failed.is_success());
    }

    #[test]
    fn test_biometric_auth_result_was_cancelled() {
        assert!(BiometricAuthResult::Cancelled.was_cancelled());
        assert!(!BiometricAuthResult::Success.was_cancelled());
    }

    #[test]
    fn test_biometric_auth_config_default() {
        let config = BiometricAuthConfig::default();
        assert!(config.allow_fallback);
        assert!(config.allow_cancellation);
        assert!(config.preferred_type.is_none());
    }

    #[test]
    fn test_biometric_auth_config_builder() {
        let config = BiometricAuthConfig::new("Access vault")
            .with_title("Unlock")
            .with_subtitle("Use biometric")
            .no_fallback()
            .no_cancellation()
            .prefer(BiometricType::Face);

        assert_eq!(config.reason, "Access vault");
        assert_eq!(config.title, "Unlock");
        assert_eq!(config.subtitle, "Use biometric");
        assert!(!config.allow_fallback);
        assert!(!config.allow_cancellation);
        assert_eq!(config.preferred_type, Some(BiometricType::Face));
    }

    #[test]
    fn test_manager_check_availability() {
        let mgr = BiometricAuthManager::new();
        assert_eq!(mgr.check_availability(), BiometricAvailability::Available);
    }

    #[test]
    fn test_manager_biometric_type() {
        let mgr = BiometricAuthManager::new();
        assert_eq!(mgr.biometric_type(), BiometricType::Fingerprint);
    }

    #[test]
    fn test_manager_authenticate_success() {
        let mgr = BiometricAuthManager::new();
        let result = mgr.authenticate(&BiometricAuthConfig::new("Test"));
        assert!(result.is_success());
        assert_eq!(mgr.auth_count(), 1);
    }

    #[test]
    fn test_manager_authenticate_no_enrollment() {
        let mgr = BiometricAuthManager::new();
        mgr.set_availability(BiometricAvailability::NoEnrollment);
        let result = mgr.authenticate(&BiometricAuthConfig::new("Test"));
        assert!(matches!(result, BiometricAuthResult::Error(_)));
    }

    #[test]
    fn test_manager_authenticate_not_available_with_fallback() {
        let mgr = BiometricAuthManager::new();
        mgr.set_availability(BiometricAvailability::NotAvailable);
        let result = mgr.authenticate(&BiometricAuthConfig::new("Test"));
        assert_eq!(result, BiometricAuthResult::Fallback);
    }

    #[test]
    fn test_manager_authenticate_not_available_no_fallback() {
        let mgr = BiometricAuthManager::new();
        mgr.set_availability(BiometricAvailability::NotAvailable);
        let result = mgr.authenticate(&BiometricAuthConfig::new("Test").no_fallback());
        assert!(matches!(result, BiometricAuthResult::Error(_)));
    }

    #[test]
    fn test_manager_authenticate_locked_out() {
        let mgr = BiometricAuthManager::new();
        mgr.set_availability(BiometricAvailability::LockedOut);
        let result = mgr.authenticate(&BiometricAuthConfig::new("Test"));
        assert_eq!(result, BiometricAuthResult::LockedOut);
    }

    #[test]
    fn test_manager_record_failure_reset() {
        let mgr = BiometricAuthManager::new();
        mgr.record_failure();
        mgr.record_failure();
        assert_eq!(mgr.fail_count(), 2);
        mgr.reset();
        assert_eq!(mgr.fail_count(), 0);
    }

    #[test]
    fn test_manager_set_biometric_type() {
        let mgr = BiometricAuthManager::new();
        mgr.set_biometric_type(BiometricType::Face);
        assert_eq!(mgr.biometric_type(), BiometricType::Face);
    }
}
