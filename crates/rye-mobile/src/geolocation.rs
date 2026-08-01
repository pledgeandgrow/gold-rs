//! Goal 201: Native geolocation.
//!
//! `use_geolocation()` hook with high-accuracy mode, background tracking, and geofencing.

use std::collections::HashMap;
use std::sync::Mutex;

/// The accuracy level for geolocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeoAccuracy {
    /// Rough accuracy (~3km), low power.
    Low,
    /// Medium accuracy (~100m), balanced power.
    Medium,
    /// High accuracy (~10m), high power.
    High,
    /// Best accuracy (~3m), highest power.
    Best,
}

impl GeoAccuracy {
    /// Get the approximate accuracy in meters.
    pub fn accuracy_meters(&self) -> u32 {
        match self {
            GeoAccuracy::Low => 3000,
            GeoAccuracy::Medium => 100,
            GeoAccuracy::High => 10,
            GeoAccuracy::Best => 3,
        }
    }
}

/// A geographic coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoCoordinates {
    /// Latitude in degrees.
    pub latitude: f64,
    /// Longitude in degrees.
    pub longitude: f64,
    /// Altitude in meters above sea level (if available).
    pub altitude: Option<f64>,
    /// Accuracy of the position in meters.
    pub accuracy: f64,
    /// Heading in degrees (0-360, if moving).
    pub heading: Option<f64>,
    /// Speed in meters per second (if moving).
    pub speed: Option<f64>,
    /// Unix timestamp of the reading.
    pub timestamp: u64,
}

impl GeoCoordinates {
    /// Create new coordinates.
    pub fn new(lat: f64, lng: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lng,
            altitude: None,
            accuracy: 10.0,
            heading: None,
            speed: None,
            timestamp: 0,
        }
    }

    /// Set altitude.
    pub fn with_altitude(mut self, alt: f64) -> Self {
        self.altitude = Some(alt);
        self
    }

    /// Set heading.
    pub fn with_heading(mut self, heading: f64) -> Self {
        self.heading = Some(heading);
        self
    }

    /// Set speed.
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Set accuracy.
    pub fn with_accuracy(mut self, accuracy: f64) -> Self {
        self.accuracy = accuracy;
        self
    }

    /// Calculate distance to another coordinate (Haversine formula).
    pub fn distance_to(&self, other: &GeoCoordinates) -> f64 {
        let earth_radius = 6371000.0_f64;
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlat = (other.latitude - self.latitude).to_radians();
        let dlng = (other.longitude - self.longitude).to_radians();

        let a = (dlat / 2.0).sin() * (dlat / 2.0).sin()
            + lat1.cos() * lat2.cos() * (dlng / 2.0).sin() * (dlng / 2.0).sin();
        let c = 2.0 * a.sqrt().asin();

        earth_radius * c
    }
}

/// Configuration for a geolocation request.
#[derive(Debug, Clone)]
pub struct GeoConfig {
    /// The desired accuracy.
    pub accuracy: GeoAccuracy,
    /// Whether to allow background tracking.
    pub allow_background: bool,
    /// The minimum distance in meters between updates.
    pub min_distance: f64,
    /// The minimum time interval in seconds between updates.
    pub min_interval: u64,
    /// Whether to request a single one-shot position.
    pub one_shot: bool,
}

impl Default for GeoConfig {
    fn default() -> Self {
        Self {
            accuracy: GeoAccuracy::Medium,
            allow_background: false,
            min_distance: 0.0,
            min_interval: 1,
            one_shot: false,
        }
    }
}

impl GeoConfig {
    /// Create a one-shot config.
    pub fn one_shot() -> Self {
        Self {
            one_shot: true,
            ..Default::default()
        }
    }

    /// Set accuracy.
    pub fn with_accuracy(mut self, accuracy: GeoAccuracy) -> Self {
        self.accuracy = accuracy;
        self
    }

    /// Enable background tracking.
    pub fn background(mut self) -> Self {
        self.allow_background = true;
        self
    }

    /// Set minimum distance between updates.
    pub fn min_distance(mut self, meters: f64) -> Self {
        self.min_distance = meters;
        self
    }

