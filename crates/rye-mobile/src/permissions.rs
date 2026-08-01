//! Goal 208: Native permissions manager.
//!
//! Unified `request_permission(Permission::Camera)` API with reactive status.

use std::collections::HashMap;
use std::sync::Mutex;

use rye_signals::Signal;

/// A permission that can be requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Camera access.
    Camera,
    /// Microphone access.
    Microphone,
    /// Location access.
    Location,
    /// Contacts access.
    Contacts,
    /// Photo gallery access.
    Photos,
    /// Push notification permission.
    Notifications,
    /// Biometric authentication.
    Biometric,
    /// Calendar access.
    Calendar,
    /// Reminders access.
    Reminders,
    /// Motion sensors.
    Motion,
    /// Bluetooth.
    Bluetooth,
    /// Local network.
    LocalNetwork,
}

impl Permission {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Permission::Camera => "Camera",
            Permission::Microphone => "Microphone",
            Permission::Location => "Location",
            Permission::Contacts => "Contacts",
            Permission::Photos => "Photos",
            Permission::Notifications => "Notifications",
            Permission::Biometric => "Biometric",
            Permission::Calendar => "Calendar",
            Permission::Reminders => "Reminders",
            Permission::Motion => "Motion & Fitness",
            Permission::Bluetooth => "Bluetooth",
            Permission::LocalNetwork => "Local Network",
        }
    }

    /// Get all permissions.
    pub fn all() -> &'static [Permission] {
        &[
            Permission::Camera,
            Permission::Microphone,
            Permission::Location,
            Permission::Contacts,
            Permission::Photos,
            Permission::Notifications,
            Permission::Biometric,
            Permission::Calendar,
            Permission::Reminders,
            Permission::Motion,
            Permission::Bluetooth,
            Permission::LocalNetwork,
        ]
    }
}

/// The state of a permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    /// Permission has not been requested yet.
    NotDetermined,
    /// Permission has been granted.
    Granted,
    /// Permission has been denied.
    Denied,
    /// Permission is restricted (parental controls, etc.).
    Restricted,
    /// Permission is limited (e.g. "Selected Photos" on iOS).
    Limited,
    /// Permission is not supported on this platform.
    NotSupported,
}

impl PermissionState {
    /// Check if the permission is granted (or limited).
    pub fn is_granted(&self) -> bool {
        matches!(self, PermissionState::Granted | PermissionState::Limited)
    }

    /// Check if the permission can be requested.
    pub fn can_request(&self) -> bool {
        matches!(self, PermissionState::NotDetermined)
    }

    /// Check if the permission was denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, PermissionState::Denied | PermissionState::Restricted)
    }
}

/// The result of a permission request.
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionRequestResult {
    /// Permission was granted.
    Granted,
    /// Permission was denied.
    Denied,
    /// Permission is limited.
    Limited,
    /// Permission is not supported.
    NotSupported,
    /// An error occurred.
    Error(String),
}

impl PermissionRequestResult {
    /// Convert to a PermissionState.
    pub fn to_state(&self) -> PermissionState {
        match self {
            PermissionRequestResult::Granted => PermissionState::Granted,
            PermissionRequestResult::Denied => PermissionState::Denied,
            PermissionRequestResult::Limited => PermissionState::Limited,
            PermissionRequestResult::NotSupported => PermissionState::NotSupported,
            PermissionRequestResult::Error(_) => PermissionState::NotDetermined,
        }
    }
}

/// The permissions manager — handles permission requests and status tracking.
pub struct PermissionsManager {
    states: Mutex<HashMap<Permission, PermissionState>>,
    reactive_states: Mutex<HashMap<Permission, Signal<PermissionState>>>,
}

impl PermissionsManager {
    /// Create a new permissions manager.
    pub fn new() -> Self {
        let mut states = HashMap::new();
        for perm in Permission::all() {
            states.insert(*perm, PermissionState::NotDetermined);
        }

        Self {
            states: Mutex::new(states),
            reactive_states: Mutex::new(HashMap::new()),
        }
    }

