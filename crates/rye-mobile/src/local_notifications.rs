//! Goal 203: Native local notifications.
//!
//! Schedule local notifications without a server.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A local notification.
#[derive(Debug, Clone)]
pub struct LocalNotification {
    /// The notification ID.
    pub id: String,
    /// The notification title.
    pub title: String,
    /// The notification body.
    pub body: String,
    /// Additional data.
    pub data: HashMap<String, String>,
    /// The category/identifier for action buttons.
    pub category: Option<String>,
    /// The sound to play.
    pub sound: Option<String>,
    /// The badge count to set.
    pub badge: Option<u32>,
}

impl LocalNotification {
    /// Create a new local notification.
    pub fn new(id: &str, title: &str, body: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            data: HashMap::new(),
            category: None,
            sound: None,
            badge: None,
        }
    }

    /// Add data.
    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the category.
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Set the sound.
    pub fn with_sound(mut self, sound: &str) -> Self {
        self.sound = Some(sound.to_string());
        self
    }

    /// Set the badge.
    pub fn with_badge(mut self, badge: u32) -> Self {
        self.badge = Some(badge);
        self
    }
}

/// The trigger for a local notification.
#[derive(Debug, Clone)]
pub enum NotificationTrigger {
    /// Trigger after a delay (in seconds).
    TimeInterval(u64),
    /// Trigger at a specific date/time (ISO 8601 string).
    Calendar(String),
    /// Trigger at a specific time daily.
    Daily {
        /// The hour (0-23).
        hour: u8,
        /// The minute (0-59).
        minute: u8,
    },
    /// Trigger weekly on a specific day and time.
    Weekly {
        /// The day of week (1-7, 1=Sunday).
        day_of_week: u8,
        /// The hour (0-23).
        hour: u8,
        /// The minute (0-59).
        minute: u8,
    },
    /// Trigger when the app enters the foreground.
    OnAppForeground,
    /// Trigger immediately.
    Immediate,
}

impl NotificationTrigger {
    /// Create a time interval trigger.
    pub fn after(seconds: u64) -> Self {
        NotificationTrigger::TimeInterval(seconds)
    }

    /// Create an immediate trigger.
    pub fn now() -> Self {
        NotificationTrigger::Immediate
    }
}

/// The permission state for local notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPermissionState {
    /// Not determined.
    NotDetermined,
    /// Granted.
    Granted,
    /// Denied.
    Denied,
    /// Provisional (iOS).
    Provisional,
}

impl NotificationPermissionState {
    /// Check if notifications can be scheduled.
    pub fn can_schedule(&self) -> bool {
        matches!(
            self,
            NotificationPermissionState::Granted | NotificationPermissionState::Provisional
        )
    }
}

/// A scheduled notification (pending delivery).
#[derive(Debug)]
pub struct ScheduledNotification {
    /// The notification content.
    pub notification: LocalNotification,
    /// The trigger.
    pub trigger: NotificationTrigger,
    /// When the notification was scheduled.
    pub scheduled_at: Instant,
    /// Whether the notification has been delivered.
    pub delivered: bool,
}

/// The local notifications manager.
pub struct LocalNotificationsManager {
    permission: Mutex<NotificationPermissionState>,
    scheduled: Mutex<HashMap<String, ScheduledNotification>>,
    delivered: Mutex<Vec<LocalNotification>>,
}

