//! Goal 206: Native background tasks.
//!
//! Background fetch, background processing, and background sync.
//! `#[background_task]` macro. iOS (BGTaskScheduler), Android (WorkManager).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The type of background task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundTaskType {
    /// Brief background fetch (iOS BGAppRefreshTask, ~30 seconds).
    BackgroundFetch,
    /// Longer background processing (iOS BGProcessingTask, can run minutes).
    BackgroundProcessing,
    /// Background sync (web Service Worker Background Sync).
    BackgroundSync,
}

impl BackgroundTaskType {
    /// Get the default timeout for this task type.
    pub fn default_timeout(&self) -> Duration {
        match self {
            BackgroundTaskType::BackgroundFetch => Duration::from_secs(30),
            BackgroundTaskType::BackgroundProcessing => Duration::from_secs(180),
            BackgroundTaskType::BackgroundSync => Duration::from_secs(60),
        }
    }
}

/// The constraints for running a background task.
#[derive(Debug, Clone, Default)]
pub struct TaskConstraints {
    /// Require network connectivity.
    pub require_network: bool,
    /// require charging.
    pub require_charging: bool,
    /// require device idle.
    pub require_idle: bool,
    /// require battery not low.
    pub require_battery_not_low: bool,
    /// Minimum battery level required (0-100).
    pub min_battery_level: Option<u8>,
}

impl TaskConstraints {
    /// Create constraints requiring network.
    pub fn network() -> Self {
        Self {
            require_network: true,
            ..Default::default()
        }
    }

    /// Create constraints requiring network and charging.
    pub fn network_and_charging() -> Self {
        Self {
            require_network: true,
            require_charging: true,
            ..Default::default()
        }
    }
}

/// The state of a background task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is scheduled but not yet running.
    Scheduled,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed.
    Failed,
    /// Task was cancelled.
    Cancelled,
}

impl TaskState {
    /// Check if the task is finished.
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            TaskState::Completed | TaskState::Failed | TaskState::Cancelled
        )
    }

    /// Check if the task is active.
    pub fn is_active(&self) -> bool {
        matches!(self, TaskState::Scheduled | TaskState::Running)
    }
}

/// A background task definition.
pub struct BackgroundTask {
    /// The task identifier.
    pub id: String,
    /// The task type.
    pub task_type: BackgroundTaskType,
    /// The task constraints.
    pub constraints: TaskConstraints,
    /// The minimum interval between runs.
    pub min_interval: Duration,
    /// Whether the task requires external power to run.
    pub requires_external_power: bool,
    /// The task function.
    task: Box<dyn Fn() -> TaskOutcome + Send + Sync>,
}

impl BackgroundTask {
    /// Create a new background task.
    pub fn new<F: Fn() -> TaskOutcome + Send + Sync + 'static>(
        id: &str,
        task_type: BackgroundTaskType,
        task: F,
    ) -> Self {
        Self {
            id: id.to_string(),
            task_type,
            constraints: TaskConstraints::default(),
            min_interval: task_type.default_timeout(),
            requires_external_power: false,
            task: Box::new(task),
        }
    }

    /// Set constraints.
    pub fn with_constraints(mut self, constraints: TaskConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Set minimum interval.
    pub fn with_min_interval(mut self, interval: Duration) -> Self {
        self.min_interval = interval;
        self
    }

    /// Require external power.
    pub fn requires_power(mut self) -> Self {
        self.requires_external_power = true;
        self
    }

    /// Run the task.
    pub fn run(&self) -> TaskOutcome {
        (self.task)()
    }
}

impl std::fmt::Debug for BackgroundTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackgroundTask")
            .field("id", &self.id)
            .field("task_type", &self.task_type)
            .field("constraints", &self.constraints)
            .field("min_interval", &self.min_interval)
            .field("requires_external_power", &self.requires_external_power)
            .finish()
    }
}

/// The outcome of running a background task.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskOutcome {
    /// Task completed successfully with new data.
    NewData,
    /// Task completed but no new data.
    NoData,
    /// Task failed.
    Failed(String),
    /// Task needs to be rescheduled.
    Reschedule,
}

impl TaskOutcome {
    /// Check if the task succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, TaskOutcome::NewData | TaskOutcome::NoData)
    }
}

/// A scheduled background task instance.
#[derive(Debug)]
pub struct ScheduledTaskInstance {
    /// The task ID.
    pub task_id: String,
    /// The current state.
    pub state: TaskState,
    /// When the task was scheduled.
    pub scheduled_at: Instant,
    /// When the task started (if running/completed).
    pub started_at: Option<Instant>,
    /// When the task finished (if completed).
    pub finished_at: Option<Instant>,
    /// The result of the task.
    pub result: Option<TaskOutcome>,
    /// Number of times this task has been retried.
    pub retry_count: u32,
}