    /// Get the current state of a permission.
    pub fn get_state(&self, permission: Permission) -> PermissionState {
        *self.states.lock().unwrap().get(&permission).unwrap_or(&PermissionState::NotDetermined)
    }

    /// Get a reactive signal for a permission state.
    pub fn get_reactive_state(&self, permission: Permission) -> Signal<PermissionState> {
        let mut reactive = self.reactive_states.lock().unwrap();
        if let Some(signal) = reactive.get(&permission) {
            return signal.clone();
        }
        let state = self.get_state(permission);
        let signal = Signal::new(state);
        reactive.insert(permission, signal.clone());
        signal
    }

    /// Request a permission (simulated — grants by default).
    pub fn request(&self, permission: Permission) -> PermissionRequestResult {
        let mut states = self.states.lock().unwrap();
        let current = *states.get(&permission).unwrap_or(&PermissionState::NotDetermined);

        if current == PermissionState::NotSupported {
            return PermissionRequestResult::NotSupported;
        }

        // Simulate granting the permission
        states.insert(permission, PermissionState::Granted);

        // Update reactive signal
        let reactive = self.reactive_states.lock().unwrap();
        if let Some(signal) = reactive.get(&permission) {
            signal.set(PermissionState::Granted);
        }

        PermissionRequestResult::Granted
    }

    /// Request a permission with a specific result (for testing).
    pub fn request_with_result(&self, permission: Permission, result: PermissionRequestResult) {
        let new_state = result.to_state();
        self.states.lock().unwrap().insert(permission, new_state);

        let reactive = self.reactive_states.lock().unwrap();
        if let Some(signal) = reactive.get(&permission) {
            signal.set(new_state);
        }
    }

    /// Set the state of a permission directly (for platform detection).
    pub fn set_state(&self, permission: Permission, state: PermissionState) {
        self.states.lock().unwrap().insert(permission, state);

        let reactive = self.reactive_states.lock().unwrap();
        if let Some(signal) = reactive.get(&permission) {
            signal.set(state);
        }
    }

    /// Check if a permission is granted.
    pub fn is_granted(&self, permission: Permission) -> bool {
        self.get_state(permission).is_granted()
    }

    /// Check if a permission can be requested.
    pub fn can_request(&self, permission: Permission) -> bool {
        self.get_state(permission).can_request()
    }

    /// Open the app settings page (simulated).
    pub fn open_settings(&self) -> bool {
        true
    }

    /// Get all permissions and their states.
    pub fn all_states(&self) -> Vec<(Permission, PermissionState)> {
        self.states
            .lock()
            .unwrap()
            .iter()
            .map(|(p, s)| (*p, *s))
            .collect()
    }

    /// Get all granted permissions.
    pub fn granted_permissions(&self) -> Vec<Permission> {
        self.states
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, s)| s.is_granted())
            .map(|(p, _)| *p)
            .collect()
    }

    /// Get all denied permissions.
    pub fn denied_permissions(&self) -> Vec<Permission> {
        self.states
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, s)| s.is_denied())
            .map(|(p, _)| *p)
            .collect()
    }
}

