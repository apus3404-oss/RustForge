use crate::api::{handlers, websocket, AppState};
use crate::error::Result;
use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

/// Start the API server
pub async fn start_server(state: AppState, port: u16) -> Result<()> {
    // Serve static files from dist/ui with SPA fallback
    let serve_dir = ServeDir::new("dist/ui")
        .not_found_service(ServeFile::new("dist/ui/index.html"));

    let app = Router::new()
        // API routes - must come before static file serving
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
        .route(
            "/api/ws/executions/:id",
            get(websocket::websocket_handler),
        )
        .with_state(state)
        // Static files and SPA fallback - must come after API routes
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting API server on {}", addr);
    tracing::info!("Serving UI from dist/ui");

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
