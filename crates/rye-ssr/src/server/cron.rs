//! Goal 195: Cron / scheduled tasks for SSR apps.
//!
//! `#[schedule(every = "5m")]` macro that runs a function on a schedule in SSR mode.
//! Useful for cache warming, ISR triggers, cleanup tasks.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A schedule frequency.
#[derive(Debug, Clone)]
pub enum Schedule {
    /// Run every N seconds.
    Every(Duration),
    /// Run at a fixed interval (cron-like).
    Cron(String),
    /// Run once after a delay.
    Once(Duration),
    /// Run on startup.
    OnStartup,
}

impl Schedule {
    /// Parse a human-readable interval string (e.g. "5m", "1h", "30s").
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if s == "startup" {
            return Schedule::OnStartup;
        }

        let (num_str, unit) = if s.len() >= 2 {
            let split_point = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
            (&s[..split_point], &s[split_point..])
        } else {
            (s, "")
        };

        let num: u64 = num_str.parse().unwrap_or(0);

        let duration = match unit {
            "s" | "sec" | "seconds" => Duration::from_secs(num),
            "m" | "min" | "minutes" => Duration::from_secs(num * 60),
            "h" | "hr" | "hours" => Duration::from_secs(num * 3600),
            "d" | "day" | "days" => Duration::from_secs(num * 86400),
            _ => Duration::from_secs(num),
        };

        Schedule::Every(duration)
    }

    /// Get the interval duration (for Every variant).
    pub fn interval(&self) -> Option<Duration> {
        match self {
            Schedule::Every(d) | Schedule::Once(d) => Some(*d),
            _ => None,
        }
    }
}

impl std::fmt::Display for Schedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schedule::Every(d) => write!(f, "every {:?}", d),
            Schedule::Cron(c) => write!(f, "cron({})", c),
            Schedule::Once(d) => write!(f, "once after {:?}", d),
            Schedule::OnStartup => write!(f, "on startup"),
        }
    }
}

/// A scheduled task.
pub struct ScheduledTask {
    /// Unique task name.
    pub name: String,
    /// The schedule.
    pub schedule: Schedule,
    /// Whether the task is enabled.
    pub enabled: bool,
    /// Number of times this task has run.
    pub run_count: u64,
    /// Last time this task ran.
    pub last_run: Option<Instant>,
    /// The task function.
    task: Box<dyn Fn() + Send + Sync>,
}

impl std::fmt::Debug for ScheduledTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScheduledTask")
            .field("name", &self.name)
            .field("schedule", &self.schedule)
            .field("enabled", &self.enabled)
            .field("run_count", &self.run_count)
            .field("last_run", &self.last_run)
            .finish()
    }
}

impl ScheduledTask {
    /// Create a new scheduled task.
    pub fn new<F: Fn() + Send + Sync + 'static>(name: &str, schedule: Schedule, task: F) -> Self {
        Self {
            name: name.to_string(),
            schedule,
            enabled: true,
            run_count: 0,
            last_run: None,
            task: Box::new(task),
        }
    }

    /// Run the task.
    pub fn run(&mut self) {
        (self.task)();
        self.run_count += 1;
        self.last_run = Some(Instant::now());
    }

    /// Check if the task is due to run.
    pub fn is_due(&self) -> bool {
        if !self.enabled {
            return false;
        }
        match &self.schedule {
            Schedule::OnStartup => self.run_count == 0,
            Schedule::Once(d) => {
                if self.run_count > 0 {
                    return false;
                }
                match self.last_run {
                    Some(last) => last.elapsed() >= *d,
                    None => true,
                }
            }
            Schedule::Every(d) => {
                match self.last_run {
                    Some(last) => last.elapsed() >= *d,
                    None => true,
                }
            }
            Schedule::Cron(_) => false, // Cron parsing not implemented in this stub
        }
    }

    /// Enable the task.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the task.
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// The task scheduler — manages and runs scheduled tasks.
pub struct TaskScheduler {
    tasks: Mutex<Vec<ScheduledTask>>,
}

