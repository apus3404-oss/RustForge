use crate::api::{handlers, websocket, AppState};
use crate::error::Result;
use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::CorsLayer;

/// Start the API server
pub async fn start_server(state: AppState, port: u16) -> Result<()> {
    let app = Router::new()
        // Workflow routes
        .route(
            "/api/workflows",
            post(handlers::workflows::create_workflow),
        )
        .route("/api/workflows", get(handlers::workflows::list_workflows))
        .route(
            "/api/workflows/:id",
            get(handlers::workflows::get_workflow),
        )
        .route(
            "/api/workflows/:id",
            delete(handlers::workflows::delete_workflow),
        )
        // Execution routes
        .route(
            "/api/workflows/:id/execute",
            post(handlers::executions::execute_workflow),
        )
        .route(
            "/api/executions/:id",
            get(handlers::executions::get_execution),
        )
        .route(
            "/api/executions",
            get(handlers::executions::list_executions),
        )
        .route(
            "/api/executions/:id/pause",
            post(handlers::executions::pause_execution),
        )
        .route(
            "/api/executions/:id/resume",
            post(handlers::executions::resume_execution),
        )
        .route(
            "/api/executions/:id",
            delete(handlers::executions::cancel_execution),
        )
        // WebSocket
        .route(
            "/api/ws/executions/:id",
            get(websocket::websocket_handler),
        )
        // Middleware
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| crate::error::Error::Internal(format!("Failed to bind to {}: {}", addr, e)))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::Error::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_module_exists() {
        // Basic smoke test to ensure module compiles
        assert!(true);
    }
}
