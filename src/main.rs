use std::{fs, net::SocketAddr};

use clap::Parser;
use config::Cli;
use server::{router, AppState};
use tokio::signal;

mod config;
mod grid;
mod server;
mod ws;
mod fine_grained;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut state = AppState::new();
    println!("Loading data");
    state.load().await;
    let app = router(state.clone());
    println!("Starting");
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", cli.port.unwrap_or(3000)))
        .await
        .expect("Failed to create listener");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal(state))
    .await
    .expect("Failed to start server");
}

async fn shutdown_signal(state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("Dumping data");
            fs::write("dump.bin", state.save().await);
            println!("Finished");
        },
        _ = terminate => {
            fs::write("dump.bin", state.save().await);
            println!("Finished")
        },
    }
}