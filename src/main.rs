use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod state;
mod http;

#[tokio::main]
async fn main() {
    // логи
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).with_target(false).compact().init();

    // состояние приложения
    let app_state = state::AppState::default();

    // собираем роутер из модуля http::routes
    let app = http::routes::router(app_state);

    let addr: SocketAddr = "0.0.0.0:8085".parse().unwrap();
    info!("listening on http://{addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}
