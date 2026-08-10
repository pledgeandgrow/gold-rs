//! Goal 211: Incremental hydration.
//!
//! Instead of hydrating the entire page at once, hydrate components
//! incrementally as the browser becomes idle. Priority queue based on
//! viewport proximity and interaction likelihood.

use std::collections::{BinaryHeap, HashMap};
use std::sync::Mutex;

/// The priority level for hydration scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HydrationPriority {
    /// Critical — above the fold, visible immediately.
    Critical = 0,
    /// High — near viewport or likely to be interacted with.
    High = 1,
    /// Normal — below the fold, standard priority.
    Normal = 2,
    /// Low — off-screen, idle hydration.
    Low = 3,
    /// Background — hydrate only when completely idle.
    Background = 4,
}

impl HydrationPriority {
    /// Determine priority from viewport proximity (0.0 = top of viewport, 1.0 = far below).
    pub fn from_viewport_proximity(proximity: f64) -> Self {
        if proximity < 0.0 {
            HydrationPriority::Critical
        } else if proximity < 0.1 {
            HydrationPriority::Critical
        } else if proximity < 0.3 {
            HydrationPriority::High
        } else if proximity < 0.6 {
            HydrationPriority::Normal
        } else if proximity < 1.0 {
            HydrationPriority::Low
        } else {
            HydrationPriority::Background
        }
    }

    /// Check if this priority should hydrate during idle time.
    pub fn is_idle_only(&self) -> bool {
        matches!(self, HydrationPriority::Low | HydrationPriority::Background)
    }
}

/// A component scheduled for incremental hydration.
#[derive(Debug, Clone)]
pub struct HydrationTask {
    /// The component ID to hydrate.
    pub component_id: String,
    /// The priority level.
    pub priority: HydrationPriority,
    /// Whether the component has been hydrated.
    pub hydrated: bool,
    /// The viewport proximity (0.0 = at viewport, higher = further away).
    pub viewport_proximity: f64,
    /// Interaction likelihood score (0.0-1.0).
    pub interaction_likelihood: f64,
    /// The chunk ID needed for hydration (if code-split).
    pub chunk_id: Option<String>,
}

impl HydrationTask {
    /// Create a new hydration task.
    pub fn new(component_id: &str, priority: HydrationPriority) -> Self {
        Self {
            component_id: component_id.to_string(),
            priority,
            hydrated: false,
            viewport_proximity: 0.0,
            interaction_likelihood: 0.0,
            chunk_id: None,
        }
    }

    /// Set viewport proximity.
    pub fn with_proximity(mut self, proximity: f64) -> Self {
        self.viewport_proximity = proximity;
        self
    }

    /// Set interaction likelihood.
    pub fn with_interaction_likelihood(mut self, likelihood: f64) -> Self {
        self.interaction_likelihood = likelihood;
        self
    }

    /// Set the chunk ID needed for hydration.
    pub fn with_chunk(mut self, chunk_id: &str) -> Self {
        self.chunk_id = Some(chunk_id.to_string());
        self
    }

    /// Compute a composite score for scheduling (lower = higher priority).
    pub fn score(&self) -> f64 {
        let priority_weight = self.priority as u8 as f64 * 100.0;
        let proximity_weight = self.viewport_proximity * 50.0;
        let interaction_weight = (1.0 - self.interaction_likelihood) * 30.0;
        priority_weight + proximity_weight + interaction_weight
    }
}

impl PartialEq for HydrationTask {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl Eq for HydrationTask {}

impl PartialOrd for HydrationTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Reverse ordering for min-heap behavior (lower score = higher priority)
        Some(self.cmp(other))
    }
}

impl Ord for HydrationTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse so that lower scores come first (higher priority)
        other
            .score()
            .partial_cmp(&self.score())
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// The incremental hydration scheduler — manages the priority queue and
/// hydrates components as the browser becomes idle.
pub struct IncrementalHydrationScheduler {
    tasks: Mutex<HashMap<String, HydrationTask>>,
    queue: Mutex<BinaryHeap<HydrationTask>>,
    hydrated_count: Mutex<usize>,
    total_count: Mutex<usize>,
    idle_batch_size: usize,
}