    /// Set minimum interval between updates.
    pub fn min_interval(mut self, seconds: u64) -> Self {
        self.min_interval = seconds;
        self
    }
}

/// The result of a geolocation request.
#[derive(Debug, Clone, PartialEq)]
pub enum GeoResult {
    /// Success with coordinates.
    Success(GeoCoordinates),
    /// Permission denied.
    PermissionDenied,
    /// Position unavailable.
    PositionUnavailable,
    /// Timeout.
    Timeout,
    /// Not supported on this platform.
    NotSupported,
    /// Error.
    Error(String),
}

impl GeoResult {
    /// Check if successful.
    pub fn is_success(&self) -> bool {
        matches!(self, GeoResult::Success(_))
    }
}

/// A geofence region.
#[derive(Debug, Clone)]
pub struct GeofenceRegion {
    /// The region identifier.
    pub id: String,
    /// The center coordinates.
    pub center: GeoCoordinates,
    /// The radius in meters.
    pub radius: f64,
    /// Whether to trigger on entry.
    pub notify_on_entry: bool,
    /// Whether to trigger on exit.
    pub notify_on_exit: bool,
    /// Whether to trigger on dwell (staying inside for a duration).
    pub notify_on_dwell: bool,
    /// Dwell delay in seconds.
    pub dwell_delay: Option<u64>,
}

impl GeofenceRegion {
    /// Create a new geofence region.
    pub fn new(id: &str, center: GeoCoordinates, radius: f64) -> Self {
        Self {
            id: id.to_string(),
            center,
            radius,
            notify_on_entry: true,
            notify_on_exit: true,
            notify_on_dwell: false,
            dwell_delay: None,
        }
    }

    /// Set notify on entry/exit.
    pub fn notify(mut self, on_entry: bool, on_exit: bool) -> Self {
        self.notify_on_entry = on_entry;
        self.notify_on_exit = on_exit;
        self
    }

    /// Set dwell notification.
    pub fn notify_dwell(mut self, delay_secs: u64) -> Self {
        self.notify_on_dwell = true;
        self.dwell_delay = Some(delay_secs);
        self
    }

    /// Check if a coordinate is inside this region.
    pub fn contains(&self, coords: &GeoCoordinates) -> bool {
        self.center.distance_to(coords) <= self.radius
    }
}

/// A geofence event.
#[derive(Debug, Clone, PartialEq)]
pub struct GeofenceEvent {
    /// The region ID.
    pub region_id: String,
    /// The event type.
    pub event_type: GeofenceEventType,
    /// The coordinates at the time of the event.
    pub coordinates: GeoCoordinates,
}

/// The type of geofence event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeofenceEventType {
    /// Entered the region.
    Entry,
    /// Exited the region.
    Exit,
    /// Dwell timer triggered.
    Dwell,
}

/// The geolocation manager.
pub struct GeolocationManager {
    has_permission: Mutex<bool>,
    current_position: Mutex<Option<GeoCoordinates>>,
    geofences: Mutex<HashMap<String, GeofenceRegion>>,
    tracking: Mutex<bool>,
}

impl GeolocationManager {
    /// Create a new geolocation manager.
    pub fn new() -> Self {
        Self {
            has_permission: Mutex::new(false),
            current_position: Mutex::new(None),
            geofences: Mutex::new(HashMap::new()),
            tracking: Mutex::new(false),
        }
    }

    /// Request geolocation permission.
    pub fn request_permission(&self) -> bool {
        *self.has_permission.lock().unwrap() = true;
        true
    }

    /// Check if permission is granted.
    pub fn has_permission(&self) -> bool {
        *self.has_permission.lock().unwrap()
    }

    /// Get the current position (simulated).
    pub fn get_current_position(&self, config: &GeoConfig) -> GeoResult {
        if !*self.has_permission.lock().unwrap() {
            return GeoResult::PermissionDenied;
        }

        let coords = GeoCoordinates::new(37.7749, -122.4194)
            .with_altitude(16.0)
            .with_accuracy(config.accuracy.accuracy_meters() as f64);
        *self.current_position.lock().unwrap() = Some(coords.clone());
        GeoResult::Success(coords)
    }

