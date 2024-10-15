use std::{collections::HashSet, fs, sync::Arc, time::Duration};

use axum::extract::ws::Message;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::Serialize;
use tokio::{
    sync::{broadcast, Mutex, RwLock},
    time,
};

use crate::{
    bit_utils::get_bit,
    fine_grained::Grid2,
    grid::{Grid, MAX_SIZE},
};

#[derive(Clone)]
pub struct AppState {
    pub dump_path: String,
    pub bitmap_path: String,
    pub grid: Arc<RwLock<Grid2>>,
    pub broadcast: Arc<Mutex<broadcast::Sender<Message>>>,
    pub queue: Arc<Mutex<PointQueue>>,
}

impl AppState {
    pub async fn toggle(&self, index: usize) -> bool {
        let mut grid = self.grid.write().await;
        let toggled = grid.toggle_item(index).await;
        self.push_index(index, toggled).await;
        log::info!("Got set checkbox to index {} {}", index, toggled);
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
    pub on: HashSet<usize>,
    pub off: HashSet<usize>,
}

impl PointQueue {
    pub fn new() -> Self {
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
    pub fn new(dump_path: &str, bitmap_path: &str) -> Self {
        let (tx, _) = broadcast::channel(100);

        let queue = Arc::new(Mutex::new(PointQueue::new()));
        let broadcast = Arc::new(Mutex::new(tx));
        tokio::spawn(broadcast_timer(queue.clone(), broadcast.clone()));

        let grid = Grid::new();
        AppState {
            dump_path: dump_path.to_owned(),
            bitmap_path: bitmap_path.to_owned(),
            grid: Arc::new(RwLock::new(grid)),
            broadcast,
            queue,
        }
    }

    pub async fn load(&mut self) {
        let mut grid = self.grid.write().await;
        let data = fs::read(&self.dump_path).unwrap_or_default();
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

    pub async fn save_png(&self, filename: &str) {
        let buffer = self.grid.read().await.get_full().await;

        let mut imgbuf = image::ImageBuffer::new(1000, 1000);

        let filled_color = image::Rgb([255u8, 0u8, 0u8]);
        let empty_color = image::Rgb([255u8, 255u8, 255u8]);

        for i in 0..MAX_SIZE * 8 {
            let bit_index = i % 8;
            let byte_index = i / 8;
            let byte = buffer[byte_index];
            let bit_value = get_bit(byte, bit_index);

            let x = i % 1000;
            let y = i / 1000;
            imgbuf.put_pixel(
                x as u32,
                y as u32,
                if bit_value { filled_color } else { empty_color },
            );
        }

        imgbuf.save(filename);
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
                    log::warn!("Failed to broadcast a message, {}", err);
                }
            }
            Err(_) => {
                continue;
            }
        }

        points.clear();
    }
}
