//! Framework-independent run use-cases and the repository port.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use local_stack_proof_domain::{DomainError, Run, RunConfiguration, RunId, calculate};

/// Storage operations needed by run use-cases.
pub trait RunRepository: Send + Sync {
    /// Inserts a new run and its creation event.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::AlreadyExists`] for a duplicate identifier or
    /// [`RepositoryError::Unavailable`] when storage cannot be accessed.
    fn insert(&self, run: &Run) -> Result<(), RepositoryError>;

    /// Replaces the saved snapshot for an existing run.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] if no run exists,
    /// [`RepositoryError::Conflict`] for a stale or invalid snapshot, or
    /// [`RepositoryError::Unavailable`] when storage cannot be accessed.
    fn save(&self, run: &Run) -> Result<(), RepositoryError>;

    /// Retrieves a run, or returns `None` if it is unknown.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Unavailable`] when storage cannot be accessed.
    fn get(&self, id: &RunId) -> Result<Option<Run>, RepositoryError>;

    /// Lists runs in stable identifier order.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Unavailable`] when storage cannot be accessed.
    fn list(&self) -> Result<Vec<Run>, RepositoryError>;
}

/// Supplies identifiers without coupling application behavior to a clock or host.
pub trait RunIdGenerator: Send + Sync {
    /// Creates the next identifier.
    ///
    /// # Errors
    ///
    /// Returns a domain error for an invalid identifier or a repository error
    /// when a persistent identifier source is unavailable.
    fn next_id(&self) -> Result<RunId, ServiceError>;
}

/// Monotonic, process-local identifiers for the in-memory proof stage.
#[derive(Debug)]
pub struct SequentialRunIdGenerator {
    next: AtomicU64,
}

impl Default for SequentialRunIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SequentialRunIdGenerator {
    /// Starts a generator at `run-000001`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }
}

impl RunIdGenerator for SequentialRunIdGenerator {
    fn next_id(&self) -> Result<RunId, ServiceError> {
        Ok(RunId::new(format!(
            "run-{:06}",
            self.next.fetch_add(1, Ordering::Relaxed)
        ))?)
    }
}

/// In-memory repository used by the first web slice and service tests.
#[derive(Debug, Default)]
pub struct InMemoryRunRepository {
    runs: Mutex<BTreeMap<String, Run>>,
}

impl RunRepository for InMemoryRunRepository {
    fn insert(&self, run: &Run) -> Result<(), RepositoryError> {
        let mut runs = self.runs.lock().map_err(|_| RepositoryError::Unavailable)?;
        let key = run.id().as_str().to_owned();
        if runs.contains_key(&key) {
            return Err(RepositoryError::AlreadyExists(key));
        }
        runs.insert(key, run.clone());
        Ok(())
    }

    fn save(&self, run: &Run) -> Result<(), RepositoryError> {
        let mut runs = self.runs.lock().map_err(|_| RepositoryError::Unavailable)?;
        let key = run.id().as_str().to_owned();
        let Some(saved) = runs.get_mut(&key) else {
            return Err(RepositoryError::NotFound(key));
        };
        if saved.configuration() != run.configuration()
            || run.events().len() != saved.events().len() + 1
            || !run.events().starts_with(saved.events())
        {
            return Err(RepositoryError::Conflict(key));
        }
        *saved = run.clone();
        Ok(())
    }

    fn get(&self, id: &RunId) -> Result<Option<Run>, RepositoryError> {
        let runs = self.runs.lock().map_err(|_| RepositoryError::Unavailable)?;
        Ok(runs.get(id.as_str()).cloned())
    }

    fn list(&self) -> Result<Vec<Run>, RepositoryError> {
        let runs = self.runs.lock().map_err(|_| RepositoryError::Unavailable)?;
        Ok(runs.values().cloned().collect())
    }
}

/// Application-level failure categories, independent of any transport framework.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    Domain(DomainError),
    Repository(RepositoryError),
    NotFound(String),
}

