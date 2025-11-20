use std::net::SocketAddr;
use tracing::info;
use crate::shutdown::shutdown;

mod state;
mod http;
mod model;
mod shutdown;
mod log;

#[tokio::main]
async fn main() {
    // логи
    let (nb, _quard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .with_writer(nb)
        .flatten_event(true)
        .init();

    // состояние приложения
    let app_state = state::AppState::default();

    // собираем роутер из модуля http::routes
    let app = http::routes::router(app_state);

    let addr: SocketAddr = "0.0.0.0:8085".parse().unwrap();
    info!("listening on http://{addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .with_graceful_shutdown(shutdown())
        .await
        .unwrap();
}
