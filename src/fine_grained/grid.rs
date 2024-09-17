use std::array;

use base64::{prelude::BASE64_STANDARD, Engine};

use super::{chunk::Chunk, CHUNK_SIZE, MAX_SIZE, NUM_CHUNKS};

pub struct Grid2 {
    chunks: [Chunk; NUM_CHUNKS], // Array of chunks, each with its own RwLock on data
}

impl Grid2 {
    pub fn new() -> Self {
        let chunks = array::from_fn(|_| Chunk::new());
        Self { chunks }
    }

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

    pub async fn get_full(&self) -> [u8; MAX_SIZE] {
        let mut full_blob: Vec<u8> = Vec::with_capacity(MAX_SIZE);

        for chunk in &self.chunks {
            let chunk_data = chunk.data.read().await;
            full_blob.extend_from_slice(&chunk_data.clone());
        }

        full_blob.try_into().unwrap()
    }

    pub async fn set_full(&mut self, data: [u8; MAX_SIZE]) {
        // Iterate over chunks and load data into each
        for (i, chunk) in self.chunks.iter().enumerate() {
            let start = i * CHUNK_SIZE;
            let end = start + CHUNK_SIZE;
            chunk.load_from_slice(&data[start..end]).await;
        }
    }

    pub async fn toggle_item(&self, bit_index: usize) -> bool {
        let (byte_index, bit_position) = Self::get_bit_info(bit_index); // Get byte and bit position
        let (chunk_index, offset_within_chunk) = Self::get_chunk_info(byte_index); // Get chunk and byte offset

        let chunk = &self.chunks[chunk_index]; // Get the specific chunk
        chunk.toggle_bit(offset_within_chunk, bit_position).await // Toggle the bit in the chunk
    }
}