impl Display for ServiceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(error) => Display::fmt(error, formatter),
            Self::Repository(error) => Display::fmt(error, formatter),
            Self::NotFound(id) => write!(formatter, "run '{id}' was not found"),
        }
    }
}

impl Error for ServiceError {}

impl From<DomainError> for ServiceError {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

impl From<RepositoryError> for ServiceError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

/// Failure category returned by the repository port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    AlreadyExists(String),
    NotFound(String),
    Conflict(String),
    Unavailable,
}

impl Display for RepositoryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyExists(id) => write!(formatter, "run '{id}' already exists"),
            Self::NotFound(id) => write!(formatter, "run '{id}' was not found"),
            Self::Conflict(id) => write!(formatter, "run '{id}' changed; reload before retrying"),
            Self::Unavailable => formatter.write_str("run repository is unavailable"),
        }
    }
}

impl Error for RepositoryError {}

/// Coordinates run lifecycle use-cases through framework-independent ports.
#[derive(Clone)]
pub struct RunService {
    repository: Arc<dyn RunRepository>,
    id_generator: Arc<dyn RunIdGenerator>,
}

impl RunService {
    /// Creates a service over the supplied repository and identifier source.
    #[must_use]
    pub fn new(repository: Arc<dyn RunRepository>, id_generator: Arc<dyn RunIdGenerator>) -> Self {
        Self {
            repository,
            id_generator,
        }
    }

    /// Creates and stores a queued run.
    ///
    /// # Errors
    ///
    /// Returns [`ServiceError::Domain`] if ID generation fails or
    /// [`ServiceError::Repository`] if the run cannot be stored.
    pub fn create(&self, configuration: RunConfiguration) -> Result<Run, ServiceError> {
        let run = Run::new(self.id_generator.next_id()?, configuration);
        self.repository.insert(&run)?;
        Ok(run)
    }

    /// Executes a queued run and stores every lifecycle transition.
    ///
    /// A business validation rejection is a successful use-case outcome represented by a
    /// `Rejected` run; repository and invalid-transition failures are returned as errors.
    ///
    /// # Errors
    ///
    /// Returns [`ServiceError::NotFound`] for an unknown ID,
    /// [`ServiceError::Domain`] for an invalid lifecycle transition, or
    /// [`ServiceError::Repository`] if any lifecycle snapshot cannot be stored.
    pub fn execute(&self, id: &RunId) -> Result<Run, ServiceError> {
        let mut run = self.load(id)?;
        run.start()?;
        self.repository.save(&run)?;

        match calculate(run.configuration()) {
            Ok(result) => run.complete(result)?,
            Err(messages) => run.reject(messages)?,
        }
        self.repository.save(&run)?;
        Ok(run)
    }

    /// Retrieves one saved run.
    ///
    /// # Errors
    ///
    /// Returns [`ServiceError::NotFound`] for an unknown ID or
    /// [`ServiceError::Repository`] if storage cannot be accessed.
    pub fn get(&self, id: &RunId) -> Result<Run, ServiceError> {
        self.load(id)
    }

    /// Lists saved runs in stable identifier order.
    ///
    /// # Errors
    ///
    /// Returns [`ServiceError::Repository`] if storage cannot be accessed.
    pub fn list(&self) -> Result<Vec<Run>, ServiceError> {
        Ok(self.repository.list()?)
    }

    fn load(&self, id: &RunId) -> Result<Run, ServiceError> {
        self.repository
            .get(id)?
            .ok_or_else(|| ServiceError::NotFound(id.as_str().to_owned()))
    }
}

