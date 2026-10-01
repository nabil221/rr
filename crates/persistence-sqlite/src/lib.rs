//! Synchronous, self-contained SQLite repository for the local desktop host.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use local_stack_proof_application::{
    RepositoryError, RunIdGenerator, RunRepository, RunService, ServiceError,
};
use local_stack_proof_domain::{Run, RunId};
use local_stack_proof_persistence_format as snapshot;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

/// One bounded local connection; independent instances coordinate through SQLite.
pub struct SqliteRunRepository {
    connection: Mutex<Connection>,
}

impl SqliteRunRepository {
    /// Opens an existing database read-only for local evidence inspection.
    /// Does not create files, apply migrations, or allocate IDs.
    ///
    /// # Errors
    /// Returns unavailable for missing/inaccessible data or unsupported schema.
    pub fn open_read_only(path: &Path) -> Result<Self, RepositoryError> {
        let connection =
            Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(unavailable)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(unavailable)?;
        supported_schema(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Opens an explicit database path and applies supported migrations.
    /// The host owns directory selection/creation; tests use temporary directories.
    ///
    /// # Errors
    /// Returns unavailable for inaccessible data or unsupported schema.
    pub fn open(path: &Path) -> Result<Self, RepositoryError> {
        let mut connection = Connection::open(path).map_err(unavailable)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(unavailable)?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(unavailable)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable)?;
        let initialized: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations')", [], |row| row.get(0)
        ).map_err(unavailable)?;
        if initialized {
            supported_schema(&transaction)?;
        } else {
            transaction
                .execute_batch(include_str!("../migrations/0001_runs.sql"))
                .map_err(unavailable)?;
        }
        transaction.commit().map_err(unavailable)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn connection(&self) -> Result<MutexGuard<'_, Connection>, RepositoryError> {
        self.connection
            .lock()
            .map_err(|_| RepositoryError::Unavailable)
    }

    /// Composes shared application services over this repository and ID source.
    #[must_use]
    pub fn service(self) -> RunService {
        let repository = Arc::new(self);
        RunService::new(repository.clone(), repository)
    }
}

fn unavailable(_: rusqlite::Error) -> RepositoryError {
    RepositoryError::Unavailable
}

fn supported_schema(connection: &Connection) -> Result<(), RepositoryError> {
    let versions: Vec<i64> = connection
        .prepare("SELECT version FROM schema_migrations ORDER BY version")
        .map_err(unavailable)?
        .query_map([], |row| row.get(0))
        .map_err(unavailable)?
        .collect::<Result<_, _>>()
        .map_err(unavailable)?;
    if versions == [1] {
        Ok(())
    } else {
        Err(RepositoryError::Unavailable)
    }
}

fn load(connection: &Connection, id: &str) -> Result<Option<Run>, RepositoryError> {
    let stored: Option<String> = connection
        .query_row("SELECT snapshot FROM runs WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .optional()
        .map_err(unavailable)?;
    let Some(stored) = stored else {
        return Ok(None);
    };
    let mut statement = connection
        .prepare("SELECT position, event FROM run_events WHERE run_id = ?1 ORDER BY position")
        .map_err(unavailable)?;
    let rows = statement
        .query_map([id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(unavailable)?;
    let mut events = Vec::new();
    for (index, row) in rows.enumerate() {
        let (position, event) = row.map_err(unavailable)?;
        if position != i64::try_from(index).map_err(|_| RepositoryError::Unavailable)? {
            return Err(RepositoryError::Unavailable);
        }
        events.push(
            serde_json::from_str::<serde_json::Value>(&event)
                .map_err(|_| RepositoryError::Unavailable)?,
        );
    }
    let stored = serde_json::from_str(&stored).map_err(|_| RepositoryError::Unavailable)?;
    snapshot::restore(id, &stored, &serde_json::Value::Array(events)).map(Some)
}

fn append(connection: &Connection, run: &Run, position: usize) -> Result<(), RepositoryError> {
    let events = snapshot::events(run);
    connection
        .execute(
            "INSERT INTO run_events (run_id, position, event) VALUES (?1, ?2, ?3)",
            params![
                run.id().as_str(),
                i64::try_from(position).map_err(|_| RepositoryError::Unavailable)?,
                events[position].to_string()
            ],
        )
        .map_err(unavailable)?;
    Ok(())
}

impl RunIdGenerator for SqliteRunRepository {
    fn next_id(&self) -> Result<RunId, ServiceError> {
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable)?;
        let value: i64 = transaction.query_row("UPDATE run_id_sequence SET value = value + 1 WHERE singleton = 1 AND value < 9223372036854775807 RETURNING value", [], |row| row.get(0)).map_err(unavailable)?;
        transaction.commit().map_err(unavailable)?;
        Ok(RunId::new(format!("run-{value:020}"))?)
    }
}

impl RunRepository for SqliteRunRepository {
    fn insert(&self, run: &Run) -> Result<(), RepositoryError> {
        let id = run.id().as_str();
        if run.events().len() != 1 {
            return Err(RepositoryError::Conflict(id.to_owned()));
        }
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable)?;
        let exists: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM runs WHERE id = ?1)",
                [id],
                |row| row.get(0),
            )
            .map_err(unavailable)?;
        if exists {
            return Err(RepositoryError::AlreadyExists(id.to_owned()));
        }
        transaction
            .execute(
                "INSERT INTO runs (id, snapshot) VALUES (?1, ?2)",
                params![id, snapshot::snapshot(run).to_string()],
            )
            .map_err(unavailable)?;
        append(&transaction, run, 0)?;
        transaction.commit().map_err(unavailable)
    }

    fn save(&self, run: &Run) -> Result<(), RepositoryError> {
        let id = run.id().as_str();
        let mut connection = self.connection()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable)?;
        let saved =
            load(&transaction, id)?.ok_or_else(|| RepositoryError::NotFound(id.to_owned()))?;
        if saved.configuration() != run.configuration()
            || run.events().len() != saved.events().len() + 1
            || !run.events().starts_with(saved.events())
        {
            return Err(RepositoryError::Conflict(id.to_owned()));
        }
        transaction
            .execute(
                "UPDATE runs SET snapshot = ?2 WHERE id = ?1",
                params![id, snapshot::snapshot(run).to_string()],
            )
            .map_err(unavailable)?;
        append(&transaction, run, saved.events().len())?;
        transaction.commit().map_err(unavailable)
    }

    fn get(&self, id: &RunId) -> Result<Option<Run>, RepositoryError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction().map_err(unavailable)?;
        let run = load(&transaction, id.as_str())?;
        transaction.commit().map_err(unavailable)?;
        Ok(run)
    }

    fn list(&self) -> Result<Vec<Run>, RepositoryError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction().map_err(unavailable)?;
        let ids: Vec<String> = transaction
            .prepare("SELECT id FROM runs ORDER BY id")
            .map_err(unavailable)?
            .query_map([], |row| row.get(0))
            .map_err(unavailable)?
            .collect::<Result<_, _>>()
            .map_err(unavailable)?;
        let runs = ids
            .iter()
            .map(|id| load(&transaction, id)?.ok_or(RepositoryError::Unavailable))
            .collect::<Result<_, _>>()?;
        transaction.commit().map_err(unavailable)?;
        Ok(runs)
    }
}

#[cfg(test)]
mod tests;
