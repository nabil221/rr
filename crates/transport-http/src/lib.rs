//! HTTP adapter: JSON DTOs in, application-service calls, JSON DTOs out.

mod boundary;
mod contract;

pub use contract::typescript_contract;

use axum::extract::{DefaultBodyLimit, Path, State, rejection::JsonRejection};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use local_stack_proof_application::{RepositoryError, RunService, ServiceError};
use local_stack_proof_domain::{
    DomainError, EventKind, GroupSummary, Run, RunConfiguration, RunEvent, RunId, RunResult,
    RunStatus, ValidationMessage,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone)]
struct AppState {
    runs: RunService,
    storage: &'static str,
}

/// Compose the host-independent demo API around shared application services.
pub fn router(runs: RunService) -> Router {
    compose(runs, "in-memory").layer(axum::middleware::from_fn(boundary::request_id))
}

/// Compose the persistent web host with its exact local-origin boundary.
pub fn web_router(runs: RunService) -> Router {
    compose(runs, "postgres")
        .layer(axum::middleware::from_fn(boundary::web_origin))
        .layer(axum::middleware::from_fn(boundary::request_id))
}

fn compose(runs: RunService, storage: &'static str) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/runs", post(create_run).get(list_runs))
        .route("/api/runs/{id}", get(get_run))
        .route("/api/runs/{id}/execute", post(execute_run))
        .fallback(|| async {
            ApiFailure::new(StatusCode::NOT_FOUND, "not-found", "Route not found")
        })
        .method_not_allowed_fallback(|| async {
            ApiFailure::new(
                StatusCode::METHOD_NOT_ALLOWED,
                "method-not-allowed",
                "Method not allowed",
            )
        })
        .layer(DefaultBodyLimit::max(65_536))
        .with_state(AppState { runs, storage })
}

async fn health(State(state): State<AppState>) -> Json<HealthDto> {
    Json(HealthDto {
        status: "ok",
        storage: state.storage,
    })
}

async fn create_run(
    State(state): State<AppState>,
    request: Result<Json<CreateRunRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiFailure> {
    let Json(request) = request.map_err(|error| {
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
            ApiFailure::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "request-too-large",
                "Request body exceeds 64 KiB",
            )
        } else {
            ApiFailure::new(
                StatusCode::BAD_REQUEST,
                "invalid-json",
                "Expected a valid run configuration JSON object",
            )
        }
    })?;
    if request.seed > 9_007_199_254_740_991 {
        let mut failure = ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid-request",
            "Seed must be a JavaScript-safe unsigned integer",
        );
        failure.1.field = Some("seed".to_owned());
        return Err(failure);
    }
    let configuration = RunConfiguration::new(
        request.seed,
        request.region,
        request.threshold,
        request.force_validation_failure,
    )
    .map_err(|error| ApiFailure::from_domain(&error))?;

    let run = blocking(move || state.runs.create(configuration)).await?;
    Ok((StatusCode::CREATED, Json(RunDto::from(run))))
}

async fn list_runs(State(state): State<AppState>) -> Result<Json<Vec<RunDto>>, ApiFailure> {
    let runs = blocking(move || state.runs.list()).await?;
    Ok(Json(runs.into_iter().map(RunDto::from).collect()))
}

async fn get_run(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RunDto>, ApiFailure> {
    let id = RunId::new(id).map_err(|error| ApiFailure::from_domain(&error))?;
    let run = blocking(move || state.runs.get(&id)).await?;
    Ok(Json(RunDto::from(run)))
}

async fn execute_run(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RunDto>, ApiFailure> {
    let id = RunId::new(id).map_err(|error| ApiFailure::from_domain(&error))?;
    let run = blocking(move || state.runs.execute(&id)).await?;
    Ok(Json(RunDto::from(run)))
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
struct CreateRunRequest {
    #[ts(type = "number")]
    seed: u64,
    region: String,
    threshold: f64,
    force_validation_failure: bool,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
struct HealthDto {
    status: &'static str,
    storage: &'static str,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
struct RunDto {
    id: String,
    configuration: ConfigurationDto,
    status: StatusDto,
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

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
struct ConfigurationDto {
    #[ts(type = "number")]
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

#[derive(Debug, Serialize, TS)]
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

#[derive(Debug, Serialize, TS)]
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

#[derive(Debug, Serialize, TS)]
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

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
struct EventDto {
    kind: EventKindDto,
    message: String,
}

impl From<&RunEvent> for EventDto {
    fn from(event: &RunEvent) -> Self {
        Self {
            kind: match event.kind {
                EventKind::Created => EventKindDto::Created,
                EventKind::Started => EventKindDto::Started,
                EventKind::Completed => EventKindDto::Completed,
                EventKind::Rejected => EventKindDto::Rejected,
                EventKind::Failed => EventKindDto::Failed,
            },
            message: event.message.clone(),
        }
    }
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
struct ErrorDto {
    code: String,
    message: String,
    field: Option<String>,
}

struct ApiFailure(StatusCode, ErrorDto);

impl ApiFailure {
    fn new(status: StatusCode, code: &str, message: &str) -> Self {
        Self(
            status,
            ErrorDto {
                code: code.to_owned(),
                message: message.to_owned(),
                field: None,
            },
        )
    }

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
        if let ServiceError::Repository(repository) = error {
            return match repository {
                RepositoryError::Conflict(_) | RepositoryError::AlreadyExists(_) => Self::new(
                    StatusCode::CONFLICT,
                    "write-conflict",
                    "Run changed; reload before retrying",
                ),
                RepositoryError::NotFound(_) => {
                    Self::new(StatusCode::NOT_FOUND, "not-found", "Run not found")
                }
                RepositoryError::Unavailable => Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "repository-error",
                    "Run storage is unavailable",
                ),
            };
        }
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

async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, ServiceError> + Send + 'static,
) -> Result<T, ApiFailure> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| {
            ApiFailure::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal-error",
                "Request could not be completed",
            )
        })?
        .map_err(|error| ApiFailure::from_service(&error))
}

impl IntoResponse for ApiFailure {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(self.1)).into_response()
    }
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "lowercase")]
enum StatusDto {
    Queued,
    Running,
    Completed,
    Rejected,
    Failed,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "lowercase")]