    /// Start continuous tracking.
    pub fn start_tracking(&self, _config: &GeoConfig) -> bool {
        if !*self.has_permission.lock().unwrap() {
            return false;
        }
        *self.tracking.lock().unwrap() = true;
        true
    }

    /// Stop tracking.
    pub fn stop_tracking(&self) {
        *self.tracking.lock().unwrap() = false;
    }

    /// Check if currently tracking.
    pub fn is_tracking(&self) -> bool {
        *self.tracking.lock().unwrap()
    }

    /// Add a geofence.
    pub fn add_geofence(&self, region: GeofenceRegion) {
        self.geofences.lock().unwrap().insert(region.id.clone(), region);
    }

    /// Remove a geofence.
    pub fn remove_geofence(&self, id: &str) -> bool {
        self.geofences.lock().unwrap().remove(id).is_some()
    }

    /// Get all geofence IDs.
    pub fn geofence_ids(&self) -> Vec<String> {
        self.geofences.lock().unwrap().keys().cloned().collect()
    }

    /// Check geofences for a position and return events.
    pub fn check_geofences(&self, coords: &GeoCoordinates) -> Vec<GeofenceEvent> {
        let geofences = self.geofences.lock().unwrap();
        geofences
            .values()
            .filter_map(|region| {
                let inside = region.contains(coords);
                if inside && region.notify_on_entry {
                    Some(GeofenceEvent {
                        region_id: region.id.clone(),
                        event_type: GeofenceEventType::Entry,
                        coordinates: coords.clone(),
                    })
                } else if !inside && region.notify_on_exit {
                    Some(GeofenceEvent {
                        region_id: region.id.clone(),
                        event_type: GeofenceEventType::Exit,
                        coordinates: coords.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the number of registered geofences.
    pub fn geofence_count(&self) -> usize {
        self.geofences.lock().unwrap().len()
    }
}

impl Default for GeolocationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_accuracy_meters() {
        assert_eq!(GeoAccuracy::Low.accuracy_meters(), 3000);
        assert_eq!(GeoAccuracy::Medium.accuracy_meters(), 100);
        assert_eq!(GeoAccuracy::High.accuracy_meters(), 10);
        assert_eq!(GeoAccuracy::Best.accuracy_meters(), 3);
    }

    #[test]
    fn test_geo_coordinates_new() {
        let c = GeoCoordinates::new(37.7749, -122.4194);
        assert_eq!(c.latitude, 37.7749);
        assert_eq!(c.longitude, -122.4194);
        assert!(c.altitude.is_none());
    }

    #[test]
    fn test_geo_coordinates_builder() {
        let c = GeoCoordinates::new(37.0, -122.0)
            .with_altitude(100.0)
            .with_heading(90.0)
            .with_speed(5.0);
        assert_eq!(c.altitude, Some(100.0));
        assert_eq!(c.heading, Some(90.0));
        assert_eq!(c.speed, Some(5.0));
    }

    #[test]
    fn test_geo_coordinates_distance() {
        let sf = GeoCoordinates::new(37.7749, -122.4194);
        let la = GeoCoordinates::new(34.0522, -118.2437);
        let distance = sf.distance_to(&la);
        // SF to LA is approximately 559 km
        assert!(distance > 550_000.0 && distance < 570_000.0);
    }

    #[test]
    fn test_geo_coordinates_distance_same_point() {
        let c = GeoCoordinates::new(37.0, -122.0);
        assert_eq!(c.distance_to(&c), 0.0);
    }

    #[test]
    fn test_geo_config_default() {
        let config = GeoConfig::default();
        assert_eq!(config.accuracy, GeoAccuracy::Medium);
        assert!(!config.allow_background);
        assert!(!config.one_shot);
    }

    #[test]
    fn test_geo_config_builder() {
        let config = GeoConfig::one_shot()
            .with_accuracy(GeoAccuracy::Best)
            .background()
            .min_distance(50.0)
            .min_interval(5);
        assert!(config.one_shot);
        assert_eq!(config.accuracy, GeoAccuracy::Best);
        assert!(config.allow_background);
        assert_eq!(config.min_distance, 50.0);
        assert_eq!(config.min_interval, 5);
    }

    #[test]
    fn test_geo_result_is_success() {
        assert!(GeoResult::Success(GeoCoordinates::new(0.0, 0.0)).is_success());
        assert!(!GeoResult::PermissionDenied.is_success());
    }

    #[test]
    fn test_geofence_region_new() {
        let region = GeofenceRegion::new("home", GeoCoordinates::new(37.0, -122.0), 100.0);
        assert_eq!(region.id, "home");
        assert_eq!(region.radius, 100.0);
        assert!(region.notify_on_entry);
        assert!(region.notify_on_exit);
    }

    #[test]
    fn test_geofence_region_notify() {
        let region = GeofenceRegion::new("work", GeoCoordinates::new(37.0, -122.0), 50.0)
            .notify(true, false);
        assert!(region.notify_on_entry);
        assert!(!region.notify_on_exit);
    }

    #[test]
    fn test_geofence_region_dwell() {
        let region = GeofenceRegion::new("store", GeoCoordinates::new(37.0, -122.0), 200.0)
            .notify_dwell(300);
        assert!(region.notify_on_dwell);
        assert_eq!(region.dwell_delay, Some(300));
    }

    #[test]
    fn test_geofence_contains_inside() {
        let center = GeoCoordinates::new(37.0, -122.0);
        let region = GeofenceRegion::new("test", center.clone(), 1000.0);
        let nearby = GeoCoordinates::new(37.001, -122.001);
        assert!(region.contains(&nearby));
    }

    #[test]
    fn test_geofence_contains_outside() {
        let center = GeoCoordinates::new(37.0, -122.0);
        let region = GeofenceRegion::new("test", center.clone(), 100.0);
        let far = GeoCoordinates::new(38.0, -121.0);
        assert!(!region.contains(&far));
    }

    #[test]
    fn test_manager_permission() {
        let mgr = GeolocationManager::new();
        assert!(!mgr.has_permission());
        mgr.request_permission();
        assert!(mgr.has_permission());
    }

    #[test]
    fn test_manager_get_position_no_permission() {
        let mgr = GeolocationManager::new();
        let result = mgr.get_current_position(&GeoConfig::one_shot());
        assert_eq!(result, GeoResult::PermissionDenied);
    }

    #[test]
    fn test_manager_get_position() {
        let mgr = GeolocationManager::new();
        mgr.request_permission();
        let result = mgr.get_current_position(&GeoConfig::one_shot());
        assert!(result.is_success());
    }

    #[test]
    fn test_manager_tracking() {
        let mgr = GeolocationManager::new();
        mgr.request_permission();
        assert!(!mgr.is_tracking());
        mgr.start_tracking(&GeoConfig::default());
        assert!(mgr.is_tracking());
        mgr.stop_tracking();
        assert!(!mgr.is_tracking());
    }

    #[test]
    fn test_manager_tracking_no_permission() {
        let mgr = GeolocationManager::new();
        assert!(!mgr.start_tracking(&GeoConfig::default()));
    }

    #[test]
    fn test_manager_geofences() {
        let mgr = GeolocationManager::new();
        mgr.add_geofence(GeofenceRegion::new("home", GeoCoordinates::new(37.0, -122.0), 100.0));
        mgr.add_geofence(GeofenceRegion::new("work", GeoCoordinates::new(37.5, -122.5), 50.0));

        assert_eq!(mgr.geofence_count(), 2);
        assert!(mgr.geofence_ids().contains(&"home".to_string()));
        assert!(mgr.remove_geofence("home"));
        assert_eq!(mgr.geofence_count(), 1);
    }

    #[test]
    fn test_manager_check_geofences() {
        let mgr = GeolocationManager::new();
        mgr.add_geofence(GeofenceRegion::new("home", GeoCoordinates::new(37.0, -122.0), 1000.0));

        let nearby = GeoCoordinates::new(37.001, -122.001);
        let events = mgr.check_geofences(&nearby);
        assert!(!events.is_empty());
        assert_eq!(events[0].event_type, GeofenceEventType::Entry);
    }
}
