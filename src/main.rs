use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use instance_monitor::error::AppError;
use instance_monitor::metric_registry::MetricRegistry;
use instance_monitor::system_monitor::system_monitor;

#[derive(Parser, Debug)]
#[command(
    name = "instance_monitor",
    about = "A simple monitor for system metrics"
)]
struct Cli {
    #[arg(long, default_value_t = 8080)]
    port: u16,

    #[arg(long, default_value_t = 5)]
    delay: u64,

    #[arg(long, default_value_t = String::from("error"))]
    log_level: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub registry: MetricRegistry,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.log_level.parse().unwrap_or(log::LevelFilter::Error))
        .init();

    let registry = MetricRegistry::default();

    // Start the system monitor
    system_monitor(registry.clone(), cli.delay)?;

    // Api server
    let app = Router::new()
        .route("/", get(root))
        .route("/metrics", get(handle_request))
        .with_state(AppState { registry });

    let socket = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
        cli.port,
    );

    let listener = tokio::net::TcpListener::bind(socket)
        .await
        .map_err(|_| AppError::ListenerBindError)?;

    println!(
        "Listening at: http://0.0.0.0:{port}\nMetrics endpoint: http://0.0.0.0:{port}/metrics",
        port = cli.port
    );

    axum::serve(listener, app)
        .await
        .map_err(|_| AppError::ServerStartError)?;

    Ok(())
}

async fn root() -> &'static str {
    "A Simple Instance Monitor."
}

pub async fn handle_request(
    State(state): State<AppState>,
) -> Result<Response, (StatusCode, String)> {
    // Attempt to get metrics from the registry
    let metrics = state.registry.get_prometheus_metrics().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve metrics".to_string(),
        )
    })?;

    // Build a response with metrics
    Response::builder()
        .header("Content-Type", HeaderValue::from_static("text/plain"))
        .body(Body::from(metrics))
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to build response".to_string(),
            )
        })
}