impl IncrementalHydrationScheduler {
    /// Create a new scheduler.
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            queue: Mutex::new(BinaryHeap::new()),
            hydrated_count: Mutex::new(0),
            total_count: Mutex::new(0),
            idle_batch_size: 3,
        }
    }

    /// Set the batch size for idle hydration.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.idle_batch_size = batch_size;
        self
    }

    /// Schedule a component for hydration.
    pub fn schedule(&self, task: HydrationTask) {
        let id = task.component_id.clone();
        self.tasks.lock().unwrap().insert(id, task.clone());
        self.queue.lock().unwrap().push(task);
        *self.total_count.lock().unwrap() += 1;
    }

    /// Get the next batch of components to hydrate during idle time.
    pub fn next_idle_batch(&self) -> Vec<String> {
        let mut queue = self.queue.lock().unwrap();
        let mut batch = Vec::new();
        let mut count = 0;

        while count < self.idle_batch_size {
            if let Some(task) = queue.pop() {
                if !task.hydrated {
                    batch.push(task.component_id.clone());
                    count += 1;
                }
            } else {
                break;
            }
        }

        batch
    }

    /// Get the next critical/high priority batch (for immediate hydration).
    pub fn next_critical_batch(&self) -> Vec<String> {
        let mut queue = self.queue.lock().unwrap();
        let mut batch = Vec::new();
        let mut remaining = Vec::new();

        while let Some(task) = queue.pop() {
            if !task.hydrated
                && (task.priority == HydrationPriority::Critical
                    || task.priority == HydrationPriority::High)
            {
                batch.push(task.component_id.clone());
            } else {
                remaining.push(task);
            }
        }

        *queue = remaining.into_iter().collect();
        batch
    }

    /// Mark a component as hydrated.
    pub fn mark_hydrated(&self, component_id: &str) {
        if let Some(task) = self.tasks.lock().unwrap().get_mut(component_id) {
            if !task.hydrated {
                task.hydrated = true;
                *self.hydrated_count.lock().unwrap() += 1;
            }
        }
    }

    /// Get the number of hydrated components.
    pub fn hydrated_count(&self) -> usize {
        *self.hydrated_count.lock().unwrap()
    }

    /// Get the total number of scheduled components.
    pub fn total_count(&self) -> usize {
        *self.total_count.lock().unwrap()
    }

    /// Get the number of pending (non-hydrated) components.
    pub fn pending_count(&self) -> usize {
        self.total_count() - self.hydrated_count()
    }

    /// Check if all components have been hydrated.
    pub fn is_complete(&self) -> bool {
        self.hydrated_count() == self.total_count() && self.total_count() > 0
    }

    /// Get the hydration progress as a percentage (0.0-1.0).
    pub fn progress(&self) -> f64 {
        let total = self.total_count();
        if total == 0 {
            return 1.0;
        }
        self.hydrated_count() as f64 / total as f64
    }

    /// Get all pending component IDs.
    pub fn pending_ids(&self) -> Vec<String> {
        self.tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, t)| !t.hydrated)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all hydrated component IDs.
    pub fn hydrated_ids(&self) -> Vec<String> {
        self.tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, t)| t.hydrated)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Update viewport proximity for a component (re-prioritize).
    pub fn update_proximity(&self, component_id: &str, proximity: f64) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(component_id) {
            task.viewport_proximity = proximity;
            task.priority = HydrationPriority::from_viewport_proximity(proximity);
        }
    }

    /// Generate the JavaScript idle hydration scheduler script.
    pub fn generate_idle_script(&self) -> String {
        format!(
            r#"(function(){{var s=window.__ryeHydration={{batch:{},hydrated:0,pending:[]}};
s.tick=function(){{if(typeof requestIdleCallback!=='function')return;
requestIdleCallback(function(deadline){{while(deadline.timeRemaining()>0&&s.pending.length>0){{var id=s.pending.shift();var el=document.querySelector('[data-rye-id="'+id+'"]');if(el){{el.setAttribute('data-rye-hydrated','true');s.hydrated++;}}}}if(s.pending.length>0)s.tick();}});}};
}})();"#,
            self.idle_batch_size
        )
    }
}

