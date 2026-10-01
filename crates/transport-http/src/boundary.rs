//! Small local HTTP boundary. No payload or driver diagnostics enter logs.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use axum::extract::{MatchedPath, Request};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use super::ApiFailure;

const WEB_ORIGIN: &str = "http://127.0.0.1:5173";
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

pub async fn request_id(request: Request, next: Next) -> Response {
    let id = format!(
        "request-{}-{}",
        std::process::id(),
        NEXT_REQUEST.fetch_add(1, Ordering::Relaxed)
    );
    let method = request.method().clone();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or("unmatched", MatchedPath::as_str)
        .to_owned();
    let started = Instant::now();
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&id).expect("generated ASCII ID"),
    );
    eprintln!(
        "http {id} {method} {route} -> {} ({} ms)",
        response.status().as_u16(),
        started.elapsed().as_millis()
    );
    response
}

pub async fn web_origin(request: Request, next: Next) -> Response {
    let origin = request.headers().get(header::ORIGIN).cloned();
    if origin.as_ref().is_some_and(|value| value != WEB_ORIGIN) {
        return ApiFailure::new(
            StatusCode::FORBIDDEN,
            "forbidden-origin",
            "Web origin is not allowed",
        )
        .into_response();
    }
    let preflight = request.method() == Method::OPTIONS && origin.is_some();
    if preflight {
        let method = request.headers().get(header::ACCESS_CONTROL_REQUEST_METHOD);
        let headers = request
            .headers()
            .get(header::ACCESS_CONTROL_REQUEST_HEADERS);
        if !method.is_some_and(|m| m == "GET" || m == "POST")
            || headers.is_some_and(|h| {
                h.to_str().map_or(true, |text| {
                    text.split(',')
                        .any(|name| !name.trim().eq_ignore_ascii_case("content-type"))
                })
            })
        {
            return ApiFailure::new(
                StatusCode::FORBIDDEN,
                "forbidden-preflight",
                "Preflight request is not allowed",
            )
            .into_response();
        }
    }
    let mut response = if preflight {
        StatusCode::NO_CONTENT.into_response()
    } else {
        next.run(request).await
    };
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Origin"));
    if origin.is_some() {
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static(WEB_ORIGIN),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Content-Type"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            HeaderValue::from_static("x-request-id"),
        );
    }
    response
}
