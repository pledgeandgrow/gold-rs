//! Saga — long-running, multi-step async operations with compensating actions.
//!
//! Each step can roll back on failure. Useful for checkout flows,
//! multi-form wizards, data migration UIs.

use std::cell::RefCell;
use std::rc::Rc;

/// The outcome of a saga step.
#[derive(Debug, Clone)]
pub enum StepResult<T> {
    /// Step succeeded with a result.
    Success(T),
    /// Step failed with an error message.
    Failure(String),
}

/// The overall state of a saga.
#[derive(Debug, Clone, PartialEq)]
pub enum SagaState {
    /// Not started yet.
    Pending,
    /// Currently executing a step.
    Running(usize),
    /// All steps completed successfully.
    Completed,
    /// Failed at the given step index, compensation was run.
    Failed { step: usize, error: String },
    /// Failed and compensation also failed.
    CompensationFailed {
        step: usize,
        error: String,
        comp_error: String,
    },
}

impl SagaState {
    pub fn is_done(&self) -> bool {
        matches!(
            self,
            SagaState::Completed | SagaState::Failed { .. } | SagaState::CompensationFailed { .. }
        )
    }

    pub fn is_success(&self) -> bool {
        matches!(self, SagaState::Completed)
    }
}

/// A saga step with its execution and compensation functions.
pub struct SagaStep<T, E> {
    pub name: String,
    pub execute: Box<dyn Fn() -> StepResult<T>>,
    pub compensate: Box<dyn Fn() -> Result<(), E>>,
}

/// A saga — a sequence of steps with compensating actions.
pub struct Saga<T, E: std::fmt::Display> {
    steps: Vec<SagaStep<T, E>>,
    state: Rc<RefCell<SagaState>>,
    results: Rc<RefCell<Vec<T>>>,
}

impl<T: Clone + 'static, E: std::fmt::Display + 'static> Saga<T, E> {
    /// Create a new empty saga.
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            state: Rc::new(RefCell::new(SagaState::Pending)),
            results: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Add a step to the saga.
    pub fn step<F, C>(mut self, name: &str, execute: F, compensate: C) -> Self
    where
        F: Fn() -> StepResult<T> + 'static,
        C: Fn() -> Result<(), E> + 'static,
    {
        self.steps.push(SagaStep {
            name: name.to_string(),
            execute: Box::new(execute),
            compensate: Box::new(compensate),
        });
        self
    }

    /// Run the saga synchronously. If any step fails, all previously
    /// completed steps are compensated in reverse order.
    pub fn run(&self) -> SagaState {
        let mut completed = 0usize;
        self.results.borrow_mut().clear();

        for (i, step) in self.steps.iter().enumerate() {
            *self.state.borrow_mut() = SagaState::Running(i);

            match (step.execute)() {
                StepResult::Success(value) => {
                    self.results.borrow_mut().push(value);
                    completed = i + 1;
                }
                StepResult::Failure(err) => {
                    // Compensate in reverse order
                    for j in (0..completed).rev() {
                        if let Err(comp_err) = (self.steps[j].compensate)() {
                            *self.state.borrow_mut() = SagaState::CompensationFailed {
                                step: i,
                                error: err.clone(),
                                comp_error: comp_err.to_string(),
                            };
                            return self.state.borrow().clone();
                        }
                    }
                    *self.state.borrow_mut() = SagaState::Failed {
                        step: i,
                        error: err,
                    };
                    return self.state.borrow().clone();
                }
            }
        }

        *self.state.borrow_mut() = SagaState::Completed;
        self.state.borrow().clone()
    }

    /// Get the current state.
    pub fn state(&self) -> SagaState {
        self.state.borrow().clone()
    }

    /// Get the results of completed steps.
    pub fn results(&self) -> Vec<T> {
        self.results.borrow().clone()
    }

    /// Get the number of steps.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get step names.
    pub fn step_names(&self) -> Vec<String> {
        self.steps.iter().map(|s| s.name.clone()).collect()
    }
}

impl<T: Clone + 'static, E: std::fmt::Display + 'static> Default for Saga<T, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating sagas with async steps.
///
/// Since the runtime is single-threaded, async steps are polled synchronously.
/// For real async execution, use within an async runtime.
pub struct SagaBuilder<T, E: std::fmt::Display + 'static> {
    saga: Saga<T, E>,
}

impl<T: Clone + 'static, E: std::fmt::Display + 'static> SagaBuilder<T, E> {
    pub fn new() -> Self {
        Self { saga: Saga::new() }
    }