/// Creates the standard in-memory service used by the API composition root.
#[must_use]
pub fn in_memory_run_service() -> RunService {
    RunService::new(
        Arc::new(InMemoryRunRepository::default()),
        Arc::new(SequentialRunIdGenerator::new()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use local_stack_proof_domain::{EventKind, RunStatus};

    fn service() -> RunService {
        in_memory_run_service()
    }

    fn configuration(force_failure: bool) -> RunConfiguration {
        RunConfiguration::new(42, "north", 0.0, force_failure).expect("valid config")
    }

    #[test]
    fn creates_executes_retrieves_and_lists_a_completed_run() {
        let service = service();
        let created = service
            .create(configuration(false))
            .expect("run is created");
        assert_eq!(created.status(), RunStatus::Queued);
        assert_eq!(created.events().len(), 1);

        let executed = service.execute(created.id()).expect("run is executed");
        assert_eq!(executed.status(), RunStatus::Completed);
        assert!(executed.result().is_some());
        assert_eq!(
            executed
                .events()
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![EventKind::Created, EventKind::Started, EventKind::Completed]
        );
        assert_eq!(
            executed
                .events()
                .iter()
                .map(|event| event.message.as_str())
                .collect::<Vec<_>>(),
            vec!["Run created", "Run started", "Run completed"]
        );

        let retrieved = service.get(created.id()).expect("run is retrievable");
        assert_eq!(retrieved, executed);
        assert_eq!(service.list().expect("runs are listable"), vec![executed]);
    }

    #[test]
    fn persists_controlled_rejection_and_its_timeline() {
        let service = service();
        let created = service.create(configuration(true)).expect("run is created");
        let rejected = service
            .execute(created.id())
            .expect("validation rejection is a run outcome");

        assert_eq!(rejected.status(), RunStatus::Rejected);
        assert_eq!(rejected.validation_messages()[0].code, "forced-rejection");
        assert_eq!(
            rejected
                .events()
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![EventKind::Created, EventKind::Started, EventKind::Rejected]
        );
        assert_eq!(
            rejected
                .events()
                .iter()
                .map(|event| event.message.as_str())
                .collect::<Vec<_>>(),
            vec!["Run created", "Run started", "Run rejected by validation"]
        );
        assert_eq!(
            service.get(created.id()).expect("saved rejection").status(),
            RunStatus::Rejected
        );
    }

    #[test]
    fn execution_cannot_be_repeated_for_a_terminal_run() {
        let service = service();
        let created = service
            .create(configuration(false))
            .expect("run is created");
        service.execute(created.id()).expect("first execution");

        assert!(matches!(
            service.execute(created.id()),
            Err(ServiceError::Domain(DomainError::InvalidTransition { .. }))
        ));
        assert_eq!(
            service
                .get(created.id())
                .expect("original remains saved")
                .status(),
            RunStatus::Completed
        );
    }

    #[test]
    fn unknown_run_is_not_found() {
        let service = service();
        let id = RunId::new("missing").expect("valid id");
        assert_eq!(
            service.get(&id),
            Err(ServiceError::NotFound("missing".to_owned()))
        );
    }

    #[test]
    fn in_memory_repository_rejects_stale_lifecycle_writes() {
        let repository = InMemoryRunRepository::default();
        let run = Run::new(RunId::new("run-1").unwrap(), configuration(false));
        repository.insert(&run).unwrap();
        let mut started = run.clone();
        started.start().unwrap();
        repository.save(&started).unwrap();
        assert_eq!(
            repository.save(&started),
            Err(RepositoryError::Conflict("run-1".to_owned()))
        );
        assert_eq!(repository.get(run.id()).unwrap(), Some(started));
    }

    #[test]
    fn result_contains_group_summaries() {
        let result = calculate(&configuration(false)).expect("fixture succeeds");
        assert_eq!(result.matched_records, 7);
        assert_eq!(
            result
                .groups
                .iter()
                .map(|group| group.matched_records)
                .sum::<u32>(),
            7
        );
        let grouped_total = result
            .groups
            .iter()
            .map(|group| group.total_score)
            .sum::<f64>();
        assert!((grouped_total - result.total_score).abs() < f64::EPSILON);
    }
}