impl Default for IncrementalHydrationScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hydration_priority_from_proximity() {
        assert_eq!(
            HydrationPriority::from_viewport_proximity(-1.0),
            HydrationPriority::Critical
        );
        assert_eq!(
            HydrationPriority::from_viewport_proximity(0.05),
            HydrationPriority::Critical
        );
        assert_eq!(
            HydrationPriority::from_viewport_proximity(0.2),
            HydrationPriority::High
        );
        assert_eq!(
            HydrationPriority::from_viewport_proximity(0.5),
            HydrationPriority::Normal
        );
        assert_eq!(
            HydrationPriority::from_viewport_proximity(0.8),
            HydrationPriority::Low
        );
        assert_eq!(
            HydrationPriority::from_viewport_proximity(1.5),
            HydrationPriority::Background
        );
    }

    #[test]
    fn test_hydration_priority_is_idle_only() {
        assert!(!HydrationPriority::Critical.is_idle_only());
        assert!(HydrationPriority::Low.is_idle_only());
        assert!(HydrationPriority::Background.is_idle_only());
    }

    #[test]
    fn test_hydration_task_new() {
        let task = HydrationTask::new("comp1", HydrationPriority::High);
        assert_eq!(task.component_id, "comp1");
        assert_eq!(task.priority, HydrationPriority::High);
        assert!(!task.hydrated);
    }

    #[test]
    fn test_hydration_task_builder() {
        let task = HydrationTask::new("comp1", HydrationPriority::Normal)
            .with_proximity(0.3)
            .with_interaction_likelihood(0.8)
            .with_chunk("chunk-42");
        assert_eq!(task.viewport_proximity, 0.3);
        assert_eq!(task.interaction_likelihood, 0.8);
        assert_eq!(task.chunk_id, Some("chunk-42".to_string()));
    }

    #[test]
    fn test_hydration_task_score() {
        let critical = HydrationTask::new("a", HydrationPriority::Critical).with_proximity(0.0);
        let background = HydrationTask::new("b", HydrationPriority::Background).with_proximity(1.0);
        assert!(critical.score() < background.score());
    }

    #[test]
    fn test_scheduler_schedule() {
        let scheduler = IncrementalHydrationScheduler::new();
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Critical));
        scheduler.schedule(HydrationTask::new("b", HydrationPriority::Normal));
        assert_eq!(scheduler.total_count(), 2);
        assert_eq!(scheduler.pending_count(), 2);
    }

    #[test]
    fn test_scheduler_mark_hydrated() {
        let scheduler = IncrementalHydrationScheduler::new();
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Critical));
        scheduler.mark_hydrated("a");
        assert_eq!(scheduler.hydrated_count(), 1);
        assert_eq!(scheduler.pending_count(), 0);
    }

    #[test]
    fn test_scheduler_progress() {
        let scheduler = IncrementalHydrationScheduler::new();
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Critical));
        scheduler.schedule(HydrationTask::new("b", HydrationPriority::Normal));
        assert_eq!(scheduler.progress(), 0.0);
        scheduler.mark_hydrated("a");
        assert_eq!(scheduler.progress(), 0.5);
        scheduler.mark_hydrated("b");
        assert_eq!(scheduler.progress(), 1.0);
    }

    #[test]
    fn test_scheduler_is_complete() {
        let scheduler = IncrementalHydrationScheduler::new();
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Critical));
        assert!(!scheduler.is_complete());
        scheduler.mark_hydrated("a");
        assert!(scheduler.is_complete());
    }

    #[test]
    fn test_scheduler_next_idle_batch() {
        let scheduler = IncrementalHydrationScheduler::new().with_batch_size(2);
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Critical));
        scheduler.schedule(HydrationTask::new("b", HydrationPriority::Normal));
        scheduler.schedule(HydrationTask::new("c", HydrationPriority::Background));
        let batch = scheduler.next_idle_batch();
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_scheduler_next_critical_batch() {
        let scheduler = IncrementalHydrationScheduler::new();
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Critical));
        scheduler.schedule(HydrationTask::new("b", HydrationPriority::High));
        scheduler.schedule(HydrationTask::new("c", HydrationPriority::Low));
        let batch = scheduler.next_critical_batch();
        assert_eq!(batch.len(), 2);
        assert!(batch.contains(&"a".to_string()));
        assert!(batch.contains(&"b".to_string()));
        assert!(!batch.contains(&"c".to_string()));
    }

    #[test]
    fn test_scheduler_pending_ids() {
        let scheduler = IncrementalHydrationScheduler::new();
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Critical));
        scheduler.schedule(HydrationTask::new("b", HydrationPriority::Normal));
        scheduler.mark_hydrated("a");
        let pending = scheduler.pending_ids();
        assert_eq!(pending, vec!["b"]);
    }

    #[test]
    fn test_scheduler_hydrated_ids() {
        let scheduler = IncrementalHydrationScheduler::new();
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Critical));
        scheduler.schedule(HydrationTask::new("b", HydrationPriority::Normal));
        scheduler.mark_hydrated("a");
        let hydrated = scheduler.hydrated_ids();
        assert_eq!(hydrated, vec!["a"]);
    }

    #[test]
    fn test_scheduler_update_proximity() {
        let scheduler = IncrementalHydrationScheduler::new();
        scheduler.schedule(HydrationTask::new("a", HydrationPriority::Background));
        scheduler.update_proximity("a", 0.05);
        let tasks = scheduler.tasks.lock().unwrap();
        let task = tasks.get("a").unwrap();
        assert_eq!(task.priority, HydrationPriority::Critical);
    }

    #[test]
    fn test_scheduler_generate_idle_script() {
        let scheduler = IncrementalHydrationScheduler::new().with_batch_size(5);
        let script = scheduler.generate_idle_script();
        assert!(script.contains("batch:5"));
        assert!(script.contains("requestIdleCallback"));
    }

    #[test]
    fn test_scheduler_empty_progress() {
        let scheduler = IncrementalHydrationScheduler::new();
        assert_eq!(scheduler.progress(), 1.0);
    }
}
