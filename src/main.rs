use std::net::SocketAddr;
use std::sync::Arc;
use mimalloc::MiMalloc;
use tracing::info;
use crate::shutdown::shutdown;
use tokio::net::TcpListener;
use socket2::{Domain, Protocol, Socket, Type};

mod state;
mod http;
mod model;
mod shutdown;
mod log;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn bind(addr: SocketAddr, backlog: i32) -> std::io::Result<TcpListener> {
    let domain = match addr {
        SocketAddr::V4(..) => Domain::IPV4,
        SocketAddr::V6(..) => Domain::IPV6,
    };
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
    let _ = socket.set_reuse_port(true);
    socket.set_nonblocking(true)?;
    socket.bind(&addr.into())?;
    socket.listen(backlog)?;
    TcpListener::from_std(socket.into())
}

// #[tokio::main(flavor = "multi_thread")]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // логи
//     let (nb, _quard) = tracing_appender::non_blocking(std::io::stdout());
//     tracing_subscriber::fmt()
//         .with_env_filter("info")
//         .json()
//         .with_writer(nb)
//         .flatten_event(true)
//         .init();
//
//     // состояние приложения
//     let app_state = state::AppState::default();
//
//     // собираем роутер из модуля http::routes
//     let app = http::routes::router(app_state);
//
//     let addr: SocketAddr = "0.0.0.0:8085".parse()?;
//     let listener = bind(addr, 8192)?;
//     info!("listening on http://{addr}");
//     axum::serve(listener, app)
//         .with_graceful_shutdown(shutdown())
//         .await?;
//     Ok(())
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Инициализируем логи один раз для всего процесса
    let (nb, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .json()
        .with_writer(nb)
        .flatten_event(true)
        .init();

    let addr: SocketAddr = "0.0.0.0:8085".parse()?;

    // 2. Шарим состояние через Arc (оно должно быть потокобезопасным)
    let app_state = Arc::new(state::AppState::default());

    // 3. Определяем количество ядер (для Asus F3Ke это обычно 2)
    let num_cores = num_cpus::get();
    info!("Starting server on {} cores", num_cores);

    let mut handles = Vec::new();

    for core_id in 0..num_cores {
        let state = Arc::clone(&app_state);

        handles.push(std::thread::spawn(move || {
            // 4. Поднимаем ЛОКАЛЬНЫЙ рантайм для каждого ядра
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build runtime");

            rt.block_on(async {
                // Внутри каждого потока свой инстанс роутера
                let app = http::routes::router(state);

                // Внутри каждого потока свой листенер на ТОМ ЖЕ порту
                let listener = bind(addr, 8192).expect("Failed to bind");

                info!(core = core_id, "Worker listening on http://{}", addr);

                axum::serve(listener, app)
                    .with_graceful_shutdown(shutdown())
                    .await
                    .expect("Server failed");
            });
        }));
    }

    // Ждем, пока все потоки отработают (вечно)
    for handle in handles {
        handle.join().map_err(|e| format!("Thread panic: {:?}", e))?;
    }

    Ok(())
}