enum EventKindDto {
    Created,
    Started,
    Completed,
    Rejected,
    Failed,
}

fn status_name(status: RunStatus) -> StatusDto {
    match status {
        RunStatus::Queued => StatusDto::Queued,
        RunStatus::Running => StatusDto::Running,
        RunStatus::Completed => StatusDto::Completed,
        RunStatus::Rejected => StatusDto::Rejected,
        RunStatus::Failed => StatusDto::Failed,
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

    #[tokio::test]
    async fn bounds_json_and_returns_safe_route_errors() {
        let router = router(local_stack_proof_application::in_memory_run_service());
        for (body, expected, code) in [
            ("{".to_owned(), StatusCode::BAD_REQUEST, "invalid-json"),
            (
                "x".repeat(65_537),
                StatusCode::PAYLOAD_TOO_LARGE,
                "request-too-large",
            ),
        ] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/runs")
                        .header("content-type", "application/json")
                        .header("x-request-id", "untrusted-client-id")
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
            assert_ne!(response.headers()["x-request-id"], "untrusted-client-id");
            let value: Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 65_536).await.unwrap())
                    .unwrap();
            assert_eq!(value["code"], code);
        }
        for (method, path, expected) in [
            ("GET", "/missing", StatusCode::NOT_FOUND),
            ("DELETE", "/api/runs", StatusCode::METHOD_NOT_ALLOWED),
        ] {
            let (status, error) = call(router.clone(), method, path, Value::Null).await;
            assert_eq!(status, expected);
            assert!(error["message"].is_string());
        }
    }

    #[tokio::test]
    async fn restricts_web_origin_and_preflight_and_correlates_all_responses() {
        let router = web_router(local_stack_proof_application::in_memory_run_service());
        for (method, origin, expected) in [
            ("GET", "http://127.0.0.1:5173", StatusCode::OK),
            ("OPTIONS", "http://127.0.0.1:5173", StatusCode::NO_CONTENT),
            ("GET", "https://example.com", StatusCode::FORBIDDEN),
            ("OPTIONS", "http://localhost:5173", StatusCode::FORBIDDEN),
        ] {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri("/api/health")
                        .header("origin", origin)
                        .header("access-control-request-method", "POST")
                        .header("access-control-request-headers", "Content-Type")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
            assert!(response.headers().contains_key("x-request-id"));
            if expected == StatusCode::FORBIDDEN {
                assert!(
                    !response
                        .headers()
                        .contains_key("access-control-allow-origin")
                );
            } else {
                assert_eq!(response.headers()["access-control-allow-origin"], origin);
            }
        }
        let (status, health) = call(router, "GET", "/api/health", Value::Null).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(health["storage"], "postgres");
    }

    struct UnavailableRepository;

    impl local_stack_proof_application::RunRepository for UnavailableRepository {
        fn insert(&self, _: &Run) -> Result<(), RepositoryError> {
            Err(RepositoryError::Unavailable)
        }
        fn save(&self, _: &Run) -> Result<(), RepositoryError> {
            Err(RepositoryError::Unavailable)
        }
        fn get(&self, _: &RunId) -> Result<Option<Run>, RepositoryError> {
            Err(RepositoryError::Unavailable)
        }
        fn list(&self) -> Result<Vec<Run>, RepositoryError> {
            Err(RepositoryError::Unavailable)
        }
    }

    #[tokio::test]
    async fn storage_failure_and_conflict_have_safe_envelopes() {
        let service = RunService::new(
            std::sync::Arc::new(UnavailableRepository),
            std::sync::Arc::new(local_stack_proof_application::SequentialRunIdGenerator::new()),
        );
        let (status, error) = call(router(service), "GET", "/api/runs", Value::Null).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            error,
            json!({"code":"repository-error", "message":"Run storage is unavailable", "field":null})
        );
        let failure = ApiFailure::from_service(&ServiceError::Repository(
            RepositoryError::Conflict("private-id".to_owned()),
        ));
        assert_eq!(failure.0, StatusCode::CONFLICT);
        assert!(!failure.1.message.contains("private-id"));
    }

    #[tokio::test]
    async fn rejects_unsafe_javascript_seed_but_accepts_boundary() {
        let router = router(local_stack_proof_application::in_memory_run_service());
        let mut input = configuration(false);
        input["seed"] = json!(9_007_199_254_740_992_u64);
        let (status, error) = call(router.clone(), "POST", "/api/runs", input.clone()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["field"], "seed");
        input["seed"] = json!(9_007_199_254_740_991_u64);
        let (status, _) = call(router, "POST", "/api/runs", input).await;
        assert_eq!(status, StatusCode::CREATED);
    }
}