impl LocalNotificationsManager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self {
            permission: Mutex::new(NotificationPermissionState::NotDetermined),
            scheduled: Mutex::new(HashMap::new()),
            delivered: Mutex::new(Vec::new()),
        }
    }

    /// Request permission.
    pub fn request_permission(&self) -> NotificationPermissionState {
        let mut state = self.permission.lock().unwrap();
        *state = NotificationPermissionState::Granted;
        *state
    }

    /// Get the current permission state.
    pub fn permission_state(&self) -> NotificationPermissionState {
        *self.permission.lock().unwrap()
    }

    /// Schedule a notification.
    pub fn schedule(
        &self,
        notification: LocalNotification,
        trigger: NotificationTrigger,
    ) -> Result<(), String> {
        let perm = self.permission.lock().unwrap();
        if !perm.can_schedule() {
            return Err("Notification permission not granted".to_string());
        }
        drop(perm);

        let id = notification.id.clone();
        let scheduled = ScheduledNotification {
            notification,
            trigger,
            scheduled_at: Instant::now(),
            delivered: false,
        };
        self.scheduled.lock().unwrap().insert(id, scheduled);
        Ok(())
    }

    /// Cancel a scheduled notification.
    pub fn cancel(&self, id: &str) -> bool {
        self.scheduled.lock().unwrap().remove(id).is_some()
    }

    /// Get all pending (scheduled, not delivered) notification IDs.
    pub fn pending_ids(&self) -> Vec<String> {
        self.scheduled
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, s)| !s.delivered)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get the number of pending notifications.
    pub fn pending_count(&self) -> usize {
        self.scheduled
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, s)| !s.delivered)
            .count()
    }

    /// Simulate delivering due notifications.
    pub fn deliver_due(&self) -> usize {
        let mut scheduled = self.scheduled.lock().unwrap();
        let mut delivered = self.delivered.lock().unwrap();
        let now = Instant::now();
        let mut count = 0;

        for sched in scheduled.values_mut() {
            if sched.delivered {
                continue;
            }

            let should_deliver = match &sched.trigger {
                NotificationTrigger::Immediate => true,
                NotificationTrigger::TimeInterval(secs) => {
                    now.duration_since(sched.scheduled_at) >= Duration::from_secs(*secs)
                }
                NotificationTrigger::OnAppForeground => false, // Can't simulate
                NotificationTrigger::Calendar(_) => false,     // Can't simulate
                NotificationTrigger::Daily { .. } => false,    // Can't simulate
                NotificationTrigger::Weekly { .. } => false,   // Can't simulate
            };

            if should_deliver {
                delivered.push(sched.notification.clone());
                sched.delivered = true;
                count += 1;
            }
        }

        count
    }

    /// Get all delivered notifications.
    pub fn delivered_notifications(&self) -> Vec<LocalNotification> {
        self.delivered.lock().unwrap().clone()
    }

    /// Clear all delivered notifications.
    pub fn clear_delivered(&self) {
        self.delivered.lock().unwrap().clear();
    }

    /// Cancel all pending notifications.
    pub fn cancel_all(&self) {
        self.scheduled.lock().unwrap().clear();
    }

    /// Get the delivered count.
    pub fn delivered_count(&self) -> usize {
        self.delivered.lock().unwrap().len()
    }
}

