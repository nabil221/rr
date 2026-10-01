//! Project-scoped, synchronous Postgres persistence adapter.

mod snapshot;

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use local_stack_proof_application::{
    RepositoryError, RunIdGenerator, RunRepository, RunService, ServiceError,
};
use local_stack_proof_domain::{Run, RunId};
use postgres::{Client, Config, NoTls, Row, config::Host, error::SqlState};

/// Non-secret defaults for the project-owned Docker fixture, never a production credential.
pub const LOCAL_DATABASE_URL: &str =
    "postgresql://local_stack_proof:local_stack_proof@127.0.0.1:54329/local_stack_proof";
const SELECT_RUN: &str = "SELECT r.id, r.snapshot, (SELECT coalesce(jsonb_agg(e.event ORDER BY e.position), '[]'::jsonb) FROM run_events e WHERE e.run_id = r.id) AS events FROM runs r";

/// Repository and sequence-backed identifier source for the local web host.
pub struct PostgresRunRepository {
    client: Mutex<Client>,
}

fn config(url: &str) -> Result<Config, RepositoryError> {
    let mut config: Config = url.parse().map_err(|_| RepositoryError::Unavailable)?;
    if config.get_dbname() != Some("local_stack_proof")
        || config.get_hosts() != [Host::Tcp("127.0.0.1".to_owned())]
        || config.get_ports() != [54329]
        || config.get_options().is_some()
    {
        return Err(RepositoryError::Unavailable);
    }
    config.connect_timeout(Duration::from_secs(5));
    Ok(config)
}

/// Validates that the configured database is exactly the local project fixture.
///
/// # Errors
/// Returns a safe error for malformed, remote, or non-project database configuration.
pub fn validate_database_url(url: &str) -> Result<(), RepositoryError> {
    config(url).map(|_| ())
}

impl PostgresRunRepository {
    /// Connects and migrates the project-owned production schema.
    /// Call from a blocking worker, not an async executor thread.
    ///
    /// # Errors
    /// Returns a safe repository error for invalid configuration or unavailable storage.
    pub fn connect(url: &str) -> Result<Self, RepositoryError> {
        Self::connect_schema(url, "stack_proof")
    }

    fn connect_schema(url: &str, schema: &str) -> Result<Self, RepositoryError> {
        if !valid_schema(schema) {
            return Err(RepositoryError::Unavailable);
        }
        let mut client = config(url)?
            .connect(NoTls)
            .map_err(|_| RepositoryError::Unavailable)?;
        let mut transaction = client
            .transaction()
            .map_err(|_| RepositoryError::Unavailable)?;
        transaction
            .batch_execute("SET LOCAL statement_timeout = '5s'; SET LOCAL lock_timeout = '5s'")
            .map_err(|_| RepositoryError::Unavailable)?;
        transaction
            .query_one("SELECT pg_advisory_xact_lock(721103001)", &[])
            .map_err(|_| RepositoryError::Unavailable)?;
        transaction.batch_execute(&format!("CREATE SCHEMA IF NOT EXISTS {schema}; SET search_path TO {schema}; CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY)")).map_err(|_| RepositoryError::Unavailable)?;
        let migrated = transaction
            .query_opt(
                "SELECT version FROM schema_migrations WHERE version = 1",
                &[],
            )
            .map_err(|_| RepositoryError::Unavailable)?
            .is_some();
        if !migrated {
            transaction
                .batch_execute(include_str!("../migrations/0001_runs.sql"))
                .map_err(|_| RepositoryError::Unavailable)?;
            transaction
                .execute("INSERT INTO schema_migrations (version) VALUES (1)", &[])
                .map_err(|_| RepositoryError::Unavailable)?;
        }
        transaction
            .commit()
            .map_err(|_| RepositoryError::Unavailable)?;
        client
            .batch_execute("SET statement_timeout = '5s'; SET lock_timeout = '5s'")
            .map_err(|_| RepositoryError::Unavailable)?;
        Ok(Self {
            client: Mutex::new(client),
        })
    }

    fn client(&self) -> Result<MutexGuard<'_, Client>, RepositoryError> {
        self.client.lock().map_err(|_| RepositoryError::Unavailable)
    }

    /// Composes the shared application service with persistent IDs and repository.
    #[must_use]
    pub fn service(self) -> RunService {
        let repository = Arc::new(self);
        RunService::new(repository.clone(), repository)
    }
}