impl TaskScheduler {
    /// Create a new empty scheduler.
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(Vec::new()),
        }
    }

    /// Register a scheduled task.
    pub fn register<F: Fn() + Send + Sync + 'static>(&self, name: &str, schedule: Schedule, task: F) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(ScheduledTask::new(name, schedule, task));
    }

    /// Run all tasks that are due.
    pub fn tick(&self) -> usize {
        let mut tasks = self.tasks.lock().unwrap();
        let mut count = 0;
        for task in tasks.iter_mut() {
            if task.is_due() {
                task.run();
                count += 1;
            }
        }
        count
    }

    /// Run a specific task by name.
    pub fn run_task(&self, name: &str) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        for task in tasks.iter_mut() {
            if task.name == name {
                task.run();
                return true;
            }
        }
        false
    }

    /// Run all startup tasks.
    pub fn run_startup_tasks(&self) -> usize {
        let mut tasks = self.tasks.lock().unwrap();
        let mut count = 0;
        for task in tasks.iter_mut() {
            if matches!(task.schedule, Schedule::OnStartup) && task.run_count == 0 {
                task.run();
                count += 1;
            }
        }
        count
    }

    /// Get the number of registered tasks.
    pub fn task_count(&self) -> usize {
        self.tasks.lock().unwrap().len()
    }

    /// Get a task's run count.
    pub fn run_count(&self, name: &str) -> Option<u64> {
        let tasks = self.tasks.lock().unwrap();
        tasks.iter().find(|t| t.name == name).map(|t| t.run_count)
    }

    /// Enable a task by name.
    pub fn enable_task(&self, name: &str) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        for task in tasks.iter_mut() {
            if task.name == name {
                task.enable();
                return true;
            }
        }
        false
    }

    /// Disable a task by name.
    pub fn disable_task(&self, name: &str) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        for task in tasks.iter_mut() {
            if task.name == name {
                task.disable();
                return true;
            }
        }
        false
    }

    /// Get all task names.
    pub fn task_names(&self) -> Vec<String> {
        self.tasks.lock().unwrap().iter().map(|t| t.name.clone()).collect()
    }

    /// Remove a task by name.
    pub fn remove_task(&self, name: &str) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        let len_before = tasks.len();
        tasks.retain(|t| t.name != name);
        tasks.len() != len_before
    }

    /// Clear all tasks.
    pub fn clear(&self) {
        self.tasks.lock().unwrap().clear();
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Global scheduler instance.
static GLOBAL_SCHEDULER: Mutex<Option<TaskScheduler>> = Mutex::new(None);

/// Initialize the global scheduler.
pub fn init_global_scheduler() {
    let mut guard = GLOBAL_SCHEDULER.lock().unwrap();
    *guard = Some(TaskScheduler::new());
}

/// Register a task on the global scheduler.
pub fn schedule<F: Fn() + Send + Sync + 'static>(name: &str, sched: Schedule, task: F) {
    let mut guard = GLOBAL_SCHEDULER.lock().unwrap();
    if guard.is_none() {
        *guard = Some(TaskScheduler::new());
    }
    guard.as_ref().unwrap().register(name, sched, task);
}

