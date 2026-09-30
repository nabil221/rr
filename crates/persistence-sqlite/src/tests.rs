use super::*;
use local_stack_proof_domain::{RunConfiguration, RunStatus, calculate};

fn configuration(reject: bool) -> RunConfiguration {
    RunConfiguration::new(42, "north", 0.0, reject).unwrap()
}

#[test]
fn evidence_inspection_is_read_only_and_never_creates_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("runs.sqlite3");
    assert!(SqliteRunRepository::open_read_only(&path).is_err());
    assert!(!path.exists());
    let service = SqliteRunRepository::open(&path).unwrap().service();
    let run = service.create(configuration(false)).unwrap();
    drop(service);
    let reader = SqliteRunRepository::open_read_only(&path).unwrap();
    assert_eq!(reader.get(run.id()).unwrap(), Some(run.clone()));
    assert!(reader.next_id().is_err());
    assert!(
        reader
            .insert(&Run::new(RunId::new("new").unwrap(), configuration(false)))
            .is_err()
    );
    assert_eq!(reader.list().unwrap(), vec![run]);
}

#[test]
fn service_survives_reconnect_and_migrations_are_idempotent() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("runs.sqlite3");
    let service = SqliteRunRepository::open(&path).unwrap().service();
    let completed = service.create(configuration(false)).unwrap();
    let completed = service.execute(completed.id()).unwrap();
    let rejected = service.create(configuration(true)).unwrap();
    let rejected = service.execute(rejected.id()).unwrap();
    let queued = service.create(configuration(false)).unwrap();
    assert_eq!(rejected.status(), RunStatus::Rejected);
    drop(service);
    let restarted = SqliteRunRepository::open(&path).unwrap().service();
    assert_eq!(
        restarted.list().unwrap(),
        vec![completed.clone(), rejected.clone(), queued.clone()]
    );
    for run in [&completed, &rejected, &queued] {
        assert_eq!(restarted.get(run.id()).unwrap(), *run);
    }
    assert!(restarted.execute(completed.id()).is_err());
    let fresh = restarted.create(configuration(false)).unwrap();
    assert!(fresh.id().as_str() > queued.id().as_str());
    assert!(restarted.get(&RunId::new("missing").unwrap()).is_err());
}

#[test]
fn all_states_roundtrip_and_stale_writes_are_rejected() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteRunRepository::open(&directory.path().join("runs.sqlite3")).unwrap();
    let mut run = Run::new(repository.next_id().unwrap(), configuration(false));
    assert!(matches!(
        repository.save(&run),
        Err(RepositoryError::NotFound(_))
    ));
    repository.insert(&run).unwrap();
    assert!(matches!(
        repository.insert(&run),
        Err(RepositoryError::AlreadyExists(_))
    ));
    assert!(matches!(
        repository.save(&run),
        Err(RepositoryError::Conflict(_))
    ));
    let queued = run.clone();
    run.start().unwrap();
    repository.save(&run).unwrap();
    assert_eq!(repository.get(run.id()).unwrap(), Some(run.clone()));
    assert!(matches!(
        repository.save(&run),
        Err(RepositoryError::Conflict(_))
    ));
    let mut altered = Run::new(
        run.id().clone(),
        RunConfiguration::new(1, "south", 0.0, false).unwrap(),
    );
    altered.start().unwrap();
    altered.fail("altered").unwrap();
    assert!(matches!(
        repository.save(&altered),
        Err(RepositoryError::Conflict(_))
    ));
    run.fail("controlled failure").unwrap();
    repository.save(&run).unwrap();
    assert_eq!(repository.get(run.id()).unwrap(), Some(run));
    assert!(matches!(
        repository.save(&queued),
        Err(RepositoryError::Conflict(_))
    ));
    assert_eq!(
        repository.get(&RunId::new("missing").unwrap()).unwrap(),
        None
    );
}

#[test]
fn event_immutability_append_failure_and_corrupt_readback_are_safe() {
    let directory = tempfile::tempdir().unwrap();
    let repository = SqliteRunRepository::open(&directory.path().join("runs.sqlite3")).unwrap();
    let mut run = Run::new(repository.next_id().unwrap(), configuration(false));
    repository.insert(&run).unwrap();
    run.start().unwrap();
    repository.save(&run).unwrap();
    {
        let connection = repository.connection().unwrap();
        assert!(
            connection
                .execute("UPDATE run_events SET event = '{}'", [])
                .is_err()
        );
        assert!(connection.execute("DELETE FROM run_events", []).is_err());
        connection.execute_batch("CREATE TRIGGER test_append_failure BEFORE INSERT ON run_events WHEN NEW.position = 2 BEGIN SELECT RAISE(ABORT, 'test only'); END;").unwrap();
    }
    let mut terminal = run.clone();
    terminal
        .complete(calculate(terminal.configuration()).unwrap())
        .unwrap();
    assert_eq!(
        repository.save(&terminal),
        Err(RepositoryError::Unavailable)
    );
    assert_eq!(repository.get(run.id()).unwrap(), Some(run.clone()));
    let connection = repository.connection().unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM run_events", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
    connection
        .execute("UPDATE runs SET snapshot = '{}'", [])
        .unwrap();
    drop(connection);
    assert_eq!(repository.get(run.id()), Err(RepositoryError::Unavailable));
    assert_eq!(repository.list(), Err(RepositoryError::Unavailable));
}

#[test]
fn independent_connections_serialize_transitions_and_allocate_unique_ids() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("runs.sqlite3");
    let repository = SqliteRunRepository::open(&path).unwrap();
    let run = Run::new(repository.next_id().unwrap(), configuration(false));
    repository.insert(&run).unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let adapter = SqliteRunRepository::open(&path).unwrap();
            let mut next = run.clone();
            next.start().unwrap();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                (adapter.save(&next), adapter.next_id().unwrap())
            })
        })
        .collect();
    let outcomes: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    assert_eq!(
        outcomes.iter().filter(|(result, _)| result.is_ok()).count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|(result, _)| matches!(result, Err(RepositoryError::Conflict(_))))
            .count(),
        1
    );
    assert_ne!(outcomes[0].1, outcomes[1].1);
    assert_eq!(repository.get(run.id()).unwrap().unwrap().events().len(), 2);
}

#[test]
fn unsupported_schema_and_inaccessible_paths_fail_without_resetting() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("runs.sqlite3");
    let repository = SqliteRunRepository::open(&path).unwrap();
    repository
        .connection()
        .unwrap()
        .execute("INSERT INTO schema_migrations VALUES (2)", [])
        .unwrap();
    drop(repository);
    assert!(matches!(
        SqliteRunRepository::open(&path),
        Err(RepositoryError::Unavailable)
    ));
    assert_eq!(
        Connection::open(&path)
            .unwrap()
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        2
    );
    assert!(matches!(
        SqliteRunRepository::open(&directory.path().join("absent/runs.sqlite3")),
        Err(RepositoryError::Unavailable)
    ));
}
