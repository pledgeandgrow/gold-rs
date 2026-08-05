//! Goal 217: Speculative preloading.
//!
//! Predict which route the user is likely to navigate to next (based on
//! hover, scroll position, link proximity) and preload its Wasm chunk and data.

use std::collections::HashMap;
use std::sync::Mutex;

/// The trigger for speculative preloading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreloadTrigger {
    /// User hovered over a link.
    Hover,
    /// Link is near the viewport (scroll proximity).
    ViewportProximity,
    /// User is likely to navigate based on app flow.
    AppFlowPrediction,
    /// Explicit prefetch request.
    Explicit,
}

impl PreloadTrigger {
    /// Get the priority weight for this trigger.
    pub fn priority_weight(&self) -> f64 {
        match self {
            PreloadTrigger::Hover => 0.9,
            PreloadTrigger::ViewportProximity => 0.6,
            PreloadTrigger::AppFlowPrediction => 0.4,
            PreloadTrigger::Explicit => 1.0,
        }
    }
}

/// A preload candidate — a route that might be navigated to.
#[derive(Debug, Clone)]
pub struct PreloadCandidate {
    /// The route path.
    pub route: String,
    /// The chunk ID to preload.
    pub chunk_id: String,
    /// The trigger that suggested this preload.
    pub trigger: PreloadTrigger,
    /// The confidence score (0.0-1.0).
    pub confidence: f64,
    /// Whether the preload has been started.
    pub preloaded: bool,
    /// The data URL to prefetch (if any).
    pub data_url: Option<String>,
}

impl PreloadCandidate {
    /// Create a new preload candidate.
    pub fn new(route: &str, chunk_id: &str, trigger: PreloadTrigger) -> Self {
        let confidence = trigger.priority_weight();
        Self {
            route: route.to_string(),
            chunk_id: chunk_id.to_string(),
            trigger,
            confidence,
            preloaded: false,
            data_url: None,
        }
    }

    /// Set the confidence score.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    /// Set the data URL to prefetch.
    pub fn with_data_url(mut self, url: &str) -> Self {
        self.data_url = Some(url.to_string());
        self
    }

    /// Get the effective priority (confidence * trigger weight).
    pub fn effective_priority(&self) -> f64 {
        self.confidence * self.trigger.priority_weight()
    }
}

/// The preload status of a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreloadStatus {
    /// Not preloaded.
    NotPreloaded,
    /// Currently preloading.
    Preloading,
    /// Preloaded and ready.
    Ready,
    /// Preload failed.
    Failed,
}

/// The speculative preloader — manages preload predictions and execution.
pub struct SpeculativePreloader {
    candidates: Mutex<HashMap<String, PreloadCandidate>>,
    statuses: Mutex<HashMap<String, PreloadStatus>>,
    preload_threshold: f64,
    max_concurrent_preloads: usize,
    active_preloads: Mutex<usize>,
    preload_count: Mutex<u32>,
    hit_count: Mutex<u32>,
}

impl SpeculativePreloader {
    /// Create a new speculative preloader.
    pub fn new() -> Self {
        Self {
            candidates: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
            preload_threshold: 0.5,
            max_concurrent_preloads: 3,
            active_preloads: Mutex::new(0),
            preload_count: Mutex::new(0),
            hit_count: Mutex::new(0),
        }
    }

