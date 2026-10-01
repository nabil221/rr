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
}