fn valid_schema(schema: &str) -> bool {
    schema == "stack_proof"
        || (schema.starts_with("stack_proof_test_")
            && schema.len() <= 63
            && schema
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_'))
}

fn restore(row: &Row) -> Result<Run, RepositoryError> {
    snapshot::restore(row.get::<_, &str>(0), &row.get(1), &row.get(2))
}

impl RunIdGenerator for PostgresRunRepository {
    fn next_id(&self) -> Result<RunId, ServiceError> {
        let value: i64 = self
            .client()?
            .query_one("SELECT nextval('run_id_seq')", &[])
            .map_err(|_| RepositoryError::Unavailable)?
            .get(0);
        Ok(RunId::new(format!("run-{value:020}"))?)
    }
}

impl RunRepository for PostgresRunRepository {
    fn insert(&self, run: &Run) -> Result<(), RepositoryError> {
        if run.events().len() != 1 {
            return Err(RepositoryError::Conflict(run.id().as_str().to_owned()));
        }
        let mut client = self.client()?;
        let mut transaction = client
            .transaction()
            .map_err(|_| RepositoryError::Unavailable)?;
        transaction
            .execute(
                "INSERT INTO runs (id, snapshot) VALUES ($1, $2)",
                &[&run.id().as_str(), &snapshot::snapshot(run)],
            )
            .map_err(|e| {
                if e.code() == Some(&SqlState::UNIQUE_VIOLATION) {
                    RepositoryError::AlreadyExists(run.id().as_str().to_owned())
                } else {
                    RepositoryError::Unavailable
                }
            })?;
        let timeline = snapshot::events(run);
        transaction
            .execute(
                "INSERT INTO run_events (run_id, position, event) VALUES ($1, 0, $2)",
                &[&run.id().as_str(), &timeline[0]],
            )
            .map_err(|_| RepositoryError::Unavailable)?;
        transaction
            .commit()
            .map_err(|_| RepositoryError::Unavailable)
    }

    fn save(&self, run: &Run) -> Result<(), RepositoryError> {
        let id = run.id().as_str();
        let mut client = self.client()?;
        let mut transaction = client
            .transaction()
            .map_err(|_| RepositoryError::Unavailable)?;
        transaction
            .query_opt("SELECT id FROM runs WHERE id = $1 FOR UPDATE", &[&id])
            .map_err(|_| RepositoryError::Unavailable)?
            .ok_or_else(|| RepositoryError::NotFound(id.to_owned()))?;
        // A fresh statement after acquiring the lock sees all committed event rows,
        // including changes committed while this transaction waited for the lock.
        let row = transaction
            .query_one(&format!("{SELECT_RUN} WHERE r.id = $1"), &[&id])
            .map_err(|_| RepositoryError::Unavailable)?;
        let saved = restore(&row)?;
        if saved.configuration() != run.configuration()
            || run.events().len() != saved.events().len() + 1
            || !run.events().starts_with(saved.events())
        {
            return Err(RepositoryError::Conflict(id.to_owned()));
        }
        let position =
            i32::try_from(saved.events().len()).map_err(|_| RepositoryError::Unavailable)?;
        let timeline = snapshot::events(run);
        transaction
            .execute(
                "UPDATE runs SET snapshot = $2 WHERE id = $1",
                &[&id, &snapshot::snapshot(run)],
            )
            .map_err(|_| RepositoryError::Unavailable)?;
        transaction
            .execute(
                "INSERT INTO run_events (run_id, position, event) VALUES ($1, $2, $3)",
                &[&id, &position, &timeline[saved.events().len()]],
            )
            .map_err(|_| RepositoryError::Unavailable)?;
        transaction
            .commit()
            .map_err(|_| RepositoryError::Unavailable)
    }

    fn get(&self, id: &RunId) -> Result<Option<Run>, RepositoryError> {
        self.client()?
            .query_opt(&format!("{SELECT_RUN} WHERE r.id = $1"), &[&id.as_str()])
            .map_err(|_| RepositoryError::Unavailable)?
            .as_ref()
            .map(restore)
            .transpose()
    }

    fn list(&self) -> Result<Vec<Run>, RepositoryError> {
        self.client()?
            .query(&format!("{SELECT_RUN} ORDER BY r.id"), &[])
            .map_err(|_| RepositoryError::Unavailable)?
            .iter()
            .map(restore)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use local_stack_proof_domain::{RunConfiguration, RunStatus};

    #[test]
    fn configuration_rejects_non_project_connections_and_schemas() {
        assert!(validate_database_url(LOCAL_DATABASE_URL).is_ok());
        for url in [
            "postgresql://localhost/postgres",
            "postgresql://127.0.0.1:54329/other",
            "postgresql://example.com:54329/local_stack_proof",
        ] {
            assert!(validate_database_url(url).is_err());
        }
        for schema in [
            "public",
            "stack_proof_test_x;DROP SCHEMA public",
            "stack_proof_test_A",
        ] {
            assert!(!valid_schema(schema));
        }
    }

    struct TestSchema {
        name: String,
    }

    impl TestSchema {
        fn new() -> Self {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self {
                name: format!("stack_proof_test_{}_{}", std::process::id(), stamp),
            }
        }
        fn repository(&self) -> PostgresRunRepository {
            PostgresRunRepository::connect_schema(LOCAL_DATABASE_URL, &self.name)
                .expect("project Postgres must be running")
        }
    }

    impl Drop for TestSchema {
        fn drop(&mut self) {
            assert!(self.name.starts_with("stack_proof_test_") && valid_schema(&self.name));
            let cleanup = config(LOCAL_DATABASE_URL).and_then(|configuration| {
                let mut client = configuration
                    .connect(NoTls)
                    .map_err(|_| RepositoryError::Unavailable)?;
                client
                    .batch_execute(&format!("DROP SCHEMA IF EXISTS {} CASCADE", self.name))
                    .map_err(|_| RepositoryError::Unavailable)
            });
            if cleanup.is_err() {
                eprintln!("test schema cleanup failed for {}", self.name);
            }
        }
    }

    #[test]
    #[ignore = "requires the project-owned local Postgres; run npm run test:postgres"]
    fn postgres_serializes_competing_transition_writes() {
        let schema = TestSchema::new();
        let repository = schema.repository();
        let run = Run::new(
            repository.next_id().unwrap(),
            RunConfiguration::new(42, "north", 0.0, false).unwrap(),
        );
        repository.insert(&run).unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = (0..2)
            .map(|_| {
                let competing = schema.repository();
                let mut next = run.clone();
                next.start().unwrap();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    competing.save(&next)
                })
            })
            .collect();
        let outcomes: Vec<_> = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect();
        assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, Err(RepositoryError::Conflict(_))))
                .count(),
            1
        );
        assert_eq!(repository.get(run.id()).unwrap().unwrap().events().len(), 2);
    }

    #[test]
    #[ignore = "requires the project-owned local Postgres; run npm run test:postgres"]
    fn postgres_service_survives_reconnect_and_preserves_events() {
        let schema = TestSchema::new();
        let service = schema.repository().service();
        let successful = service
            .create(RunConfiguration::new(42, "north", 0.0, false).unwrap())
            .unwrap();
        let completed = service.execute(successful.id()).unwrap();
        let rejected = service
            .create(RunConfiguration::new(42, "north", 0.0, true).unwrap())
            .unwrap();
        let rejected = service.execute(rejected.id()).unwrap();
        assert_eq!(rejected.status(), RunStatus::Rejected);
        drop(service);
        let reconnected = schema.repository().service();
        assert_eq!(reconnected.get(completed.id()).unwrap(), completed);
        assert_eq!(reconnected.get(rejected.id()).unwrap(), rejected);
        assert_eq!(
            reconnected.list().unwrap(),
            vec![completed.clone(), rejected]
        );
        let fresh = reconnected
            .create(RunConfiguration::new(1, "south", 0.0, false).unwrap())
            .unwrap();
        assert_ne!(fresh.id(), completed.id());
        assert!(reconnected.execute(completed.id()).is_err());
    }

    #[test]
    #[ignore = "requires the project-owned local Postgres; run npm run test:postgres"]
    fn postgres_rejects_stale_writes_and_event_mutation_atomically() {
        let schema = TestSchema::new();
        let repository = schema.repository();
        let run = Run::new(
            repository.next_id().unwrap(),
            RunConfiguration::new(42, "north", 0.0, false).unwrap(),
        );
        repository.insert(&run).unwrap();
        assert!(matches!(
            repository.insert(&run),
            Err(RepositoryError::AlreadyExists(_))
        ));
        let mut first = repository.get(run.id()).unwrap().unwrap();
        let mut stale = first.clone();
        first.start().unwrap();
        stale.start().unwrap();
        repository.save(&first).unwrap();
        assert!(matches!(
            schema.repository().save(&stale),
            Err(RepositoryError::Conflict(_))
        ));
        assert_eq!(repository.get(run.id()).unwrap().unwrap(), first);
        let mut client = repository.client().unwrap();
        let count: i64 = client
            .query_one("SELECT count(*) FROM run_events", &[])
            .unwrap()
            .get(0);
        assert_eq!(count, 2);
        assert!(
            client
                .execute("UPDATE run_events SET event = '{}'::jsonb", &[])
                .is_err()
        );
        assert!(client.execute("DELETE FROM run_events", &[]).is_err());
        client.batch_execute("ALTER TABLE run_events ADD CONSTRAINT test_force_append_failure CHECK (position < 2)").unwrap();
        drop(client);
        let mut terminal = first.clone();
        terminal
            .complete(local_stack_proof_domain::calculate(terminal.configuration()).unwrap())
            .unwrap();
        assert_eq!(
            repository.save(&terminal),
            Err(RepositoryError::Unavailable)
        );
        assert_eq!(repository.get(run.id()).unwrap().unwrap(), first);
        let mut client = repository.client().unwrap();
        let corrupt = serde_json::json!({"configuration": null});
        client
            .execute("UPDATE runs SET snapshot = $1", &[&corrupt])
            .unwrap();
        drop(client);
        assert_eq!(repository.get(run.id()), Err(RepositoryError::Unavailable));
    }
}
