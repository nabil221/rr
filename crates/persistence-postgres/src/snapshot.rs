//! Storage mapping, separate from HTTP DTOs and the framework-free domain.

use local_stack_proof_application::RepositoryError;
use local_stack_proof_domain::{
    GroupSummary, Run, RunConfiguration, RunId, RunResult, ValidationMessage,
};
use serde::Deserialize;
use serde_json::{Value, json};

pub fn snapshot(run: &Run) -> Value {
    let config = run.configuration();
    json!({
        "configuration": {
            "seed": config.seed, "region": config.region, "threshold": config.threshold,
            "force_validation_failure": config.force_validation_failure
        },
        "status": format!("{:?}", run.status()),
        "result": run.result().map(|r| json!({
            "total_records": r.total_records, "matched_records": r.matched_records,
            "total_score": r.total_score, "average_score": r.average_score,
            "groups": r.groups.iter().map(|g| json!({
                "category": g.category, "matched_records": g.matched_records,
                "total_score": g.total_score, "average_score": g.average_score
            })).collect::<Vec<_>>()
        })),
        "validation_messages": run.validation_messages().iter().map(|m| json!({
            "code": m.code, "message": m.message, "field": m.field
        })).collect::<Vec<_>>()
    })
}

pub fn events(run: &Run) -> Value {
    json!(
        run.events()
            .iter()
            .map(|e| json!({
                "kind": format!("{:?}", e.kind), "message": e.message
            }))
            .collect::<Vec<_>>()
    )
}

#[derive(Deserialize)]
struct StoredRun {
    configuration: StoredConfiguration,
    result: Option<StoredResult>,
    validation_messages: Vec<StoredMessage>,
}

#[derive(Deserialize)]
struct StoredConfiguration {
    seed: u64,
    region: String,
    threshold: f64,
    force_validation_failure: bool,
}

#[derive(Deserialize)]
struct StoredResult {
    total_records: u32,
    matched_records: u32,
    total_score: f64,
    average_score: f64,
    groups: Vec<StoredGroup>,
}

#[derive(Deserialize)]
struct StoredGroup {
    category: String,
    matched_records: u32,
    total_score: f64,
    average_score: f64,
}

#[derive(Deserialize)]
struct StoredMessage {
    code: String,
    message: String,
    field: Option<String>,
}

#[derive(Deserialize)]
struct StoredEvent {
    kind: String,
    message: String,
}

pub fn restore(id: &str, stored: &Value, timeline: &Value) -> Result<Run, RepositoryError> {
    let record: StoredRun =
        serde_json::from_value(stored.clone()).map_err(|_| RepositoryError::Unavailable)?;
    let history: Vec<StoredEvent> =
        serde_json::from_value(timeline.clone()).map_err(|_| RepositoryError::Unavailable)?;
    let config = record.configuration;
    let mut run = Run::new(
        RunId::new(id).map_err(|_| RepositoryError::Unavailable)?,
        RunConfiguration::new(
            config.seed,
            config.region,
            config.threshold,
            config.force_validation_failure,
        )
        .map_err(|_| RepositoryError::Unavailable)?,
    );
    let mut result = record.result;
    let mut messages = record.validation_messages;
    for event in history.iter().skip(1) {
        match event.kind.as_str() {
            "Started" => run.start(),
            "Completed" => {
                let result = result.take().ok_or(RepositoryError::Unavailable)?;
                run.complete(RunResult {
                    total_records: result.total_records,
                    matched_records: result.matched_records,
                    total_score: result.total_score,
                    average_score: result.average_score,
                    groups: result
                        .groups
                        .into_iter()
                        .map(|g| GroupSummary {
                            category: g.category,
                            matched_records: g.matched_records,
                            total_score: g.total_score,
                            average_score: g.average_score,
                        })
                        .collect(),
                })
            }
            "Rejected" => run.reject(
                std::mem::take(&mut messages)
                    .into_iter()
                    .map(|m| ValidationMessage {
                        code: m.code,
                        message: m.message,
                        field: m.field,
                    })
                    .collect(),
            ),
            "Failed" => run.fail(&event.message),
            _ => return Err(RepositoryError::Unavailable),
        }
        .map_err(|_| RepositoryError::Unavailable)?;
    }
    if &snapshot(&run) != stored || &events(&run) != timeline {
        return Err(RepositoryError::Unavailable);
    }
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use local_stack_proof_domain::calculate;

    #[test]
    fn roundtrips_all_lifecycle_states_without_recalculating() {
        let mut run = Run::new(
            RunId::new("run-1").unwrap(),
            RunConfiguration::new(42, "north", 0.0, false).unwrap(),
        );
        for _ in 0..2 {
            assert_eq!(
                restore(run.id().as_str(), &snapshot(&run), &events(&run)).unwrap(),
                run
            );
            if run.events().len() == 1 {
                run.start().unwrap();
            }
        }
        run.complete(calculate(run.configuration()).unwrap())
            .unwrap();
        assert_eq!(
            restore(run.id().as_str(), &snapshot(&run), &events(&run)).unwrap(),
            run
        );
        let mut corrupt = snapshot(&run);
        corrupt["status"] = json!("Queued");
        assert_eq!(
            restore(run.id().as_str(), &corrupt, &events(&run)),
            Err(RepositoryError::Unavailable)
        );
        for rejected in [false, true] {
            let mut terminal = Run::new(
                RunId::new("terminal").unwrap(),
                RunConfiguration::new(42, "north", 0.0, rejected).unwrap(),
            );
            terminal.start().unwrap();
            if rejected {
                terminal
                    .reject(calculate(terminal.configuration()).unwrap_err())
                    .unwrap();
            } else {
                terminal.fail("controlled failure").unwrap();
            }
            assert_eq!(
                restore(
                    terminal.id().as_str(),
                    &snapshot(&terminal),
                    &events(&terminal)
                )
                .unwrap(),
                terminal
            );
        }
    }
}