/// The background task scheduler.
pub struct BackgroundTaskScheduler {
    tasks: Mutex<HashMap<String, BackgroundTask>>,
    instances: Mutex<Vec<ScheduledTaskInstance>>,
}

impl BackgroundTaskScheduler {
    /// Create a new scheduler.
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            instances: Mutex::new(Vec::new()),
        }
    }

    /// Register a background task.
    pub fn register(&self, task: BackgroundTask) {
        self.tasks.lock().unwrap().insert(task.id.clone(), task);
    }

    /// Schedule a task by ID.
    pub fn schedule(&self, task_id: &str) -> bool {
        let tasks = self.tasks.lock().unwrap();
        if !tasks.contains_key(task_id) {
            return false;
        }
        drop(tasks);

        self.instances.lock().unwrap().push(ScheduledTaskInstance {
            task_id: task_id.to_string(),
            state: TaskState::Scheduled,
            scheduled_at: Instant::now(),
            started_at: None,
            finished_at: None,
            result: None,
            retry_count: 0,
        });
        true
    }

    /// Run all scheduled tasks that are ready.
    pub fn run_ready(&self) -> usize {
        let tasks = self.tasks.lock().unwrap();
        let mut instances = self.instances.lock().unwrap();
        let mut count = 0;

        for instance in instances.iter_mut() {
            if instance.state != TaskState::Scheduled {
                continue;
            }

            if let Some(task) = tasks.get(&instance.task_id) {
                instance.state = TaskState::Running;
                instance.started_at = Some(Instant::now());

                let outcome = task.run();

                instance.state = if outcome.is_success() {
                    TaskState::Completed
                } else {
                    TaskState::Failed
                };
                instance.finished_at = Some(Instant::now());
                instance.result = Some(outcome);
                count += 1;
            }
        }

        count
    }

    /// Cancel a scheduled task.
    pub fn cancel(&self, task_id: &str) -> bool {
        let mut instances = self.instances.lock().unwrap();
        for instance in instances.iter_mut() {
            if instance.task_id == task_id && instance.state.is_active() {
                instance.state = TaskState::Cancelled;
                instance.finished_at = Some(Instant::now());
                return true;
            }
        }
        false
    }

    /// Get the number of registered tasks.
    pub fn task_count(&self) -> usize {
        self.tasks.lock().unwrap().len()
    }

    /// Get the number of scheduled instances.
    pub fn scheduled_count(&self) -> usize {
        self.instances
            .lock()
            .unwrap()
            .iter()
            .filter(|i| i.state.is_active())
            .count()
    }

    /// Get the number of completed instances.
    pub fn completed_count(&self) -> usize {
        self.instances
            .lock()
            .unwrap()
            .iter()
            .filter(|i| i.state == TaskState::Completed)
            .count()
    }

    /// Get all task IDs.
    pub fn task_ids(&self) -> Vec<String> {
        self.tasks.lock().unwrap().keys().cloned().collect()
    }

    /// Remove a task registration.
    pub fn unregister(&self, task_id: &str) -> bool {
        self.tasks.lock().unwrap().remove(task_id).is_some()
    }
}

