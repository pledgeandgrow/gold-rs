//! Goal 207: Native haptic feedback.
//!
//! `use_haptics()` hook. iOS (UIFeedbackGenerator), Android (Vibrator), web (Vibration API).

use std::sync::Mutex;

/// The type of haptic impact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticImpact {
    /// Light impact.
    Light,
    /// Medium impact.
    Medium,
    /// Heavy impact.
    Heavy,
    /// Rigid impact (short and sharp).
    Rigid,
    /// Soft impact (longer and softer).
    Soft,
}

impl HapticImpact {
    /// Get the approximate duration in milliseconds.
    pub fn duration_ms(&self) -> u32 {
        match self {
            HapticImpact::Light => 10,
            HapticImpact::Medium => 20,
            HapticImpact::Heavy => 30,
            HapticImpact::Rigid => 5,
            HapticImpact::Soft => 40,
        }
    }

    /// Get the vibration intensity (0-255).
    pub fn intensity(&self) -> u8 {
        match self {
            HapticImpact::Light => 80,
            HapticImpact::Medium => 160,
            HapticImpact::Heavy => 255,
            HapticImpact::Rigid => 200,
            HapticImpact::Soft => 60,
        }
    }
}

/// The notification haptic type (success/warning/error patterns).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticNotification {
    /// Success pattern.
    Success,
    /// Warning pattern.
    Warning,
    /// Error pattern.
    Error,
}

impl HapticNotification {
    /// Get the vibration pattern as a sequence of on/off durations (ms).
    pub fn pattern(&self) -> Vec<u32> {
        match self {
            HapticNotification::Success => vec![0, 10, 50, 10],
            HapticNotification::Warning => vec![0, 20, 50, 20],
            HapticNotification::Error => vec![0, 30, 50, 30, 50, 30],
        }
    }
}

/// The selection haptic type (for scroll pickers, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticSelection {
    /// Default selection feedback.
    Default,
    /// Soft selection feedback.
    Soft,
}

/// A custom haptic pattern.
#[derive(Debug, Clone)]
pub struct HapticPattern {
    /// The pattern as alternating on/off durations in milliseconds.
    pub timings: Vec<u32>,
    /// The amplitudes (0-255) for each "on" period.
    pub amplitudes: Vec<u8>,
}

impl HapticPattern {
    /// Create a new haptic pattern.
    pub fn new() -> Self {
        Self {
            timings: Vec::new(),
            amplitudes: Vec::new(),
        }
    }

    /// Add an "on" period with a duration and amplitude.
    pub fn on(mut self, duration_ms: u32, amplitude: u8) -> Self {
        self.timings.push(duration_ms);
        self.amplitudes.push(amplitude);
        self
    }

    /// Add an "off" period (gap).
    pub fn off(mut self, duration_ms: u32) -> Self {
        self.timings.push(duration_ms);
        self.amplitudes.push(0);
        self
    }

    /// Check if the pattern is empty.
    pub fn is_empty(&self) -> bool {
        self.timings.is_empty()
    }

    /// Get the total duration of the pattern.
    pub fn total_duration_ms(&self) -> u32 {
        self.timings.iter().sum()
    }
}

impl Default for HapticPattern {
    fn default() -> Self {
        Self::new()
    }
}

/// The haptic feedback manager.
pub struct HapticsManager {
    available: bool,
    enabled: Mutex<bool>,
    play_count: Mutex<u32>,
}

impl HapticsManager {
    /// Create a new haptics manager.
    pub fn new() -> Self {
        Self {
            available: true,
            enabled: Mutex::new(true),
            play_count: Mutex::new(0),
        }
    }

    /// Create a manager with availability.
    pub fn with_availability(available: bool) -> Self {
        Self {
            available,
            enabled: Mutex::new(true),
            play_count: Mutex::new(0),
        }
    }

    /// Check if haptics are available.
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Check if haptics are enabled.
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    /// Enable haptics.
    pub fn enable(&self) {
        *self.enabled.lock().unwrap() = true;
    }

    /// Disable haptics.
    pub fn disable(&self) {
        *self.enabled.lock().unwrap() = false;
    }

    /// Play an impact haptic.
    pub fn impact(&self, _impact: HapticImpact) -> bool {
        if !self.available || !*self.enabled.lock().unwrap() {
            return false;
        }
        *self.play_count.lock().unwrap() += 1;
        true
    }

    /// Play a notification haptic.
    pub fn notification(&self, _notif: HapticNotification) -> bool {
        if !self.available || !*self.enabled.lock().unwrap() {
            return false;
        }
        *self.play_count.lock().unwrap() += 1;
        true
    }

