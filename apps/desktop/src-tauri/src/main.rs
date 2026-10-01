#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod protocol;

fn main() {
    let router = local_stack_proof_transport_http::router(
        local_stack_proof_application::in_memory_run_service(),
    );

    tauri::Builder::default()
        .register_asynchronous_uri_scheme_protocol(
            "proof-api",
            move |_context, request, responder| {
                let router = router.clone();
                tauri::async_runtime::spawn(async move {
                    responder.respond(protocol::dispatch(router, request).await);
                });
            },
        )
        .run(tauri::generate_context!())
        .expect("desktop host failed to start");
}
