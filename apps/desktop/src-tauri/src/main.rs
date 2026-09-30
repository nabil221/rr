#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod protocol;
mod storage;

use tauri::Manager;

struct DesktopState(axum::Router);

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
            app.manage(DesktopState(
                local_stack_proof_transport_http::desktop_router(service),
            ));
            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol(
            "proof-api",
            move |context, request, responder| {
                let router = context.app_handle().state::<DesktopState>().0.clone();
                tauri::async_runtime::spawn(async move {
                    responder.respond(protocol::dispatch(router, request).await);
                });
            },
        )
        .run(tauri::generate_context!())
        .expect("desktop host failed to start");
}