impl Default for BackgroundTaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_task_type_default_timeout() {
        assert_eq!(
            BackgroundTaskType::BackgroundFetch.default_timeout(),
            Duration::from_secs(30)
        );
        assert_eq!(
            BackgroundTaskType::BackgroundProcessing.default_timeout(),
            Duration::from_secs(180)
        );
        assert_eq!(
            BackgroundTaskType::BackgroundSync.default_timeout(),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn test_task_constraints_network() {
        let c = TaskConstraints::network();
        assert!(c.require_network);
        assert!(!c.require_charging);
    }

    #[test]
    fn test_task_constraints_network_and_charging() {
        let c = TaskConstraints::network_and_charging();
        assert!(c.require_network);
        assert!(c.require_charging);
    }

    #[test]
    fn test_task_state_is_finished() {
        assert!(TaskState::Completed.is_finished());
        assert!(TaskState::Failed.is_finished());
        assert!(TaskState::Cancelled.is_finished());
        assert!(!TaskState::Scheduled.is_finished());
        assert!(!TaskState::Running.is_finished());
    }

    #[test]
    fn test_task_state_is_active() {
        assert!(TaskState::Scheduled.is_active());
        assert!(TaskState::Running.is_active());
        assert!(!TaskState::Completed.is_active());
    }

    #[test]
    fn test_background_task_new() {
        let task = BackgroundTask::new("fetch", BackgroundTaskType::BackgroundFetch, || {
            TaskOutcome::NewData
        });
        assert_eq!(task.id, "fetch");
        assert_eq!(task.task_type, BackgroundTaskType::BackgroundFetch);
    }

    #[test]
    fn test_background_task_builder() {
        let task = BackgroundTask::new("sync", BackgroundTaskType::BackgroundSync, || {
            TaskOutcome::NewData
        })
        .with_constraints(TaskConstraints::network())
        .with_min_interval(Duration::from_secs(120))
        .requires_power();

        assert!(task.constraints.require_network);
        assert_eq!(task.min_interval, Duration::from_secs(120));
        assert!(task.requires_external_power);
    }

    #[test]
    fn test_background_task_run() {
        let task = BackgroundTask::new("test", BackgroundTaskType::BackgroundFetch, || {
            TaskOutcome::NewData
        });
        assert_eq!(task.run(), TaskOutcome::NewData);
    }

    #[test]
    fn test_task_outcome_is_success() {
        assert!(TaskOutcome::NewData.is_success());
        assert!(TaskOutcome::NoData.is_success());
        assert!(!TaskOutcome::Failed("err".to_string()).is_success());
        assert!(!TaskOutcome::Reschedule.is_success());
    }

    #[test]
    fn test_scheduler_register() {
        let scheduler = BackgroundTaskScheduler::new();
        scheduler.register(BackgroundTask::new(
            "t1",
            BackgroundTaskType::BackgroundFetch,
            || TaskOutcome::NewData,
        ));
        assert_eq!(scheduler.task_count(), 1);
    }

    #[test]
    fn test_scheduler_schedule() {
        let scheduler = BackgroundTaskScheduler::new();
        scheduler.register(BackgroundTask::new(
            "t1",
            BackgroundTaskType::BackgroundFetch,
            || TaskOutcome::NewData,
        ));
        assert!(scheduler.schedule("t1"));
        assert!(!scheduler.schedule("nonexistent"));
        assert_eq!(scheduler.scheduled_count(), 1);
    }

    #[test]
    fn test_scheduler_run_ready() {
        let counter = std::sync::Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        let scheduler = BackgroundTaskScheduler::new();
        scheduler.register(BackgroundTask::new(
            "t1",
            BackgroundTaskType::BackgroundFetch,
            move || {
                c.fetch_add(1, Ordering::SeqCst);
                TaskOutcome::NewData
            },
        ));
        scheduler.schedule("t1");

        let ran = scheduler.run_ready();
        assert_eq!(ran, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(scheduler.completed_count(), 1);
        assert_eq!(scheduler.scheduled_count(), 0);
    }

    #[test]
    fn test_scheduler_cancel() {
        let scheduler = BackgroundTaskScheduler::new();
        scheduler.register(BackgroundTask::new(
            "t1",
            BackgroundTaskType::BackgroundFetch,
            || TaskOutcome::NewData,
        ));
        scheduler.schedule("t1");
        assert!(scheduler.cancel("t1"));
        assert_eq!(scheduler.scheduled_count(), 0);
    }

    #[test]
    fn test_scheduler_task_ids() {
        let scheduler = BackgroundTaskScheduler::new();
        scheduler.register(BackgroundTask::new(
            "a",
            BackgroundTaskType::BackgroundFetch,
            || TaskOutcome::NoData,
        ));
        scheduler.register(BackgroundTask::new(
            "b",
            BackgroundTaskType::BackgroundSync,
            || TaskOutcome::NoData,
        ));
        let ids = scheduler.task_ids();
        assert!(ids.contains(&"a".to_string()));
        assert!(ids.contains(&"b".to_string()));
    }

    #[test]
    fn test_scheduler_unregister() {
        let scheduler = BackgroundTaskScheduler::new();
        scheduler.register(BackgroundTask::new(
            "temp",
            BackgroundTaskType::BackgroundFetch,
            || TaskOutcome::NoData,
        ));
        assert!(scheduler.unregister("temp"));
        assert_eq!(scheduler.task_count(), 0);
    }

    #[test]
    fn test_scheduler_run_ready_failed() {
        let scheduler = BackgroundTaskScheduler::new();
        scheduler.register(BackgroundTask::new(
            "fail",
            BackgroundTaskType::BackgroundProcessing,
            || TaskOutcome::Failed("error".to_string()),
        ));
        scheduler.schedule("fail");
        scheduler.run_ready();
        assert_eq!(scheduler.completed_count(), 0);
    }
}
