use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use instance_monitor::error::AppError;
use instance_monitor::metric_registry::MetricRegistry;
use instance_monitor::system_monitor::system_monitor;

#[derive(Debug, Clone)]
pub struct AppState {
    pub registry: MetricRegistry,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::init();

    let registry = MetricRegistry::default();

    // Start the system monitor
    system_monitor(registry.clone())?;

    // Api server
    let app = Router::new()
        .route("/", get(root))
        .route("/metrics", get(handle_request))
        .with_state(AppState { registry });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .map_err(|_| AppError::ListenerBindError)?;

    println!("Listening at http://0.0.0.0:8080");

    axum::serve(listener, app)
        .await
        .map_err(|_| AppError::ServerStartError)?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn handle_request(State(state): State<AppState>) -> Result<Response, (StatusCode, String)> {
    // Attempt to get metrics from the registry
    let metrics = state
        .registry
        .get_prometheus_metrics()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve metrics".to_string()))?;

    // Build a response with metrics
    Response::builder()
        .header("Content-Type", HeaderValue::from_static("text/plain"))
        .body(Body::from(metrics))
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response".to_string()))
}
