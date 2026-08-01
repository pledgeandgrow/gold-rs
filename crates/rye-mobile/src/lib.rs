//! # rye-mobile
//!
//! Mobile renderer for rye — iOS and Android via wgpu + winit.
//! Phase 19: Native & Mobile Deep Cuts (Goals 196–210).

#![deny(missing_docs)]

pub mod lifecycle;
pub mod native_module;
pub mod push_notifications;
pub mod biometric;
pub mod share;
pub mod camera;
pub mod geolocation;
pub mod contacts;
pub mod local_notifications;
pub mod iap;
pub mod deep_link;
pub mod background_tasks;
pub mod haptics;
pub mod permissions;
pub mod lifecycle_persistence;
pub mod widgets;

pub use lifecycle::MobileLifecycle;
pub use native_module::{NativeModule, NativeModuleRegistry, NativeModuleBuilder, NativePlatform, NativeType, NativeFunction};
pub use push_notifications::{PushNotificationManager, PushNotification, NotificationChannel, NotificationAction, PushPermissionState};
pub use biometric::{BiometricAuthManager, BiometricAuthConfig, BiometricAuthResult, BiometricType, BiometricAvailability};
pub use share::{ShareManager, ShareContent, ShareResult, ShareConfig};
pub use camera::{CameraManager, CameraConfig, CameraResult, GalleryConfig, GalleryResult, CapturedMedia, CaptureType, CameraDirection};
pub use geolocation::{GeolocationManager, GeoCoordinates, GeoConfig, GeoResult, GeoAccuracy, GeofenceRegion, GeofenceEvent, GeofenceEventType};
pub use contacts::{ContactsManager, Contact, ContactField, ContactAddress, ContactsConfig, ContactsResult};
pub use local_notifications::{LocalNotificationsManager, LocalNotification, NotificationTrigger, NotificationPermissionState};
pub use iap::{IapManager, Product, Purchase, PurchaseResult, ProductType, PurchaseState};
pub use deep_link::{DeepLinkManager, DeepLink, DeepLinkRoute};
pub use background_tasks::{BackgroundTaskScheduler, BackgroundTask, TaskOutcome, TaskState, BackgroundTaskType, TaskConstraints};
pub use haptics::{HapticsManager, HapticImpact, HapticNotification, HapticSelection, HapticPattern};
pub use permissions::{PermissionsManager, Permission, PermissionState, PermissionRequestResult};
pub use lifecycle_persistence::{LifecyclePersistenceManager, StateSnapshot, StorageType};
pub use widgets::{WidgetManager, WidgetDefinition, WidgetState, WidgetBinding, WidgetPlatform, WidgetSize};
