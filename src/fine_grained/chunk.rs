use tokio::sync::{Mutex, RwLock};

use crate::bit_utils::{get_bit, set_bit, toggle_bit};

use super::CHUNK_SIZE;

pub struct Chunk {
    pub(crate) data: RwLock<[u8; CHUNK_SIZE]>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            data: RwLock::new([0; CHUNK_SIZE]), // Initialize the data to zeros
        }
    }

    pub (crate) async fn load_from_slice(&self, data: &[u8]) {
        let mut chunk_data = self.data.write().await;
        chunk_data.copy_from_slice(data);
    }

    pub (crate) async fn toggle_bit(&self, byte_offset: usize, bit_position: usize) -> bool {
        let mut chunk_data = self.data.write().await;
        
        if bit_position < 8 {
            let new_byte = toggle_bit(chunk_data[byte_offset], bit_position);
            chunk_data[byte_offset] = new_byte;
            return get_bit(new_byte, bit_position);
        }
        false
    }

    async fn set_bit(&self, byte_offset: usize, bit_position: usize, value: bool) {
        let mut chunk_data = self.data.write().await;
        if bit_position < 8 {
            chunk_data[byte_offset] = set_bit(chunk_data[byte_offset], bit_position, value);
        }
    }

    async fn read_bit(&self, byte_offset: usize, bit_position: usize) -> bool {
        let chunk_data = self.data.read().await;
        if bit_position < 8 {
            get_bit(chunk_data[byte_offset], bit_position)
        } else {
            false
        }
    }
}
