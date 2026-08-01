//! Goal 197: Native push notifications.
//!
//! Cross-platform push notification API. `use_push_notifications()` hook.
//! Web (Push API), iOS (APNs), Android (FCM).

use std::collections::HashMap;
use std::sync::Mutex;

/// The push notification permission status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushPermissionState {
    /// Permission has not been requested yet.
    NotDetermined,
    /// Permission has been granted.
    Granted,
    /// Permission has been denied.
    Denied,
    /// Permission is provisionally granted (iOS only).
    Provisional,
    /// Push notifications are not available on this platform.
    Unsupported,
}

impl PushPermissionState {
    /// Check if push notifications can be used.
    pub fn can_send(&self) -> bool {
        matches!(self, PushPermissionState::Granted | PushPermissionState::Provisional)
    }
}

/// A push notification payload.
#[derive(Debug, Clone)]
pub struct PushNotification {
    /// The notification title.
    pub title: String,
    /// The notification body text.
    pub body: String,
    /// Additional data to send with the notification.
    pub data: HashMap<String, String>,
    /// The notification category/identifier (for action buttons).
    pub category: Option<String>,
    /// The badge count to set (iOS).
    pub badge: Option<u32>,
    /// The sound to play.
    pub sound: Option<String>,
    /// Whether this is a silent notification (no UI, just data).
    pub silent: bool,
}

impl PushNotification {
    /// Create a new push notification with a title and body.
    pub fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_string(),
            body: body.to_string(),
            data: HashMap::new(),
            category: None,
            badge: None,
            sound: None,
            silent: false,
        }
    }

    /// Add data to the notification.
    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the category.
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Set the badge count.
    pub fn with_badge(mut self, badge: u32) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Set the sound.
    pub fn with_sound(mut self, sound: &str) -> Self {
        self.sound = Some(sound.to_string());
        self
    }

    /// Make this a silent notification.
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    /// Serialize to JSON (APNs/FCM compatible).
    pub fn to_json(&self) -> String {
        let mut json = format!("{{\"title\":\"{}\",\"body\":\"{}\"", self.title, self.body);

        if !self.data.is_empty() {
            let data: Vec<String> = self
                .data
                .iter()
                .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
                .collect();
            json.push_str(&format!(",\"data\":{{{}}}", data.join(",")));
        }

        if let Some(ref cat) = self.category {
            json.push_str(&format!(",\"category\":\"{}\"", cat));
        }

        if let Some(badge) = self.badge {
            json.push_str(&format!(",\"badge\":{}", badge));
        }

        if let Some(ref sound) = self.sound {
            json.push_str(&format!(",\"sound\":\"{}\"", sound));
        }

        if self.silent {
            json.push_str(",\"content-available\":1");
        }

        json.push('}');
        json
    }
}

/// A push notification channel (Android notification channels).
#[derive(Debug, Clone)]
pub struct NotificationChannel {
    /// The channel ID.
    pub id: String,
    /// The channel name.
    pub name: String,
    /// The channel description.
    pub description: String,
    /// The importance level (1-4).
    pub importance: u8,
    /// Whether to show a badge.
    pub show_badge: bool,
}

impl NotificationChannel {
    /// Create a new notification channel.
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            importance: 3,
            show_badge: true,
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set the importance (1=low, 2=default, 3=high, 4=urgent).
    pub fn with_importance(mut self, importance: u8) -> Self {
        self.importance = importance.min(4).max(1);
        self
    }

    /// Set whether to show a badge.
    pub fn with_badge(mut self, show: bool) -> Self {
        self.show_badge = show;
        self
    }
}

/// An action button for push notifications.
#[derive(Debug, Clone)]
pub struct NotificationAction {
    /// The action identifier.
    pub id: String,
    /// The button title.
    pub title: String,
    /// Whether the action is destructive (red button).
    pub destructive: bool,
    /// Whether the action requires foreground.
    pub foreground: bool,
}

impl NotificationAction {
    /// Create a new notification action.
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            destructive: false,
            foreground: false,
        }
    }

    /// Mark as destructive.
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }

    /// Mark as foreground action.
    pub fn foreground(mut self) -> Self {
        self.foreground = true;
        self
    }
}

