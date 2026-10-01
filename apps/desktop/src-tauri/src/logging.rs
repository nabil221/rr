//! Capped local host diagnostics, never payloads, credentials, IDs, or paths.

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

const MAX_BYTES: u64 = 1_048_576;

pub struct LocalLog(Mutex<File>);

impl LocalLog {
    pub fn open(directory: &Path) -> io::Result<Self> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(directory.join("desktop.log"))
            .map(|file| Self(Mutex::new(file)))
            .map_err(|_| io::Error::other("Desktop local log is unavailable"))
    }

    fn write(&self, record: &str) {
        if let Ok(mut file) = self.0.lock() {
            // Stop writing at the cap; never truncate or rotate existing user data.
            if file
                .metadata()
                .is_ok_and(|metadata| metadata.len() < MAX_BYTES)
            {
                let _ = writeln!(file, "{record}");
            }
        }
    }

    pub fn started(&self) {
        self.write("desktop started storage=sqlite transport=embedded-protocol");
    }

    pub fn request(&self, method: &axum::http::Method, route: &str, status: u16, elapsed_ms: u128) {
        let method = match method.as_str() {
            "GET" => "GET",
            "POST" => "POST",
            "OPTIONS" => "OPTIONS",
            _ => "OTHER",
        };
        self.write(&format!(
            "{method} {route} status={status} elapsed_ms={elapsed_ms}"
        ));
    }
}

pub fn route(path: &str) -> &'static str {
    match path {
        "/api/health" => "/api/health",
        "/api/runs" => "/api/runs",
        path if path.starts_with("/api/runs/") => {
            let tail = &path["/api/runs/".len()..];
            if tail.ends_with("/execute") && tail.matches('/').count() == 1 {
                "/api/runs/{id}/execute"
            } else if !tail.contains('/') && !tail.is_empty() {
                "/api/runs/{id}"
            } else {
                "unmatched"
            }
        }
        _ => "unmatched",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_safe_routes_and_preserves_capped_logs() {
        let directory = tempfile::tempdir().unwrap();
        let log = LocalLog::open(directory.path()).unwrap();
        log.started();
        log.request(
            &axum::http::Method::POST,
            route("/api/runs/private-id/execute"),
            200,
            3,
        );
        log.request(&axum::http::Method::GET, route("/private/secret"), 404, 1);
        let path = directory.path().join("desktop.log");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("storage=sqlite"));
        assert!(text.contains("/api/runs/{id}/execute"));
        assert!(!text.contains("private-id") && !text.contains("secret"));
        // Windows append-only handles intentionally cannot truncate/extend files.
        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(MAX_BYTES)
            .unwrap();
        log.started();
        assert_eq!(std::fs::metadata(path).unwrap().len(), MAX_BYTES);
    }
}
