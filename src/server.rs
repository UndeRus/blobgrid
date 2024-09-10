use std::sync::Arc;

use axum::{extract::{Path, State}, response::{Html, IntoResponse}, routing::{get, post}, Router};
use base64::{prelude::BASE64_STANDARD, Engine};
use tokio::sync::RwLock;
use tower_http::compression::CompressionLayer;

use crate::grid::Grid;

#[derive(Clone)]
pub struct AppState {
    grid: Arc<RwLock<Grid>>
}

impl AppState {
    fn new() -> Self {
        AppState {
            grid: Arc::new(RwLock::new(Grid::new()))
        }
    }
}

async fn get_grid(Path((from_index, to_index), ) : Path<(usize, usize)>, State(state): State<AppState>) -> impl IntoResponse {
    let grid = state.grid.read().await;
    let item = grid.get_item(from_index);
    if let Some(item) = item {
        item.to_string()
    } else {
        "None".to_owned()
    }
}



async fn set_checkbox(Path(index) : Path<usize>, State(state): State<AppState>) -> impl IntoResponse {
    println!("Got set checkbox to index {}", index);
    let mut grid = state.grid.write().await;
    grid.set_item(index, true);
}

async fn index() -> impl IntoResponse {
    Html(r#"""
    <html>
        <head><title>Grid shit</title></head>
        <body>
            <h1>Here'll be dynamic grid</h1>
        </body>
    </html>
    """#)
}

async fn full_grid(State(state): State<AppState>) -> impl IntoResponse {
    let grid = state.grid.read().await;
    let full = grid.get_full();

    BASE64_STANDARD.encode(full)
}

pub fn router() -> Router {
    Router::new()
    .route("/", get(index))
    .route("/grid", get(full_grid))
    .route("/set/:index", post(set_checkbox))
    .route("/grid/:from/:to",get( get_grid))
    .layer(CompressionLayer::new())
    .with_state(AppState::new())
}