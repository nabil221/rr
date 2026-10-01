//! Read-only verification of the two exact runs produced by the native UI smoke.

use std::path::PathBuf;
use std::process::ExitCode;

use local_stack_proof_application::RunRepository;
use local_stack_proof_domain::{RunId, RunStatus};
use local_stack_proof_persistence_sqlite::SqliteRunRepository;

fn verify() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.len() != 3 {
        return Err("usage: sqlite-proof <database> <completed-id> <rejected-id>".into());
    }
    let path = PathBuf::from(&arguments[0]);
    let repository = SqliteRunRepository::open_read_only(&path)?;
    for (index, status) in [RunStatus::Completed, RunStatus::Rejected]
        .into_iter()
        .enumerate()
    {
        let id = RunId::new(arguments[index + 1].to_str().ok_or("invalid run ID")?)?;
        let run = repository.get(&id)?.ok_or("missing smoke run")?;
        if run.status() != status
            || run.events().len() != 3
            || run.configuration().seed != 42
            || run.configuration().region != "north"
            || run.configuration().threshold != 0.0
        {
            return Err("smoke state does not match expected UI fixture".into());
        }
        if status == RunStatus::Completed {
            let result = run.result().ok_or("missing result")?;
            if result.total_records != 24
                || result.matched_records != 7
                || (result.average_score - (449.0 / 7.0)).abs() > 0.000_001
            {
                return Err("smoke result does not match deterministic fixture".into());
            }
        } else if run
            .validation_messages()
            .first()
            .is_none_or(|message| message.code != "forced-rejection")
        {
            return Err("missing controlled rejection".into());
        }
    }
    println!(
        "SQLite smoke verified read-only: exact completed/rejected IDs, fixture result, configuration, and three-event timelines"
    );
    Ok(())
}

fn main() -> ExitCode {
    if verify().is_ok() {
        ExitCode::SUCCESS
    } else {
        eprintln!("SQLite smoke verification failed; check the explicit database and expected IDs");
        ExitCode::FAILURE
    }
}
