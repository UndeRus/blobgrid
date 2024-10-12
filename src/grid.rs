use std::fmt::Debug;

use serde::Serialize;

use crate::bit_utils::{get_bit, set_bit, toggle_bit};

pub const MAX_SIZE: usize = 125000;

pub trait Grid {
    fn new() -> Self;

    async fn get_full(&self) -> [u8; MAX_SIZE];
    async fn set_full(&mut self, data: [u8; MAX_SIZE]);

    async fn get_rect(
        &self,
        bytes_x: usize,
        bytes_y: usize,
        bytes_width: usize,
        height: usize,
    ) -> SubRectInfo;

    async fn toggle_item(&mut self, index: usize) -> bool;
}

#[derive(Debug, Serialize)]
pub struct SubRectInfo {
    pub data: Vec<u8>,
    pub x_shift: usize,
    pub y_shift: usize,
    pub width: usize,
    pub height: usize,
    pub canvas_width: usize,
}