impl Default for PermissionsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_display_name() {
        assert_eq!(Permission::Camera.display_name(), "Camera");
        assert_eq!(Permission::Location.display_name(), "Location");
        assert_eq!(Permission::Notifications.display_name(), "Notifications");
    }

    #[test]
    fn test_permission_all() {
        assert_eq!(Permission::all().len(), 12);
    }

    #[test]
    fn test_permission_state_is_granted() {
        assert!(PermissionState::Granted.is_granted());
        assert!(PermissionState::Limited.is_granted());
        assert!(!PermissionState::Denied.is_granted());
        assert!(!PermissionState::NotDetermined.is_granted());
    }

    #[test]
    fn test_permission_state_can_request() {
        assert!(PermissionState::NotDetermined.can_request());
        assert!(!PermissionState::Granted.can_request());
        assert!(!PermissionState::Denied.can_request());
    }

    #[test]
    fn test_permission_state_is_denied() {
        assert!(PermissionState::Denied.is_denied());
        assert!(PermissionState::Restricted.is_denied());
        assert!(!PermissionState::Granted.is_denied());
    }

    #[test]
    fn test_permission_request_result_to_state() {
        assert_eq!(PermissionRequestResult::Granted.to_state(), PermissionState::Granted);
        assert_eq!(PermissionRequestResult::Denied.to_state(), PermissionState::Denied);
        assert_eq!(PermissionRequestResult::Limited.to_state(), PermissionState::Limited);
        assert_eq!(PermissionRequestResult::NotSupported.to_state(), PermissionState::NotSupported);
        assert_eq!(PermissionRequestResult::Error("e".to_string()).to_state(), PermissionState::NotDetermined);
    }

    #[test]
    fn test_manager_get_state_default() {
        let mgr = PermissionsManager::new();
        assert_eq!(mgr.get_state(Permission::Camera), PermissionState::NotDetermined);
    }

    #[test]
    fn test_manager_request() {
        let mgr = PermissionsManager::new();
        let result = mgr.request(Permission::Camera);
        assert_eq!(result, PermissionRequestResult::Granted);
        assert_eq!(mgr.get_state(Permission::Camera), PermissionState::Granted);
    }

    #[test]
    fn test_manager_request_not_supported() {
        let mgr = PermissionsManager::new();
        mgr.set_state(Permission::Bluetooth, PermissionState::NotSupported);
        let result = mgr.request(Permission::Bluetooth);
        assert_eq!(result, PermissionRequestResult::NotSupported);
    }

    #[test]
    fn test_manager_is_granted() {
        let mgr = PermissionsManager::new();
        assert!(!mgr.is_granted(Permission::Camera));
        mgr.request(Permission::Camera);
        assert!(mgr.is_granted(Permission::Camera));
    }

    #[test]
    fn test_manager_can_request() {
        let mgr = PermissionsManager::new();
        assert!(mgr.can_request(Permission::Location));
        mgr.request(Permission::Location);
        assert!(!mgr.can_request(Permission::Location));
    }

    #[test]
    fn test_manager_set_state() {
        let mgr = PermissionsManager::new();
        mgr.set_state(Permission::Photos, PermissionState::Limited);
        assert_eq!(mgr.get_state(Permission::Photos), PermissionState::Limited);
        assert!(mgr.is_granted(Permission::Photos));
    }

    #[test]
    fn test_manager_request_with_result() {
        let mgr = PermissionsManager::new();
        mgr.request_with_result(Permission::Microphone, PermissionRequestResult::Denied);
        assert_eq!(mgr.get_state(Permission::Microphone), PermissionState::Denied);
    }

    #[test]
    fn test_manager_reactive_state() {
        let mgr = PermissionsManager::new();
        let signal = mgr.get_reactive_state(Permission::Camera);
        assert_eq!(signal.get(), PermissionState::NotDetermined);

        mgr.request(Permission::Camera);
        assert_eq!(signal.get(), PermissionState::Granted);
    }

    #[test]
    fn test_manager_reactive_state_cached() {
        let mgr = PermissionsManager::new();
        let s1 = mgr.get_reactive_state(Permission::Camera);
        let s2 = mgr.get_reactive_state(Permission::Camera);
        // Both should be the same signal (same value)
        assert_eq!(s1.get(), s2.get());
    }

    #[test]
    fn test_manager_all_states() {
        let mgr = PermissionsManager::new();
        let states = mgr.all_states();
        assert_eq!(states.len(), 12);
    }

    #[test]
    fn test_manager_granted_permissions() {
        let mgr = PermissionsManager::new();
        mgr.request(Permission::Camera);
        mgr.request(Permission::Location);
        let granted = mgr.granted_permissions();
        assert!(granted.contains(&Permission::Camera));
        assert!(granted.contains(&Permission::Location));
        assert_eq!(granted.len(), 2);
    }

    #[test]
    fn test_manager_denied_permissions() {
        let mgr = PermissionsManager::new();
        mgr.request_with_result(Permission::Microphone, PermissionRequestResult::Denied);
        let denied = mgr.denied_permissions();
        assert!(denied.contains(&Permission::Microphone));
    }

    #[test]
    fn test_manager_open_settings() {
        let mgr = PermissionsManager::new();
        assert!(mgr.open_settings());
    }
}
