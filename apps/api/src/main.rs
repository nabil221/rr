mod config;

use std::error::Error;

use local_stack_proof_persistence_postgres::PostgresRunRepository;

fn main() -> Result<(), Box<dyn Error>> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => {
            return Err(std::io::Error::other("Could not load local .env configuration").into());
        }
    }

    let config = config::ApiConfig::from_env().map_err(std::io::Error::other)?;
    // Retain this owner until after the executor shuts down: the synchronous
    // driver's internal runtime must be created and dropped outside async code.
    let service = PostgresRunRepository::connect(&config.database_url)?.service();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind(config.bind_address).await?;
        println!(
            "Postgres Local Stack Proof API listening on http://{}",
            config.bind_address
        );
        axum::serve(
            listener,
            local_stack_proof_transport_http::web_router(service.clone()),
        )
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    })?;
    Ok(())
}