    /// Set the preload threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.preload_threshold = threshold;
        self
    }

    /// Set the max concurrent preloads.
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent_preloads = max;
        self
    }

    /// Register a preload candidate.
    pub fn register(&self, candidate: PreloadCandidate) {
        let route = candidate.route.clone();
        self.candidates.lock().unwrap().insert(route.clone(), candidate);
        self.statuses.lock().unwrap().insert(route, PreloadStatus::NotPreloaded);
    }

    /// Check if a route should be preloaded based on its priority.
    pub fn should_preload(&self, route: &str) -> bool {
        self.should_preload_inner(route, &self.candidates.lock().unwrap(), &self.statuses.lock().unwrap())
    }

    /// Inner check that accepts pre-locked guards to avoid re-entrant locking.
    fn should_preload_inner(
        &self,
        route: &str,
        candidates: &HashMap<String, PreloadCandidate>,
        statuses: &HashMap<String, PreloadStatus>,
    ) -> bool {
        if let Some(candidate) = candidates.get(route) {
            if candidate.preloaded {
                return false;
            }
            if let Some(status) = statuses.get(route) {
                if *status == PreloadStatus::Preloading || *status == PreloadStatus::Ready {
                    return false;
                }
            }
            return candidate.effective_priority() >= self.preload_threshold;
        }
        false
    }

    /// Start preloading a route.
    pub fn preload(&self, route: &str) -> bool {
        let mut candidates = self.candidates.lock().unwrap();
        let mut statuses = self.statuses.lock().unwrap();

        if !self.should_preload_inner(route, &candidates, &statuses) {
            return false;
        }

        let active = *self.active_preloads.lock().unwrap();
        if active >= self.max_concurrent_preloads {
            return false;
        }

        *self.active_preloads.lock().unwrap() += 1;
        statuses.insert(route.to_string(), PreloadStatus::Preloading);

        if let Some(candidate) = candidates.get_mut(route) {
            candidate.preloaded = true;
        }

        *self.preload_count.lock().unwrap() += 1;
        true
    }

    /// Mark a preload as complete.
    pub fn mark_ready(&self, route: &str) {
        self.statuses.lock().unwrap().insert(route.to_string(), PreloadStatus::Ready);
        let mut active = self.active_preloads.lock().unwrap();
        *active = active.saturating_sub(1);
    }

    /// Mark a preload as failed.
    pub fn mark_failed(&self, route: &str) {
        self.statuses.lock().unwrap().insert(route.to_string(), PreloadStatus::Failed);
        let mut active = self.active_preloads.lock().unwrap();
        *active = active.saturating_sub(1);
    }

    /// Check if a route has been preloaded (navigation hit).
    pub fn check_hit(&self, route: &str) -> bool {
        let is_ready = {
            let statuses = self.statuses.lock().unwrap();
            statuses.get(route).map(|s| *s == PreloadStatus::Ready).unwrap_or(false)
        };
        if is_ready {
            *self.hit_count.lock().unwrap() += 1;
            return true;
        }
        false
    }

    /// Get the preload status for a route.
    pub fn status(&self, route: &str) -> PreloadStatus {
        self.statuses.lock().unwrap().get(route).copied().unwrap_or(PreloadStatus::NotPreloaded)
    }

    /// Get all routes that should be preloaded.
    pub fn routes_to_preload(&self) -> Vec<String> {
        let candidates = self.candidates.lock().unwrap();
        let statuses = self.statuses.lock().unwrap();
        candidates
            .keys()
            .filter(|route| self.should_preload_inner(route, &candidates, &statuses))
            .cloned()
            .collect()
    }

    /// Get the number of registered candidates.
    pub fn candidate_count(&self) -> usize {
        self.candidates.lock().unwrap().len()
    }

    /// Get the number of preloaded routes.
    pub fn preload_count(&self) -> u32 {
        *self.preload_count.lock().unwrap()
    }

    /// Get the number of cache hits (navigations to preloaded routes).
    pub fn hit_count(&self) -> u32 {
        *self.hit_count.lock().unwrap()
    }

    /// Get the hit rate (0.0-1.0).
    pub fn hit_rate(&self) -> f64 {
        let preloads = self.preload_count();
        if preloads == 0 {
            return 0.0;
        }
        self.hit_count() as f64 / preloads as f64
    }

    /// Get the number of active preloads.
    pub fn active_preloads(&self) -> usize {
        *self.active_preloads.lock().unwrap()
    }

    /// Clear all candidates and statuses.
    pub fn clear(&self) {
        self.candidates.lock().unwrap().clear();
        self.statuses.lock().unwrap().clear();
        *self.active_preloads.lock().unwrap() = 0;
    }

    /// Generate the JavaScript preload script.
    pub fn generate_preload_script(&self) -> String {
        r#"(function(){var s=window.__ryePreload={candidates:[],preloaded:{},threshold:0.5};
s.hover=function(route,chunkId){var c={route:route,chunkId:chunkId,trigger:'hover',confidence:0.9};
if(c.confidence>=s.threshold&&!s.preloaded[route]){s.preloaded[route]='preloading';
var l=document.createElement('link');l.rel='modulepreload';l.href=chunkId;document.head.appendChild(l);
s.preloaded[route]='ready';}};
s.proximity=function(route,chunkId){var c={route:route,chunkId:chunkId,trigger:'proximity',confidence:0.6};
if(c.confidence>=s.threshold&&!s.preloaded[route]){s.preloaded[route]='preloading';
var l=document.createElement('link');l.rel='modulepreload';l.href=chunkId;document.head.appendChild(l);
s.preloaded[route]='ready';}};
document.addEventListener('mouseover',function(e){var a=e.target.closest('a[href]');if(a){s.hover(a.getAttribute('href'),a.getAttribute('data-chunk'));}},true);
})();"#.to_string()
    }
}

