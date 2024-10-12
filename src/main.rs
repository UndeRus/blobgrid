use std::{fs, net::SocketAddr};

use clap::Parser;
use config::Cli;
use server::router;
use state::AppState;
use tokio::signal;

mod bit_utils;
mod config;
mod fine_grained;
mod grid;
mod grid1;
mod server;
mod state;
mod ws;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut state = AppState::new();
    println!("Loading data");
    state.load().await;

    tokio::spawn(periodic_save(state.clone()));

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

async fn periodic_save(state: AppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(30000));
    loop {
        interval.tick().await;
        println!("Saving backup at {:?}", tokio::time::Instant::now());
        state.save().await;
        state.save_png("dump.png").await;
    }
}
