#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logging;
mod protocol;
mod storage;

use tauri::Manager;

struct DesktopState {
    router: axum::Router,
    log: std::sync::Arc<logging::LocalLog>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let default = app
                .path()
                .app_local_data_dir()
                .map_err(|_| storage::unavailable())?;
            let directory =
                storage::data_directory(default, std::env::var_os("LOCAL_STACK_PROOF_DATA_DIR"))?;
            let service = storage::service(&directory)?;
            let log = std::sync::Arc::new(logging::LocalLog::open(&directory)?);
            log.started();
            app.manage(DesktopState {
                router: local_stack_proof_transport_http::desktop_router(service),
                log,
            });
            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol(
            "proof-api",
            move |context, request, responder| {
                let state = context.app_handle().state::<DesktopState>();
                let router = state.router.clone();
                let log = state.log.clone();
                let method = request.method().clone();
                let route = logging::route(request.uri().path());
                tauri::async_runtime::spawn(async move {
                    let started = std::time::Instant::now();
                    let response = protocol::dispatch(router, request).await;
                    log.request(
                        &method,
                        route,
                        response.status().as_u16(),
                        started.elapsed().as_millis(),
                    );
                    responder.respond(response);
                });
            },
        )
        .run(tauri::generate_context!())
        .expect("desktop host failed to start");
}