    pub fn add_step<F, C>(mut self, name: &str, execute: F, compensate: C) -> Self
    where
        F: Fn() -> StepResult<T> + 'static,
        C: Fn() -> Result<(), E> + 'static,
    {
        self.saga = self.saga.step(name, execute, compensate);
        self
    }

    pub fn build(self) -> Saga<T, E> {
        self.saga
    }
}

impl<T: Clone + 'static, E: std::fmt::Display + 'static> Default for SagaBuilder<T, E> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_saga_all_success() {
        let saga = Saga::<i32, String>::new()
            .step("step1", || StepResult::Success(1), || Ok(()))
            .step("step2", || StepResult::Success(2), || Ok(()))
            .step("step3", || StepResult::Success(3), || Ok(()));

        let state = saga.run();
        assert_eq!(state, SagaState::Completed);
        assert_eq!(saga.results(), vec![1, 2, 3]);
    }

    #[test]
    fn test_saga_failure_compensates() {
        let compensated = Rc::new(Cell::new(false));
        let comp_clone = Rc::clone(&compensated);

        let saga = Saga::<i32, String>::new()
            .step(
                "step1",
                || StepResult::Success(1),
                move || {
                    comp_clone.set(true);
                    Ok(())
                },
            )
            .step(
                "step2",
                || StepResult::Failure("oops".to_string()),
                || Ok(()),
            );

        let state = saga.run();
        assert!(matches!(state, SagaState::Failed { step: 1, .. }));
        assert!(compensated.get());
    }

    #[test]
    fn test_saga_compensation_failure() {
        let saga = Saga::<i32, String>::new()
            .step(
                "step1",
                || StepResult::Success(1),
                || Err("comp failed".to_string()),
            )
            .step(
                "step2",
                || StepResult::Failure("step failed".to_string()),
                || Ok(()),
            );

        let state = saga.run();
        assert!(matches!(state, SagaState::CompensationFailed { .. }));
    }

    #[test]
    fn test_saga_empty() {
        let saga = Saga::<i32, String>::new();
        let state = saga.run();
        assert_eq!(state, SagaState::Completed);
        assert_eq!(saga.results(), Vec::<i32>::new());
    }

    #[test]
    fn test_saga_step_names() {
        let saga = Saga::<i32, String>::new()
            .step("create_order", || StepResult::Success(1), || Ok(()))
            .step("charge_card", || StepResult::Success(2), || Ok(()))
            .step("ship", || StepResult::Success(3), || Ok(()));

        assert_eq!(
            saga.step_names(),
            vec!["create_order", "charge_card", "ship"]
        );
        assert_eq!(saga.step_count(), 3);
    }

    #[test]
    fn test_saga_builder() {
        let saga = SagaBuilder::<i32, String>::new()
            .add_step("a", || StepResult::Success(10), || Ok(()))
            .add_step("b", || StepResult::Success(20), || Ok(()))
            .build();

        let state = saga.run();
        assert_eq!(state, SagaState::Completed);
        assert_eq!(saga.results(), vec![10, 20]);
    }

    #[test]
    fn test_saga_state_is_done() {
        let saga = Saga::<i32, String>::new().step("step1", || StepResult::Success(1), || Ok(()));

        assert!(!saga.state().is_done());
        saga.run();
        assert!(saga.state().is_done());
        assert!(saga.state().is_success());
    }

    #[test]
    fn test_saga_reverse_compensation_order() {
        let order = Rc::new(RefCell::new(Vec::new()));

        let order1 = Rc::clone(&order);
        let order2 = Rc::clone(&order);
        let order3 = Rc::clone(&order);

        let comp1 = Rc::clone(&order);
        let comp2 = Rc::clone(&order);

        let saga = Saga::<i32, String>::new()
            .step(
                "s1",
                move || {
                    order1.borrow_mut().push("s1");
                    StepResult::Success(1)
                },
                move || {
                    comp1.borrow_mut().push("c1");
                    Ok(())
                },
            )
            .step(
                "s2",
                move || {
                    order2.borrow_mut().push("s2");
                    StepResult::Success(2)
                },
                move || {
                    comp2.borrow_mut().push("c2");
                    Ok(())
                },
            )
            .step(
                "s3",
                move || {
                    order3.borrow_mut().push("s3");
                    StepResult::Failure("fail".to_string())
                },
                || Ok(()),
            );

        saga.run();

        // Compensation should be in reverse order: c2, c1
        assert_eq!(*order.borrow(), vec!["s1", "s2", "s3", "c2", "c1"]);
    }
}
