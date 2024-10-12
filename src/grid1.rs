use crate::{
    bit_utils::{get_bit, set_bit, toggle_bit},
    grid::{Grid, SubRectInfo, MAX_SIZE},
};

pub struct Grid1 {
    blob: [u8; MAX_SIZE],
}

impl Grid for Grid1 {
    fn new() -> Self {
        Grid1 {
            blob: [0; MAX_SIZE],
        }
    }

    async fn get_full(&self) -> [u8; MAX_SIZE] {
        self.blob
    }

    async fn set_full(&mut self, data: [u8; MAX_SIZE]) {
        self.blob = data;
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
        let canvas_width_in_bytes = canvas_width / 8;

        for y in 0..height {
            for x in 0..bytes_width {
                let global_y = bytes_y + y;
                let global_x = bytes_x + x;
                let index = global_y * canvas_width_in_bytes + global_x;
                result[bytes_width * y + x] = self.blob[index];
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

    async fn toggle_item(&mut self, index: usize) -> bool {
        if index >= MAX_SIZE * 8 {
            return false;
        }
        let cell_index = index / 8;
        let bit_index = index % 8;
        let cell = self.blob[cell_index];

        let toggled_byte = toggle_bit(cell, bit_index);
        self.blob[cell_index] = toggled_byte;
        get_bit(toggled_byte, bit_index)
        //dbg!(cell_index, self.blob[cell_index]);
    }
}

impl Grid1 {
    pub fn get_item(&self, index: usize) -> Option<bool> {
        if index >= MAX_SIZE {
            None
        } else {
            let cell_index = index / 8;
            let bit_index = index % 8;
            let cell = self.blob[cell_index];
            Some(get_bit(cell, bit_index))
        }
    }

    fn get_range(&self, from_index: usize, to_index: usize) -> Vec<bool> {
        if to_index < from_index {
            return vec![];
        }
        if to_index == from_index {
            return self.get_item(to_index).map(|b| vec![b]).unwrap_or_default();
        }

        let from_cell_index = from_index / 8;
        let from_bit_index = from_index % 8;

        let to_cell_index = to_index / 8;
        let to_bit_index = to_index % 8;

        //TODO: implement
        vec![]
    }

    pub fn set_item(&mut self, index: usize, value: bool) {
        if index >= MAX_SIZE * 8 {
            return;
        }
        let cell_index = index / 8;
        let bit_index = index % 8;
        let cell = self.blob[cell_index];
        self.blob[cell_index] = set_bit(cell, bit_index, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn grid_test() {
        let mut grid = Grid1::new();

        let b4 = grid.get_item(10);
        assert_eq!(b4, Some(false));

        grid.set_item(0, true);
        assert_eq!(Some(true), grid.get_item(0));

        let mut rect = grid.get_rect(0, 0, 10, 10).await;
        dbg!(rect);

        rect = grid.get_rect(100, 100, 10, 10).await;
        dbg!(rect);
    }
}
