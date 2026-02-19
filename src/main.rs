#![deny(rust_2018_idioms, unsafe_code)]

use bile::{Bile, config::Config};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[tokio::main]
async fn main() -> bile::error::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                concat!(
                    env!("CARGO_CRATE_NAME"),
                    "=debug,tower_http=debug,axum::rejection=trace",
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(tracing_error::ErrorLayer::default())
        .init();

    tracing::info!("starting bile");

    let config = Config::load()?;

    if !config.project_root.exists() {
        tracing::warn!(path=?config.project_root.display(), "configured project_root does not exist");
    }
    if config.project_root.read_dir()?.next().is_none() {
        tracing::warn!(path=?config.project_root.display(), "configured project_root is empty");
    }

    let addr = format!("[::]:{}", config.port);

    let bile = Bile::init(config.finalize()?);

    let app = bile.routes();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
