//! Framework-free domain types and rules for the local stack proof.

mod computation;
pub use computation::calculate;

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunId(String);

impl RunId {
    /// Creates a run identifier.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidRunId`] when the value is empty or whitespace-only.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::InvalidRunId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunConfiguration {
    pub seed: u64,
    pub region: String,
    pub threshold: f64,
    pub force_validation_failure: bool,
}

impl RunConfiguration {
    /// Creates a validated configuration snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidRegion`] for an empty region and
    /// [`DomainError::InvalidThreshold`] for a non-finite threshold.
    pub fn new(
        seed: u64,
        region: impl Into<String>,
        threshold: f64,
        force_validation_failure: bool,
    ) -> Result<Self, DomainError> {
        let region = region.into();
        if region.trim().is_empty() {
            return Err(DomainError::InvalidRegion);
        }
        if !threshold.is_finite() {
            return Err(DomainError::InvalidThreshold);
        }

        Ok(Self {
            seed,
            region,
            threshold,
            force_validation_failure,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Queued,
    Running,
    Completed,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Created,
    Started,
    Completed,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunEvent {
    pub kind: EventKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunResult {
    pub total_records: u32,
    pub matched_records: u32,
    pub total_score: f64,
    pub average_score: f64,
    pub groups: Vec<GroupSummary>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupSummary {
    pub category: String,
    pub matched_records: u32,
    pub total_score: f64,
    pub average_score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationMessage {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    id: RunId,
    configuration: RunConfiguration,
    status: RunStatus,
    result: Option<RunResult>,
    validation_messages: Vec<ValidationMessage>,
    events: Vec<RunEvent>,
}

impl Run {
    #[must_use]
    pub fn new(id: RunId, configuration: RunConfiguration) -> Self {
        Self {
            id,
            configuration,
            status: RunStatus::Queued,
            result: None,
            validation_messages: Vec::new(),
            events: vec![RunEvent {
                kind: EventKind::Created,
                message: "Run created".to_owned(),
            }],
        }
    }

    #[must_use]
    pub fn id(&self) -> &RunId {
        &self.id
    }

    #[must_use]
    pub fn configuration(&self) -> &RunConfiguration {
        &self.configuration
    }

    #[must_use]
    pub fn status(&self) -> RunStatus {
        self.status
    }

    #[must_use]
    pub fn result(&self) -> Option<&RunResult> {
        self.result.as_ref()
    }

    #[must_use]
    pub fn validation_messages(&self) -> &[ValidationMessage] {
        &self.validation_messages
    }

    #[must_use]
    pub fn events(&self) -> &[RunEvent] {
        &self.events
    }

    /// Moves a queued run into the running state.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidTransition`] unless the run is queued.
    pub fn start(&mut self) -> Result<(), DomainError> {
        if self.status != RunStatus::Queued {
            return Err(DomainError::InvalidTransition {
                from: self.status,
                action: "start",
            });
        }
        self.status = RunStatus::Running;
        self.events.push(RunEvent {
            kind: EventKind::Started,
            message: "Run started".to_owned(),
        });
        Ok(())
    }

    /// Completes a running run with its result.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidTransition`] unless the run is running.
    pub fn complete(&mut self, result: RunResult) -> Result<(), DomainError> {
        if self.status != RunStatus::Running {
            return Err(DomainError::InvalidTransition {
                from: self.status,
                action: "complete",
            });
        }
        self.result = Some(result);
        self.status = RunStatus::Completed;
        self.events.push(RunEvent {
            kind: EventKind::Completed,
            message: "Run completed".to_owned(),
        });
        Ok(())
    }

    /// Rejects a running run with one or more validation messages.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidTransition`] unless the run is running, or
    /// [`DomainError::MissingValidationMessage`] when no message is supplied.
    pub fn reject(&mut self, messages: Vec<ValidationMessage>) -> Result<(), DomainError> {
        if self.status != RunStatus::Running {
            return Err(DomainError::InvalidTransition {
                from: self.status,
                action: "reject",
            });
        }
        if messages.is_empty() {
            return Err(DomainError::MissingValidationMessage);
        }
        self.validation_messages = messages;
        self.status = RunStatus::Rejected;
        self.events.push(RunEvent {
            kind: EventKind::Rejected,
            message: "Run rejected by validation".to_owned(),
        });
        Ok(())
    }

    /// Marks a running run as failed with a diagnostic message.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidTransition`] unless the run is running.
    pub fn fail(&mut self, message: impl Into<String>) -> Result<(), DomainError> {
        if self.status != RunStatus::Running {
            return Err(DomainError::InvalidTransition {
                from: self.status,
                action: "fail",
            });
        }
        self.status = RunStatus::Failed;
        self.events.push(RunEvent {
            kind: EventKind::Failed,
            message: message.into(),
        });
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InvalidRunId,
    InvalidRegion,
    InvalidThreshold,
    MissingValidationMessage,
    InvalidTransition {
        from: RunStatus,
        action: &'static str,
    },
}

impl Display for DomainError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRunId => formatter.write_str("run ID cannot be empty"),
            Self::InvalidRegion => formatter.write_str("region cannot be empty"),
            Self::InvalidThreshold => formatter.write_str("threshold must be finite"),
            Self::MissingValidationMessage => {
                formatter.write_str("a rejected run must contain a validation message")
            }
            Self::InvalidTransition { from, action } => {
                write!(formatter, "cannot {action} a run in {from:?} status")
            }
        }
    }
}

impl std::error::Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn queued_run() -> Run {
        let id = RunId::new("run-1").expect("valid ID");
        let configuration =
            RunConfiguration::new(42, "north", 0.75, false).expect("valid configuration");
        Run::new(id, configuration)
    }

    #[test]
    fn new_run_is_queued_with_creation_event() {
        let run = queued_run();

        assert_eq!(run.status(), RunStatus::Queued);
        assert!(run.result().is_none());
        assert_eq!(run.events()[0].kind, EventKind::Created);
    }

    #[test]
    fn completed_result_requires_running_state() {
        let mut run = queued_run();
        let result = RunResult {
            total_records: 10,
            matched_records: 4,
            total_score: 12.0,
            average_score: 3.0,
            groups: Vec::new(),
        };

        assert!(run.complete(result.clone()).is_err());
        assert!(run.result().is_none());

        run.start().expect("queued run can start");
        run.complete(result).expect("running run can complete");
        assert_eq!(run.status(), RunStatus::Completed);
        assert!(run.result().is_some());
    }

    #[test]
    fn rejection_requires_a_validation_message() {
        let mut run = queued_run();
        run.start().expect("queued run can start");

        assert!(run.reject(Vec::new()).is_err());
        run.reject(vec![ValidationMessage {
            code: "threshold".to_owned(),
            message: "threshold was not met".to_owned(),
            field: Some("threshold".to_owned()),
        }])
        .expect("running run can be rejected");

        assert_eq!(run.status(), RunStatus::Rejected);
        assert_eq!(run.validation_messages().len(), 1);
        assert!(run.result().is_none());
    }

    #[test]
    fn terminal_runs_cannot_start_again() {
        let mut run = queued_run();
        run.start().expect("queued run can start");
        run.complete(RunResult {
            total_records: 1,
            matched_records: 1,
            total_score: 1.0,
            average_score: 1.0,
            groups: Vec::new(),
        })
        .expect("running run can complete");

        assert!(run.start().is_err());
    }
}