impl Default for SpeculativePreloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preload_trigger_priority_weight() {
        assert!(PreloadTrigger::Hover.priority_weight() > PreloadTrigger::ViewportProximity.priority_weight());
        assert!(PreloadTrigger::Explicit.priority_weight() > PreloadTrigger::AppFlowPrediction.priority_weight());
    }

    #[test]
    fn test_preload_candidate_new() {
        let candidate = PreloadCandidate::new("/about", "chunk-about", PreloadTrigger::Hover);
        assert_eq!(candidate.route, "/about");
        assert_eq!(candidate.chunk_id, "chunk-about");
        assert!(!candidate.preloaded);
    }

    #[test]
    fn test_preload_candidate_with_data_url() {
        let candidate = PreloadCandidate::new("/users", "chunk-users", PreloadTrigger::Explicit)
            .with_data_url("/api/users");
        assert_eq!(candidate.data_url, Some("/api/users".to_string()));
    }

    #[test]
    fn test_preload_candidate_effective_priority() {
        let hover = PreloadCandidate::new("/a", "c1", PreloadTrigger::Hover);
        let proximity = PreloadCandidate::new("/b", "c2", PreloadTrigger::ViewportProximity);
        assert!(hover.effective_priority() > proximity.effective_priority());
    }

    #[test]
    fn test_preloader_register() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::Hover));
        assert_eq!(preloader.candidate_count(), 1);
        assert_eq!(preloader.status("/about"), PreloadStatus::NotPreloaded);
    }

    #[test]
    fn test_preloader_should_preload() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::Hover));
        assert!(preloader.should_preload("/about"));
    }

    #[test]
    fn test_preloader_should_not_preload_low_confidence() {
        let preloader = SpeculativePreloader::new().with_threshold(0.95);
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::ViewportProximity));
        assert!(!preloader.should_preload("/about"));
    }

    #[test]
    fn test_preloader_should_not_preload_already_preloaded() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::Hover));
        preloader.preload("/about");
        assert!(!preloader.should_preload("/about"));
    }

    #[test]
    fn test_preloader_preload() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::Hover));
        assert!(preloader.preload("/about"));
        assert_eq!(preloader.status("/about"), PreloadStatus::Preloading);
        assert_eq!(preloader.preload_count(), 1);
        assert_eq!(preloader.active_preloads(), 1);
    }

    #[test]
    fn test_preloader_preload_not_registered() {
        let preloader = SpeculativePreloader::new();
        assert!(!preloader.preload("/unknown"));
    }

    #[test]
    fn test_preloader_max_concurrent() {
        let preloader = SpeculativePreloader::new().with_max_concurrent(2);
        preloader.register(PreloadCandidate::new("/a", "c1", PreloadTrigger::Hover));
        preloader.register(PreloadCandidate::new("/b", "c2", PreloadTrigger::Hover));
        preloader.register(PreloadCandidate::new("/c", "c3", PreloadTrigger::Hover));

        assert!(preloader.preload("/a"));
        assert!(preloader.preload("/b"));
        assert!(!preloader.preload("/c")); // Max concurrent reached
    }

    #[test]
    fn test_preloader_mark_ready() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::Hover));
        preloader.preload("/about");
        preloader.mark_ready("/about");
        assert_eq!(preloader.status("/about"), PreloadStatus::Ready);
        assert_eq!(preloader.active_preloads(), 0);
    }

    #[test]
    fn test_preloader_mark_failed() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::Hover));
        preloader.preload("/about");
        preloader.mark_failed("/about");
        assert_eq!(preloader.status("/about"), PreloadStatus::Failed);
    }

    #[test]
    fn test_preloader_check_hit() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::Hover));
        preloader.preload("/about");
        preloader.mark_ready("/about");

        assert!(preloader.check_hit("/about"));
        assert_eq!(preloader.hit_count(), 1);
    }

    #[test]
    fn test_preloader_check_hit_not_ready() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/about", "chunk", PreloadTrigger::Hover));
        preloader.preload("/about");
        assert!(!preloader.check_hit("/about"));
    }

    #[test]
    fn test_preloader_hit_rate() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/a", "c1", PreloadTrigger::Hover));
        preloader.register(PreloadCandidate::new("/b", "c2", PreloadTrigger::Hover));
        preloader.preload("/a");
        preloader.preload("/b");
        preloader.mark_ready("/a");
        preloader.mark_ready("/b");
        preloader.check_hit("/a");
        preloader.check_hit("/b");
        assert_eq!(preloader.hit_rate(), 1.0);
    }

    #[test]
    fn test_preloader_routes_to_preload() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/a", "c1", PreloadTrigger::Hover));
        preloader.register(PreloadCandidate::new("/b", "c2", PreloadTrigger::AppFlowPrediction));
        let routes = preloader.routes_to_preload();
        assert!(routes.contains(&"/a".to_string()));
    }

    #[test]
    fn test_preloader_clear() {
        let preloader = SpeculativePreloader::new();
        preloader.register(PreloadCandidate::new("/a", "c1", PreloadTrigger::Hover));
        preloader.clear();
        assert_eq!(preloader.candidate_count(), 0);
    }

    #[test]
    fn test_preloader_generate_script() {
        let preloader = SpeculativePreloader::new();
        let script = preloader.generate_preload_script();
        assert!(script.contains("modulepreload"));
        assert!(script.contains("mouseover"));
    }
}