impl Default for LocalNotificationsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_notification_new() {
        let n = LocalNotification::new("1", "Title", "Body");
        assert_eq!(n.id, "1");
        assert_eq!(n.title, "Title");
        assert_eq!(n.body, "Body");
    }

    #[test]
    fn test_local_notification_builder() {
        let n = LocalNotification::new("1", "Title", "Body")
            .with_data("key", "value")
            .with_category("CATEGORY")
            .with_sound("default")
            .with_badge(3);

        assert_eq!(n.data.get("key"), Some(&"value".to_string()));
        assert_eq!(n.category, Some("CATEGORY".to_string()));
        assert_eq!(n.sound, Some("default".to_string()));
        assert_eq!(n.badge, Some(3));
    }

    #[test]
    fn test_trigger_after() {
        let trigger = NotificationTrigger::after(60);
        assert!(matches!(trigger, NotificationTrigger::TimeInterval(60)));
    }

    #[test]
    fn test_trigger_now() {
        let trigger = NotificationTrigger::now();
        assert!(matches!(trigger, NotificationTrigger::Immediate));
    }

    #[test]
    fn test_permission_can_schedule() {
        assert!(NotificationPermissionState::Granted.can_schedule());
        assert!(NotificationPermissionState::Provisional.can_schedule());
        assert!(!NotificationPermissionState::Denied.can_schedule());
        assert!(!NotificationPermissionState::NotDetermined.can_schedule());
    }

    #[test]
    fn test_manager_request_permission() {
        let mgr = LocalNotificationsManager::new();
        assert_eq!(
            mgr.permission_state(),
            NotificationPermissionState::NotDetermined
        );
        mgr.request_permission();
        assert_eq!(mgr.permission_state(), NotificationPermissionState::Granted);
    }

    #[test]
    fn test_manager_schedule_no_permission() {
        let mgr = LocalNotificationsManager::new();
        let result = mgr.schedule(
            LocalNotification::new("1", "Test", "Body"),
            NotificationTrigger::now(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_schedule() {
        let mgr = LocalNotificationsManager::new();
        mgr.request_permission();
        let result = mgr.schedule(
            LocalNotification::new("1", "Test", "Body"),
            NotificationTrigger::after(60),
        );
        assert!(result.is_ok());
        assert_eq!(mgr.pending_count(), 1);
    }

    #[test]
    fn test_manager_cancel() {
        let mgr = LocalNotificationsManager::new();
        mgr.request_permission();
        mgr.schedule(
            LocalNotification::new("1", "Test", "Body"),
            NotificationTrigger::after(60),
        );
        assert!(mgr.cancel("1"));
        assert_eq!(mgr.pending_count(), 0);
        assert!(!mgr.cancel("nonexistent"));
    }

    #[test]
    fn test_manager_deliver_immediate() {
        let mgr = LocalNotificationsManager::new();
        mgr.request_permission();
        mgr.schedule(
            LocalNotification::new("1", "Test", "Body"),
            NotificationTrigger::now(),
        );
        let count = mgr.deliver_due();
        assert_eq!(count, 1);
        assert_eq!(mgr.delivered_count(), 1);
        assert_eq!(mgr.pending_count(), 0);
    }

    #[test]
    fn test_manager_deliver_time_interval_not_due() {
        let mgr = LocalNotificationsManager::new();
        mgr.request_permission();
        mgr.schedule(
            LocalNotification::new("1", "Test", "Body"),
            NotificationTrigger::after(3600),
        );
        let count = mgr.deliver_due();
        assert_eq!(count, 0);
        assert_eq!(mgr.pending_count(), 1);
    }

    #[test]
    fn test_manager_delivered_notifications() {
        let mgr = LocalNotificationsManager::new();
        mgr.request_permission();
        mgr.schedule(
            LocalNotification::new("1", "Test", "Body"),
            NotificationTrigger::now(),
        );
        mgr.deliver_due();
        let delivered = mgr.delivered_notifications();
        assert_eq!(delivered.len(), 1);
        assert_eq!(delivered[0].title, "Test");
    }

    #[test]
    fn test_manager_clear_delivered() {
        let mgr = LocalNotificationsManager::new();
        mgr.request_permission();
        mgr.schedule(
            LocalNotification::new("1", "Test", "Body"),
            NotificationTrigger::now(),
        );
        mgr.deliver_due();
        assert_eq!(mgr.delivered_count(), 1);
        mgr.clear_delivered();
        assert_eq!(mgr.delivered_count(), 0);
    }

    #[test]
    fn test_manager_cancel_all() {
        let mgr = LocalNotificationsManager::new();
        mgr.request_permission();
        mgr.schedule(
            LocalNotification::new("1", "A", "B"),
            NotificationTrigger::after(60),
        );
        mgr.schedule(
            LocalNotification::new("2", "C", "D"),
            NotificationTrigger::after(30),
        );
        mgr.cancel_all();
        assert_eq!(mgr.pending_count(), 0);
    }

    #[test]
    fn test_manager_pending_ids() {
        let mgr = LocalNotificationsManager::new();
        mgr.request_permission();
        mgr.schedule(
            LocalNotification::new("a", "A", "B"),
            NotificationTrigger::after(60),
        );
        mgr.schedule(
            LocalNotification::new("b", "C", "D"),
            NotificationTrigger::after(30),
        );
        let ids = mgr.pending_ids();
        assert!(ids.contains(&"a".to_string()));
        assert!(ids.contains(&"b".to_string()));
    }
}
