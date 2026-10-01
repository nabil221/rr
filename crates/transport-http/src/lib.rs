//! HTTP adapter: JSON DTOs in, application-service calls, JSON DTOs out.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use local_stack_proof_application::{RunService, ServiceError};
use local_stack_proof_domain::{
    DomainError, EventKind, GroupSummary, Run, RunConfiguration, RunEvent, RunId, RunResult,
    RunStatus, ValidationMessage,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    runs: RunService,
}

/// Compose the host-independent demo API around shared application services.
pub fn router(runs: RunService) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/runs", post(create_run).get(list_runs))
        .route("/api/runs/{id}", get(get_run))
        .route("/api/runs/{id}/execute", post(execute_run))
        .with_state(AppState { runs })
}

async fn health() -> Json<HealthDto> {
    Json(HealthDto {
        status: "ok",
        storage: "in-memory",
    })
}

async fn create_run(
    State(state): State<AppState>,
    Json(request): Json<CreateRunRequest>,
) -> Result<impl IntoResponse, ApiFailure> {
    let configuration = RunConfiguration::new(
        request.seed,
        request.region,
        request.threshold,
        request.force_validation_failure,
    )
    .map_err(|error| ApiFailure::from_domain(&error))?;

    let run = state
        .runs
        .create(configuration)
        .map_err(|error| ApiFailure::from_service(&error))?;
    Ok((StatusCode::CREATED, Json(RunDto::from(run))))
}

async fn list_runs(State(state): State<AppState>) -> Result<Json<Vec<RunDto>>, ApiFailure> {
    let runs = state
        .runs
        .list()
        .map_err(|error| ApiFailure::from_service(&error))?;
    Ok(Json(runs.into_iter().map(RunDto::from).collect()))
}