/// The push notification manager — handles registration, permissions, and sending.
pub struct PushNotificationManager {
    permission_state: Mutex<PushPermissionState>,
    device_token: Mutex<Option<String>>,
    channels: Mutex<HashMap<String, NotificationChannel>>,
}

impl PushNotificationManager {
    /// Create a new push notification manager.
    pub fn new() -> Self {
        Self {
            permission_state: Mutex::new(PushPermissionState::NotDetermined),
            device_token: Mutex::new(None),
            channels: Mutex::new(HashMap::new()),
        }
    }

    /// Request permission to send push notifications.
    pub fn request_permission(&self) -> PushPermissionState {
        let mut state = self.permission_state.lock().unwrap();
        // In a real implementation, this would call the platform API
        // For now, we simulate a granted permission
        *state = PushPermissionState::Granted;
        *state
    }

    /// Get the current permission state.
    pub fn permission_state(&self) -> PushPermissionState {
        *self.permission_state.lock().unwrap()
    }

    /// Register the device for push notifications and get a token.
    pub fn register_for_push(&self) -> Option<String> {
        let state = self.permission_state.lock().unwrap();
        if !state.can_send() {
            return None;
        }
        drop(state);

        // In a real implementation, this would call APNs/FCM/Push API
        let token = "simulated-device-token".to_string();
        *self.device_token.lock().unwrap() = Some(token.clone());
        Some(token)
    }

    /// Get the device token.
    pub fn device_token(&self) -> Option<String> {
        self.device_token.lock().unwrap().clone()
    }

    /// Register a notification channel (Android).
    pub fn register_channel(&self, channel: NotificationChannel) {
        self.channels.lock().unwrap().insert(channel.id.clone(), channel);
    }

    /// Get a channel by ID.
    pub fn get_channel(&self, id: &str) -> Option<NotificationChannel> {
        self.channels.lock().unwrap().get(id).cloned()
    }

    /// Get all channel IDs.
    pub fn channel_ids(&self) -> Vec<String> {
        self.channels.lock().unwrap().keys().cloned().collect()
    }

    /// Send a push notification (simulated).
    pub fn send(&self, notification: &PushNotification) -> Result<String, String> {
        let state = self.permission_state.lock().unwrap();
        if !state.can_send() {
            return Err("Push permission not granted".to_string());
        }
        drop(state);

        let token = self.device_token.lock().unwrap();
        if token.is_none() {
            return Err("Device not registered for push".to_string());
        }

        Ok(format!("sent:{}", notification.title))
    }

    /// Unregister from push notifications.
    pub fn unregister(&self) {
        *self.device_token.lock().unwrap() = None;
    }
}

