use std::{collections::HashSet, fs, sync::Arc, time::Duration};

use axum::{
    extract::{ws::Message, Path, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::{thread_rng, Rng};
use serde::Serialize;
use tokio::{
    sync::{broadcast, Mutex, RwLock},
    time,
};
use tower_http::compression::CompressionLayer;

use crate::{
    fine_grained::Grid2,
    grid::{Grid, SubRectInfo},
    ws,
};

#[derive(Clone)]
pub struct AppState {
    grid: Arc<RwLock<Grid2>>,
    pub broadcast: Arc<Mutex<broadcast::Sender<Message>>>,
    pub queue: Arc<Mutex<PointQueue>>,
}

impl AppState {
    pub async fn toggle(&self, index: usize) -> bool {
        let mut grid = self.grid.write().await;
        let toggled = grid.toggle_item(index).await;
        self.push_index(index, toggled).await;
        println!("Got set checkbox to index {} {}", index, toggled);
        toggled
    }

    async fn push_index(&self, index: usize, toggled: bool) {
        let mut queue = self.queue.lock().await;
        if toggled {
            if queue.on.contains(&index) {
                return;
            } else if queue.off.contains(&index) {
                queue.on.remove(&index);
                queue.off.remove(&index);
            } else {
                queue.on.insert(index);
            }
        } else {
            if queue.off.contains(&index) {
                return;
            } else if queue.on.contains(&index) {
                queue.on.remove(&index);
                queue.off.remove(&index);
            } else {
                queue.off.insert(index);
            }
        }
    }
}

#[derive(Serialize)]
pub struct PointQueue {
    on: HashSet<usize>,
    off: HashSet<usize>,
}

impl PointQueue {
    fn new() -> Self {
        Self {
            on: HashSet::new(),
            off: HashSet::new(),
        }
    }

    fn clear(&mut self) {
        self.on.clear();
        self.off.clear();
    }

    fn is_empty(&self) -> bool {
        self.on.is_empty() && self.off.is_empty()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);

        let queue = Arc::new(Mutex::new(PointQueue::new()));
        let broadcast = Arc::new(Mutex::new(tx));
        tokio::spawn(broadcast_timer(queue.clone(), broadcast.clone()));

        let grid = Grid::new();
        AppState {
            grid: Arc::new(RwLock::new(grid)),
            broadcast,
            queue,
        }
    }

    pub async fn load(&mut self) {
        let mut grid = self.grid.write().await;
        let data = fs::read("dump.bin").unwrap_or_default();
        if let Ok(data) = BASE64_STANDARD.decode(&data) {
            if let Ok(data) = data.try_into() {
                grid.set_full(data).await
            }
        }
    }

    pub async fn save(&self) -> String {
        let grid = self.grid.read().await;
        BASE64_STANDARD.encode(grid.get_full().await)
    }
}

async fn broadcast_timer(
    queue: Arc<Mutex<PointQueue>>,
    tx: Arc<Mutex<broadcast::Sender<Message>>>,
) {
    let mut interval = time::interval(Duration::from_millis(5000));
    loop {
        interval.tick().await;
        let mut points = queue.lock().await;

        if points.is_empty() {
            continue;
        }

        let points2 = PointQueue {
            on: points.on.clone(),
            off: points.off.clone(),
        };

        let message = serde_json::to_string(&points2);
        match message {
            Ok(text) => {
                if let Err(err) = tx.lock().await.send(Message::Text(text)) {
                    println!("Failed to broadcast a message, {}", err);
                }
            }
            Err(_) => {
                continue;
            }
        }

        points.clear();
    }
}

/*
async fn get_grid(
    Path((from_index, to_index)): Path<(usize, usize)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let grid = state.grid.read().await;
    let item = grid.get_item(from_index);
    if let Some(item) = item {
        item.to_string()
    } else {
        "None".to_owned()
    }
}
*/

async fn set_checkbox(
    Path(index): Path<usize>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let toggled = state.toggle(index).await;

    if toggled {
        "1"
    } else {
        "0"
    }
}

async fn index() -> impl IntoResponse {
    Html(
        r#"""
    <html>
        <head><title>Grid shit</title></head>
        <body>
            <h1>Here'll be dynamic grid</h1>
        </body>
    </html>
    """#,
    )
}

async fn full_grid(State(state): State<AppState>) -> impl IntoResponse {
    let grid = state.grid.read().await;
    let full = grid.get_full().await;

    BASE64_STANDARD.encode(full)
}

async fn sub_grid(State(state): State<AppState>) -> impl IntoResponse {
    let grid = state.grid.read().await;
    let x_shift = rand::thread_rng().gen_range(0..(125 - 10));
    let y_shift = rand::thread_rng().gen_range(0..(1000 - 80));
    let subgrid = grid.get_rect(x_shift, y_shift, 10, 80).await;
    let subgrid2 = SubRectInfoJson::from_info(&subgrid);
    Json(subgrid2)
}

#[derive(Serialize)]
pub struct SubRectInfoJson {
    pub data: String,
    pub x_shift: usize,
    pub y_shift: usize,
    pub width: usize,
    pub height: usize,
    pub canvas_width: usize,
}

impl SubRectInfoJson {
    fn from_info(info: &SubRectInfo) -> Self {
        Self {
            data: BASE64_STANDARD.encode(info.data.clone()),
            x_shift: info.x_shift,
            y_shift: info.y_shift,
            width: info.width,
            height: info.height,
            canvas_width: info.canvas_width,
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ws", get(ws::ws_grid))
        .route("/api/grid", get(full_grid))
        .route("/api/subgrid", get(sub_grid))
        .route("/set/:index", post(set_checkbox))
        //.route("/grid/:from/:to", get(get_grid))
        .route("/", get(index))
        .layer(CompressionLayer::new())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use crate::server::PointQueue;

    #[test]
    fn json_test() {
        let mut pq = PointQueue::new();
        pq.off.push(1111);
        pq.on.push(323123);
        let result = serde_json::to_string(&pq);
        dbg!(result);
    }
}
