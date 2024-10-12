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
    state::AppState,
    ws,
};

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
    use image::{buffer, GenericImage, GenericImageView};

    use crate::{
        bit_utils::{get_bit, set_bit},
        fine_grained::Grid2,
        grid::{Grid, MAX_SIZE},
        state::PointQueue,
    };

    #[test]
    fn json_test() {
        let mut pq = PointQueue::new();
        pq.off.insert(1111);
        pq.on.insert(323123);
        let result = serde_json::to_string(&pq);
        dbg!(result);
    }

    #[tokio::test]
    async fn create_png() {
        let grid = Grid2::new();

        let buffer = grid.get_full().await;

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

        imgbuf.save("dump.png").unwrap();
    }

    #[tokio::test]
    async fn load_png() {
        let mut grid = Grid2::new();

        let empty_color = image::Rgba([255u8, 255u8, 255u8, 255u8]);

        let img = image::open("dump.png").unwrap();

        let mut buffer: [u8; MAX_SIZE] = [0; MAX_SIZE];

        for i in 0..MAX_SIZE * 8 {
            let bit_index = i % 8;
            let byte_index = i / 8;

            let x = (i % 1000) as u32;
            let y = (i / 1000) as u32;

            let byte = buffer[byte_index];

            //imgbuf.put_pixel(x as u32, y as u32, if bit_value { filled_color } else { empty_color });
            let color = img.get_pixel(x, y);
            let byte = if color == empty_color {
                set_bit(byte, bit_index, false)
            } else {
                set_bit(byte, bit_index, true)
            };

            buffer[byte_index] = byte;
        }

        grid.set_full(buffer).await;
    }
}