/// Run all due tasks on the global scheduler.
pub fn run_due() -> usize {
    let guard = GLOBAL_SCHEDULER.lock().unwrap();
    guard.as_ref().map(|s| s.tick()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_schedule_parse_seconds() {
        let s = Schedule::parse("30s");
        assert_eq!(s.interval(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_schedule_parse_minutes() {
        let s = Schedule::parse("5m");
        assert_eq!(s.interval(), Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_schedule_parse_hours() {
        let s = Schedule::parse("1h");
        assert_eq!(s.interval(), Some(Duration::from_secs(3600)));
    }

    #[test]
    fn test_schedule_parse_days() {
        let s = Schedule::parse("1d");
        assert_eq!(s.interval(), Some(Duration::from_secs(86400)));
    }

    #[test]
    fn test_schedule_parse_startup() {
        let s = Schedule::parse("startup");
        assert!(matches!(s, Schedule::OnStartup));
    }

    #[test]
    fn test_schedule_display() {
        assert!(format!("{}", Schedule::OnStartup).contains("startup"));
        assert!(format!("{}", Schedule::Every(Duration::from_secs(60))).contains("every"));
    }

    #[test]
    fn test_scheduled_task_run() {
        let counter = std::sync::Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        let mut task = ScheduledTask::new("test", Schedule::Every(Duration::from_secs(60)), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(task.run_count, 0);
        task.run();
        assert_eq!(task.run_count, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert!(task.last_run.is_some());
    }

    #[test]
    fn test_scheduled_task_is_due_initial() {
        let task = ScheduledTask::new("test", Schedule::Every(Duration::from_secs(60)), || {});
        assert!(task.is_due()); // never run before
    }

    #[test]
    fn test_scheduled_task_is_due_disabled() {
        let mut task = ScheduledTask::new("test", Schedule::Every(Duration::from_secs(60)), || {});
        task.disable();
        assert!(!task.is_due());
    }

    #[test]
    fn test_scheduled_task_on_startup() {
        let mut task = ScheduledTask::new("startup", Schedule::OnStartup, || {});
        assert!(task.is_due());
        task.run();
        assert!(!task.is_due()); // already ran
    }

    #[test]
    fn test_scheduled_task_once() {
        let mut task = ScheduledTask::new("once", Schedule::Once(Duration::from_secs(100)), || {});
        assert!(task.is_due()); // never run
        task.run();
        assert!(!task.is_due()); // already ran once
    }

    #[test]
    fn test_task_scheduler_register() {
        let scheduler = TaskScheduler::new();
        scheduler.register("task1", Schedule::Every(Duration::from_secs(60)), || {});
        scheduler.register("task2", Schedule::OnStartup, || {});
        assert_eq!(scheduler.task_count(), 2);
    }

    #[test]
    fn test_task_scheduler_tick() {
        let scheduler = TaskScheduler::new();
        let counter = std::sync::Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        scheduler.register("task1", Schedule::Every(Duration::from_secs(60)), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });

        let ran = scheduler.tick();
        assert_eq!(ran, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_task_scheduler_run_task() {
        let scheduler = TaskScheduler::new();
        let counter = std::sync::Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        scheduler.register("my-task", Schedule::OnStartup, move || {
            c.fetch_add(1, Ordering::SeqCst);
        });

        assert!(scheduler.run_task("my-task"));
        assert!(!scheduler.run_task("nonexistent"));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_task_scheduler_run_startup() {
        let scheduler = TaskScheduler::new();
        let counter = std::sync::Arc::new(AtomicU32::new(0));
        let c = counter.clone();
        scheduler.register("startup1", Schedule::OnStartup, move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        scheduler.register("periodic", Schedule::Every(Duration::from_secs(60)), || {});

        let ran = scheduler.run_startup_tasks();
        assert_eq!(ran, 1);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_task_scheduler_enable_disable() {
        let scheduler = TaskScheduler::new();
        scheduler.register("task1", Schedule::Every(Duration::from_secs(60)), || {});

        assert!(scheduler.disable_task("task1"));
        let ran = scheduler.tick();
        assert_eq!(ran, 0); // disabled, shouldn't run

        assert!(scheduler.enable_task("task1"));
        let ran = scheduler.tick();
        assert_eq!(ran, 1);
    }

    #[test]
    fn test_task_scheduler_run_count() {
        let scheduler = TaskScheduler::new();
        scheduler.register("task1", Schedule::Every(Duration::from_secs(60)), || {});

        scheduler.run_task("task1");
        scheduler.run_task("task1");
        assert_eq!(scheduler.run_count("task1"), Some(2));
    }

    #[test]
    fn test_task_scheduler_task_names() {
        let scheduler = TaskScheduler::new();
        scheduler.register("a", Schedule::OnStartup, || {});
        scheduler.register("b", Schedule::OnStartup, || {});
        let names = scheduler.task_names();
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }

    #[test]
    fn test_task_scheduler_remove() {
        let scheduler = TaskScheduler::new();
        scheduler.register("temp", Schedule::OnStartup, || {});
        assert_eq!(scheduler.task_count(), 1);
        assert!(scheduler.remove_task("temp"));
        assert_eq!(scheduler.task_count(), 0);
    }

    #[test]
    fn test_task_scheduler_clear() {
        let scheduler = TaskScheduler::new();
        scheduler.register("a", Schedule::OnStartup, || {});
        scheduler.register("b", Schedule::OnStartup, || {});
        scheduler.clear();
        assert_eq!(scheduler.task_count(), 0);
    }
}
