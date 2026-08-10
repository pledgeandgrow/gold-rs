//! # rye-mobile
//!
//! Mobile renderer for rye — iOS and Android via wgpu + winit.
//! Phase 19: Native & Mobile Deep Cuts (Goals 196–210).

#![deny(missing_docs)]

pub mod background_tasks;
pub mod biometric;
pub mod camera;
pub mod contacts;
pub mod deep_link;
pub mod ffi;
pub mod geolocation;
pub mod haptics;
pub mod iap;
pub mod lifecycle;
pub mod lifecycle_persistence;
pub mod local_notifications;
pub mod native_module;
pub mod permissions;
pub mod push_notifications;
pub mod share;
pub mod widgets;

pub use background_tasks::{
    BackgroundTask, BackgroundTaskScheduler, BackgroundTaskType, TaskConstraints, TaskOutcome,
    TaskState,
};
pub use biometric::{
    BiometricAuthConfig, BiometricAuthManager, BiometricAuthResult, BiometricAvailability,
    BiometricType,
};
pub use camera::{
    CameraConfig, CameraDirection, CameraManager, CameraResult, CaptureType, CapturedMedia,
    GalleryConfig, GalleryResult,
};
pub use contacts::{
    Contact, ContactAddress, ContactField, ContactsConfig, ContactsManager, ContactsResult,
};
pub use deep_link::{DeepLink, DeepLinkManager, DeepLinkRoute};
pub use geolocation::{
    GeoAccuracy, GeoConfig, GeoCoordinates, GeoResult, GeofenceEvent, GeofenceEventType,
    GeofenceRegion, GeolocationManager,
};
pub use haptics::{
    HapticImpact, HapticNotification, HapticPattern, HapticSelection, HapticsManager,
};
pub use iap::{IapManager, Product, ProductType, Purchase, PurchaseResult, PurchaseState};
pub use lifecycle::MobileLifecycle;
pub use lifecycle_persistence::{LifecyclePersistenceManager, StateSnapshot, StorageType};
pub use local_notifications::{
    LocalNotification, LocalNotificationsManager, NotificationPermissionState, NotificationTrigger,
};
pub use native_module::{
    NativeFunction, NativeModule, NativeModuleBuilder, NativeModuleRegistry, NativePlatform,
    NativeType,
};
pub use permissions::{Permission, PermissionRequestResult, PermissionState, PermissionsManager};
pub use push_notifications::{
    NotificationAction, NotificationChannel, PushNotification, PushNotificationManager,
    PushPermissionState,
};
pub use share::{ShareConfig, ShareContent, ShareManager, ShareResult};
pub use widgets::{
    WidgetBinding, WidgetDefinition, WidgetManager, WidgetPlatform, WidgetSize, WidgetState,
};