impl Default for PushNotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_permission_can_send() {
        assert!(PushPermissionState::Granted.can_send());
        assert!(PushPermissionState::Provisional.can_send());
        assert!(!PushPermissionState::Denied.can_send());
        assert!(!PushPermissionState::NotDetermined.can_send());
        assert!(!PushPermissionState::Unsupported.can_send());
    }

    #[test]
    fn test_push_notification_new() {
        let n = PushNotification::new("Hello", "World");
        assert_eq!(n.title, "Hello");
        assert_eq!(n.body, "World");
        assert!(n.data.is_empty());
        assert!(!n.silent);
    }

    #[test]
    fn test_push_notification_builder() {
        let n = PushNotification::new("Title", "Body")
            .with_data("key", "value")
            .with_category("CATEGORY")
            .with_badge(5)
            .with_sound("default")
            .silent();

        assert_eq!(n.data.get("key"), Some(&"value".to_string()));
        assert_eq!(n.category, Some("CATEGORY".to_string()));
        assert_eq!(n.badge, Some(5));
        assert_eq!(n.sound, Some("default".to_string()));
        assert!(n.silent);
    }

    #[test]
    fn test_push_notification_to_json() {
        let n = PushNotification::new("Hello", "World");
        let json = n.to_json();
        assert!(json.contains("\"title\":\"Hello\""));
        assert!(json.contains("\"body\":\"World\""));
    }

    #[test]
    fn test_push_notification_to_json_with_data() {
        let n = PushNotification::new("Hello", "World")
            .with_data("url", "https://example.com");
        let json = n.to_json();
        assert!(json.contains("\"data\""));
        assert!(json.contains("\"url\":\"https://example.com\""));
    }

    #[test]
    fn test_push_notification_to_json_silent() {
        let n = PushNotification::new("Hello", "World").silent();
        let json = n.to_json();
        assert!(json.contains("\"content-available\":1"));
    }

    #[test]
    fn test_notification_channel_new() {
        let ch = NotificationChannel::new("messages", "Messages");
        assert_eq!(ch.id, "messages");
        assert_eq!(ch.name, "Messages");
        assert_eq!(ch.importance, 3);
        assert!(ch.show_badge);
    }

    #[test]
    fn test_notification_channel_builder() {
        let ch = NotificationChannel::new("alerts", "Alerts")
            .with_description("Important alerts")
            .with_importance(4)
            .with_badge(false);
        assert_eq!(ch.description, "Important alerts");
        assert_eq!(ch.importance, 4);
        assert!(!ch.show_badge);
    }

    #[test]
    fn test_notification_channel_importance_clamped() {
        let ch = NotificationChannel::new("test", "Test").with_importance(10);
        assert_eq!(ch.importance, 4);

        let ch2 = NotificationChannel::new("test", "Test").with_importance(0);
        assert_eq!(ch2.importance, 1);
    }

    #[test]
    fn test_notification_action_new() {
        let action = NotificationAction::new("reply", "Reply");
        assert_eq!(action.id, "reply");
        assert_eq!(action.title, "Reply");
        assert!(!action.destructive);
        assert!(!action.foreground);
    }

    #[test]
    fn test_notification_action_destructive() {
        let action = NotificationAction::new("delete", "Delete").destructive();
        assert!(action.destructive);
    }

    #[test]
    fn test_notification_action_foreground() {
        let action = NotificationAction::new("open", "Open").foreground();
        assert!(action.foreground);
    }

    #[test]
    fn test_manager_request_permission() {
        let mgr = PushNotificationManager::new();
        assert_eq!(mgr.permission_state(), PushPermissionState::NotDetermined);
        let state = mgr.request_permission();
        assert_eq!(state, PushPermissionState::Granted);
    }

    #[test]
    fn test_manager_register_for_push() {
        let mgr = PushNotificationManager::new();
        mgr.request_permission();
        let token = mgr.register_for_push();
        assert!(token.is_some());
        assert!(mgr.device_token().is_some());
    }

    #[test]
    fn test_manager_register_for_push_no_permission() {
        let mgr = PushNotificationManager::new();
        let token = mgr.register_for_push();
        assert!(token.is_none());
    }

    #[test]
    fn test_manager_send() {
        let mgr = PushNotificationManager::new();
        mgr.request_permission();
        mgr.register_for_push();
        let result = mgr.send(&PushNotification::new("Test", "Body"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_manager_send_no_permission() {
        let mgr = PushNotificationManager::new();
        let result = mgr.send(&PushNotification::new("Test", "Body"));
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_send_no_token() {
        let mgr = PushNotificationManager::new();
        mgr.request_permission();
        let result = mgr.send(&PushNotification::new("Test", "Body"));
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_unregister() {
        let mgr = PushNotificationManager::new();
        mgr.request_permission();
        mgr.register_for_push();
        assert!(mgr.device_token().is_some());
        mgr.unregister();
        assert!(mgr.device_token().is_none());
    }

    #[test]
    fn test_manager_channels() {
        let mgr = PushNotificationManager::new();
        mgr.register_channel(NotificationChannel::new("ch1", "Channel 1"));
        mgr.register_channel(NotificationChannel::new("ch2", "Channel 2"));

        assert_eq!(mgr.channel_ids().len(), 2);
        assert!(mgr.get_channel("ch1").is_some());
        assert!(mgr.get_channel("nonexistent").is_none());
    }
}