async fn get_run(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RunDto>, ApiFailure> {
    let id = RunId::new(id).map_err(|error| ApiFailure::from_domain(&error))?;
    let run = state
        .runs
        .get(&id)
        .map_err(|error| ApiFailure::from_service(&error))?;
    Ok(Json(RunDto::from(run)))
}

async fn execute_run(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RunDto>, ApiFailure> {
    let id = RunId::new(id).map_err(|error| ApiFailure::from_domain(&error))?;
    let run = state
        .runs
        .execute(&id)
        .map_err(|error| ApiFailure::from_service(&error))?;
    Ok(Json(RunDto::from(run)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRunRequest {
    seed: u64,
    region: String,
    threshold: f64,
    force_validation_failure: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthDto {
    status: &'static str,
    storage: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunDto {
    id: String,
    configuration: ConfigurationDto,
    status: &'static str,
    result: Option<ResultDto>,
    validation_messages: Vec<ValidationMessageDto>,
    events: Vec<EventDto>,
}

impl From<Run> for RunDto {
    fn from(run: Run) -> Self {
        Self {
            id: run.id().as_str().to_owned(),
            configuration: ConfigurationDto::from(run.configuration()),
            status: status_name(run.status()),
            result: run.result().map(ResultDto::from),
            validation_messages: run
                .validation_messages()
                .iter()
                .map(ValidationMessageDto::from)
                .collect(),
            events: run.events().iter().map(EventDto::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigurationDto {
    seed: u64,
    region: String,
    threshold: f64,
    force_validation_failure: bool,
}

impl From<&RunConfiguration> for ConfigurationDto {
    fn from(configuration: &RunConfiguration) -> Self {
        Self {
            seed: configuration.seed,
            region: configuration.region.clone(),
            threshold: configuration.threshold,
            force_validation_failure: configuration.force_validation_failure,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResultDto {
    total_records: u32,
    matched_records: u32,
    total_score: f64,
    average_score: f64,
    groups: Vec<GroupDto>,
}

impl From<&RunResult> for ResultDto {
    fn from(result: &RunResult) -> Self {
        Self {
            total_records: result.total_records,
            matched_records: result.matched_records,
            total_score: result.total_score,
            average_score: result.average_score,
            groups: result.groups.iter().map(GroupDto::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupDto {
    category: String,
    matched_records: u32,
    total_score: f64,
    average_score: f64,
}

impl From<&GroupSummary> for GroupDto {
    fn from(group: &GroupSummary) -> Self {
        Self {
            category: group.category.clone(),
            matched_records: group.matched_records,
            total_score: group.total_score,
            average_score: group.average_score,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationMessageDto {
    code: String,
    message: String,
    field: Option<String>,
}

impl From<&ValidationMessage> for ValidationMessageDto {
    fn from(message: &ValidationMessage) -> Self {
        Self {
            code: message.code.clone(),
            message: message.message.clone(),
            field: message.field.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EventDto {
    kind: &'static str,
    message: String,
}

impl From<&RunEvent> for EventDto {
    fn from(event: &RunEvent) -> Self {
        Self {
            kind: match event.kind {
                EventKind::Created => "created",
                EventKind::Started => "started",
                EventKind::Completed => "completed",
                EventKind::Rejected => "rejected",
                EventKind::Failed => "failed",
            },
            message: event.message.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorDto {
    code: String,
    message: String,
    field: Option<String>,
}

struct ApiFailure(StatusCode, ErrorDto);

impl ApiFailure {
    fn from_domain(error: &DomainError) -> Self {
        let field = match error {
            DomainError::InvalidRegion => Some("region"),
            DomainError::InvalidThreshold => Some("threshold"),
            _ => None,
        };
        Self(
            StatusCode::BAD_REQUEST,
            ErrorDto {
                code: "invalid-request".to_owned(),
                message: error.to_string(),
                field: field.map(str::to_owned),
            },
        )
    }

    fn from_service(error: &ServiceError) -> Self {
        let (status, code) = match error {
            ServiceError::NotFound(_) => (StatusCode::NOT_FOUND, "not-found"),
            ServiceError::Domain(DomainError::InvalidTransition { .. }) => {
                (StatusCode::CONFLICT, "invalid-transition")
            }
            ServiceError::Domain(_) => (StatusCode::BAD_REQUEST, "invalid-request"),
            ServiceError::Repository(_) => (StatusCode::INTERNAL_SERVER_ERROR, "repository-error"),
        };
        Self(
            status,
            ErrorDto {
                code: code.to_owned(),
                message: error.to_string(),
                field: None,
            },
        )
    }
}

impl IntoResponse for ApiFailure {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(self.1)).into_response()
    }
}

fn status_name(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Queued => "queued",
        RunStatus::Running => "running",
        RunStatus::Completed => "completed",
        RunStatus::Rejected => "rejected",
        RunStatus::Failed => "failed",
    }
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;

    async fn call(router: Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("Content-Type", "application/json")
            .body(if body.is_null() {
                Body::empty()
            } else {
                Body::from(body.to_string())
            })
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 65_536).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    fn configuration(force_rejection: bool) -> Value {
        json!({"seed":42,"region":"north","threshold":0,"forceValidationFailure":force_rejection})
    }

    #[tokio::test]
    async fn creates_executes_reads_and_prevents_repeat_execution() {
        let router = router(local_stack_proof_application::in_memory_run_service());
        let (status, created) =
            call(router.clone(), "POST", "/api/runs", configuration(false)).await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(created["status"], "queued");
        let id = created["id"].as_str().unwrap();
        let execute_path = format!("/api/runs/{id}/execute");
        let (status, completed) = call(router.clone(), "POST", &execute_path, Value::Null).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(completed["status"], "completed");
        assert_eq!(completed["result"]["matchedRecords"], 7);
        assert_eq!(completed["events"].as_array().unwrap().len(), 3);
        let (_, readback) = call(
            router.clone(),
            "GET",
            &format!("/api/runs/{id}"),
            Value::Null,
        )
        .await;
        assert_eq!(readback, completed);
        let (_, list) = call(router.clone(), "GET", "/api/runs", Value::Null).await;
        assert_eq!(list, json!([completed]));
        let (status, error) = call(router, "POST", &execute_path, Value::Null).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(error["code"], "invalid-transition");
    }

    #[tokio::test]
    async fn returns_controlled_rejection_and_typed_http_errors() {
        let router = router(local_stack_proof_application::in_memory_run_service());
        let (_, created) = call(router.clone(), "POST", "/api/runs", configuration(true)).await;
        let path = format!("/api/runs/{}/execute", created["id"].as_str().unwrap());
        let (status, rejected) = call(router.clone(), "POST", &path, Value::Null).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(rejected["status"], "rejected");
        assert_eq!(rejected["validationMessages"].as_array().unwrap().len(), 1);
        let mut invalid = configuration(false);
        invalid["region"] = json!("");
        let (status, error) = call(router.clone(), "POST", "/api/runs", invalid).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["field"], "region");
        let (status, error) = call(router, "GET", "/api/runs/missing", Value::Null).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(error["code"], "not-found");
    }
}
