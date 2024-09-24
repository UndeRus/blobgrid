use std::array;

use base64::{prelude::BASE64_STANDARD, Engine};

use crate::grid::{Grid, SubRectInfo};

use super::{chunk::Chunk, CHUNK_SIZE, MAX_SIZE, NUM_CHUNKS};

pub struct Grid2 {
    chunks: [Chunk; NUM_CHUNKS], // Array of chunks, each with its own RwLock on data
}

impl Grid for Grid2 {
    fn new() -> Self {
        let chunks = array::from_fn(|_| Chunk::new());
        Self { chunks }
    }


    async fn get_full(&self) -> [u8; MAX_SIZE] {
        let mut full_blob: Vec<u8> = Vec::with_capacity(MAX_SIZE);

        for chunk in &self.chunks {
            let chunk_data = chunk.data.read().await;
            full_blob.extend_from_slice(&chunk_data.clone());
        }

        full_blob.try_into().unwrap()
    }

    async fn set_full(&mut self, data: [u8; MAX_SIZE]) {
        // Iterate over chunks and load data into each
        for (i, chunk) in self.chunks.iter().enumerate() {
            let start = i * CHUNK_SIZE;
            let end = start + CHUNK_SIZE;
            chunk.load_from_slice(&data[start..end]).await;
        }
    }

    async fn toggle_item(&mut self, bit_index: usize) -> bool {
        let (byte_index, bit_position) = Self::get_bit_info(bit_index); // Get byte and bit position
        let (chunk_index, offset_within_chunk) = Self::get_chunk_info(byte_index); // Get chunk and byte offset

        let chunk = &self.chunks[chunk_index]; // Get the specific chunk
        chunk.toggle_bit(offset_within_chunk, bit_position).await // Toggle the bit in the chunk
    }

    async fn get_rect(
        &self,
        bytes_x: usize,
        bytes_y: usize,
        bytes_width: usize,
        height: usize,
    ) -> SubRectInfo {
        let mut result = vec![0; bytes_width * height];
        let canvas_width = 1000;
        for y in 0..height {
            let global_y = bytes_y + y;
            let row = self.chunks[global_y].data.read().await;
            for x in 0..bytes_width {
                let global_x = bytes_x + x;
                result[bytes_width * y + x] = row[global_x]
            }
        }
        SubRectInfo {
            data: result,
            x_shift: bytes_x * 8,
            y_shift: bytes_y,
            width: bytes_width * 8,
            height: height,
            canvas_width: canvas_width,
        }
    }
}

impl Grid2 {
    
    fn get_chunk_info(byte_index: usize) -> (usize, usize) {
        let chunk_index = byte_index / CHUNK_SIZE;
        let offset_within_chunk = byte_index % CHUNK_SIZE;
        (chunk_index, offset_within_chunk)
    }

    fn get_bit_info(bit_index: usize) -> (usize, usize) {
        let byte_index = bit_index / 8;
        let bit_position = bit_index % 8;
        (byte_index, bit_position)
    }
}
