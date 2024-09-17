const MAX_SIZE: usize = 125000; // 125 KB
const CHUNK_SIZE: usize = 1000; // 1 KB chunk size
const NUM_CHUNKS: usize = MAX_SIZE / CHUNK_SIZE; //Number of chunks (125 for 125 KB)

mod chunk;
mod grid;

pub use grid::Grid2;