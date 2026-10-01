//! Dispatch `WebView` custom-protocol requests in-process. No socket is bound.

use std::time::Instant;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{HeaderValue, Method, Request, Response, StatusCode, header};
use tower::ServiceExt;

const MAX_BODY_BYTES: usize = 65_536;

pub async fn dispatch(router: Router, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let started = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let origin = request.headers().get(header::ORIGIN).cloned();
    let allowed = origin.as_ref().is_some_and(allowed_origin);

    let mut response = if !allowed {
        failure(
            StatusCode::FORBIDDEN,
            "forbidden-origin",
            "Desktop origin is not allowed",
        )
    } else if request.body().len() > MAX_BODY_BYTES {
        failure(
            StatusCode::PAYLOAD_TOO_LARGE,
            "request-too-large",
            "Request body exceeds 64 KiB",
        )
    } else if method == Method::OPTIONS {
        Response::new(Vec::new())
    } else {
        let response = match router.oneshot(request.map(Body::from)).await {
            Ok(response) => response,
            Err(never) => match never {},
        };
        let (parts, body) = response.into_parts();
        match to_bytes(body, MAX_BODY_BYTES).await {
            Ok(bytes) => Response::from_parts(parts, bytes.to_vec()),
            Err(_) => failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                "response-too-large",
                "Response body exceeds 64 KiB",
            ),
        }
    };

    if allowed {
        if let Some(origin) = origin {
            response
                .headers_mut()
                .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        }
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Origin"));
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Content-Type"),
        );
        if method == Method::OPTIONS && response.status() == StatusCode::OK {
            *response.status_mut() = StatusCode::NO_CONTENT;
        }
    }

    eprintln!(
        "desktop {method} {path} -> {} ({} ms)",
        response.status().as_u16(),
        started.elapsed().as_millis()
    );
    response
}

fn allowed_origin(origin: &HeaderValue) -> bool {
    origin == "http://tauri.localhost"
        || (cfg!(debug_assertions) && origin == "http://127.0.0.1:5174")
}

fn failure(status: StatusCode, code: &str, message: &str) -> Response<Vec<u8>> {
    let mut response = Response::new(
        format!(r#"{{"code":"{code}","message":"{message}","field":null}}"#).into_bytes(),
    );
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> Router {
        local_stack_proof_transport_http::router(
            local_stack_proof_application::in_memory_run_service(),
        )
    }

    fn request(method: Method, origin: &str, body: Vec<u8>) -> Request<Vec<u8>> {
        Request::builder()
            .method(method)
            .uri("http://proof-api.localhost/api/health")
            .header(header::ORIGIN, origin)
            .body(body)
            .unwrap()
    }

    #[tokio::test]
    async fn dispatches_health_with_exact_origin() {
        let response = dispatch(
            router(),
            request(Method::GET, "http://tauri.localhost", Vec::new()),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
            "http://tauri.localhost"
        );
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(body["storage"], "in-memory");
    }

    #[tokio::test]
    async fn handles_preflight_without_dispatching_a_route() {
        let response = dispatch(
            router(),
            request(Method::OPTIONS, "http://tauri.localhost", Vec::new()),
        )
        .await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(response.body().is_empty());
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_ALLOW_HEADERS],
            "Content-Type"
        );
    }

    #[tokio::test]
    async fn rejects_untrusted_origins_without_reflecting_them() {
        let response = dispatch(
            router(),
            request(Method::GET, "https://example.com", Vec::new()),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(
            !response
                .headers()
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        );
    }

    #[tokio::test]
    async fn bounds_request_body_size() {
        let response = dispatch(
            router(),
            request(
                Method::POST,
                "http://tauri.localhost",
                vec![0; MAX_BODY_BYTES + 1],
            ),
        )
        .await;
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    async fn sqlite_call(
        router: &Router,
        method: Method,
        path: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let request = Request::builder()
            .method(method)
            .uri(format!("http://proof-api.localhost{path}"))
            .header(header::ORIGIN, "http://tauri.localhost")
            .header(header::CONTENT_TYPE, "application/json")
            .body(if body.is_null() {
                Vec::new()
            } else {
                serde_json::to_vec(&body).unwrap()
            })
            .unwrap();
        let response = dispatch(router.clone(), request).await;
        (
            response.status(),
            serde_json::from_slice(response.body()).unwrap(),
        )
    }

    #[tokio::test]
    async fn native_sqlite_contract_survives_host_recomposition() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("runs.sqlite3");
        let repository =
            local_stack_proof_persistence_sqlite::SqliteRunRepository::open(&path).unwrap();
        let router = local_stack_proof_transport_http::desktop_router(repository.service());
        let (status, health) =
            sqlite_call(&router, Method::GET, "/api/health", serde_json::Value::Null).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(health["storage"], "sqlite");
        let (status, invalid) = sqlite_call(&router, Method::POST, "/api/runs", serde_json::json!({"seed":42,"region":" ","threshold":0,"forceValidationFailure":false})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(invalid["field"], "region");
        let mut saved = Vec::new();
        for reject in [false, true] {
            let (status, queued) = sqlite_call(&router, Method::POST, "/api/runs", serde_json::json!({"seed":42,"region":"north","threshold":0,"forceValidationFailure":reject})).await;
            assert_eq!(status, StatusCode::CREATED);
            assert_eq!(queued["status"], "queued");
            let id = queued["id"].as_str().unwrap();
            let (status, terminal) = sqlite_call(
                &router,
                Method::POST,
                &format!("/api/runs/{id}/execute"),
                serde_json::Value::Null,
            )
            .await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(
                terminal["status"],
                if reject { "rejected" } else { "completed" }
            );
            assert_eq!(terminal["events"].as_array().unwrap().len(), 3);
            saved.push(terminal);
        }
        drop(router);
        let reopened = local_stack_proof_transport_http::desktop_router(
            local_stack_proof_persistence_sqlite::SqliteRunRepository::open(&path)
                .unwrap()
                .service(),
        );
        for terminal in &saved {
            let (status, fetched) = sqlite_call(
                &reopened,
                Method::GET,
                &format!("/api/runs/{}", terminal["id"].as_str().unwrap()),
                serde_json::Value::Null,
            )
            .await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(&fetched, terminal);
        }
        let (_, history) =
            sqlite_call(&reopened, Method::GET, "/api/runs", serde_json::Value::Null).await;
        assert_eq!(history, serde_json::json!(saved));
    }
}
