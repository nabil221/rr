mod config;

use std::error::Error;

use local_stack_proof_application::in_memory_run_service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => eprintln!("warning: could not load .env: {error}"),
    }

    let config = config::ApiConfig::from_env().map_err(std::io::Error::other)?;
    let listener = tokio::net::TcpListener::bind(config.bind_address).await?;
    println!(
        "In-memory Local Stack Proof API listening on http://{}",
        config.bind_address
    );

    axum::serve(
        listener,
        local_stack_proof_transport_http::router(in_memory_run_service()),
    )
    .await?;
    Ok(())
}