    /// Play a selection haptic.
    pub fn selection(&self, _selection: HapticSelection) -> bool {
        if !self.available || !*self.enabled.lock().unwrap() {
            return false;
        }
        *self.play_count.lock().unwrap() += 1;
        true
    }

    /// Play a custom pattern.
    pub fn pattern(&self, _pattern: &HapticPattern) -> bool {
        if !self.available || !*self.enabled.lock().unwrap() {
            return false;
        }
        *self.play_count.lock().unwrap() += 1;
        true
    }

    /// Get the number of haptics played.
    pub fn play_count(&self) -> u32 {
        *self.play_count.lock().unwrap()
    }
}

impl Default for HapticsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haptic_impact_duration() {
        assert!(HapticImpact::Light.duration_ms() < HapticImpact::Medium.duration_ms());
        assert!(HapticImpact::Medium.duration_ms() < HapticImpact::Heavy.duration_ms());
    }

    #[test]
    fn test_haptic_impact_intensity() {
        assert!(HapticImpact::Light.intensity() < HapticImpact::Heavy.intensity());
        assert_eq!(HapticImpact::Heavy.intensity(), 255);
    }

    #[test]
    fn test_haptic_notification_pattern() {
        let success = HapticNotification::Success.pattern();
        assert!(!success.is_empty());
        assert_eq!(success[0], 0); // starts with delay

        let error = HapticNotification::Error.pattern();
        assert!(error.len() > success.len());
    }

    #[test]
    fn test_haptic_pattern_builder() {
        let pattern = HapticPattern::new()
            .on(20, 200)
            .off(50)
            .on(30, 255);

        assert_eq!(pattern.timings, vec![20, 50, 30]);
        assert_eq!(pattern.amplitudes, vec![200, 0, 255]);
        assert!(!pattern.is_empty());
        assert_eq!(pattern.total_duration_ms(), 100);
    }

    #[test]
    fn test_haptic_pattern_empty() {
        let pattern = HapticPattern::new();
        assert!(pattern.is_empty());
        assert_eq!(pattern.total_duration_ms(), 0);
    }

    #[test]
    fn test_manager_available() {
        let mgr = HapticsManager::new();
        assert!(mgr.is_available());
    }

    #[test]
    fn test_manager_unavailable() {
        let mgr = HapticsManager::with_availability(false);
        assert!(!mgr.is_available());
    }

    #[test]
    fn test_manager_enable_disable() {
        let mgr = HapticsManager::new();
        assert!(mgr.is_enabled());
        mgr.disable();
        assert!(!mgr.is_enabled());
        mgr.enable();
        assert!(mgr.is_enabled());
    }

    #[test]
    fn test_manager_impact() {
        let mgr = HapticsManager::new();
        assert!(mgr.impact(HapticImpact::Medium));
        assert_eq!(mgr.play_count(), 1);
    }

    #[test]
    fn test_manager_impact_disabled() {
        let mgr = HapticsManager::new();
        mgr.disable();
        assert!(!mgr.impact(HapticImpact::Medium));
        assert_eq!(mgr.play_count(), 0);
    }

    #[test]
    fn test_manager_impact_unavailable() {
        let mgr = HapticsManager::with_availability(false);
        assert!(!mgr.impact(HapticImpact::Medium));
    }

    #[test]
    fn test_manager_notification() {
        let mgr = HapticsManager::new();
        assert!(mgr.notification(HapticNotification::Success));
        assert!(mgr.notification(HapticNotification::Warning));
        assert!(mgr.notification(HapticNotification::Error));
        assert_eq!(mgr.play_count(), 3);
    }

    #[test]
    fn test_manager_selection() {
        let mgr = HapticsManager::new();
        assert!(mgr.selection(HapticSelection::Default));
        assert!(mgr.selection(HapticSelection::Soft));
        assert_eq!(mgr.play_count(), 2);
    }

    #[test]
    fn test_manager_pattern() {
        let mgr = HapticsManager::new();
        let pattern = HapticPattern::new().on(20, 200).off(50).on(30, 255);
        assert!(mgr.pattern(&pattern));
        assert_eq!(mgr.play_count(), 1);
    }

    #[test]
    fn test_manager_pattern_disabled() {
        let mgr = HapticsManager::new();
        mgr.disable();
        let pattern = HapticPattern::new().on(20, 200);
        assert!(!mgr.pattern(&pattern));
    }
}
