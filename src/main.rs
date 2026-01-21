mod agents;
mod api;
mod envs;
mod infra;
mod state;

use std::sync::Arc;

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    // 1. Initialize Tracing (Logging)
    if let Err(e) = infra::telemetry::init_tracing().await {
        eprintln!("Failed to initialize tracing: {}", e);
    }

    // 2. Initialize Orchestrator
    let orchestrator = agents::orchestrator::Orchestrator::new();

    // 2.1 Initialize Redis
    let redis_provider = infra::redis::RedisProvider::new()
        .await
        .expect("Failed to initialize Redis");

    // 3. Initialize State
    let state = Arc::new(state::AppState::new(orchestrator, redis_provider));

    // 4. Setup Router
    let app = api::routes::app_router(state);

    // 5. Start Server
    let config = envs::get();
    let port = config.port;
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
