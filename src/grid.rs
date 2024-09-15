const MAX_SIZE: usize = 125000;

pub struct Grid {
    blob: [u8; MAX_SIZE],
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            blob: [0; MAX_SIZE],
        }
    }

    pub fn get_full(&self) -> [u8; MAX_SIZE] {
        self.blob
    }
    
    pub fn set_full(&mut self, data: [u8; MAX_SIZE]) {
        self.blob = data;
    }

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

    pub fn get_range(&self, from_index: usize, to_index: usize) -> Vec<bool> {
        todo!();
        if to_index < from_index {
            return vec![];
        }
        if to_index == from_index {
            return self.get_item(to_index).map(|b|vec![b]).unwrap_or_default();
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

    pub fn toggle_item(&mut self, index: usize) -> bool {
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

#[inline]
fn get_bit(byte: u8, bit_index: usize) -> bool {
    let bit_mask = 0x1;
    let shifted_bit = byte >> bit_index;
    (shifted_bit & bit_mask) == 1
}

/*
#[inline]
fn get_bit_range(byte: u8, from: usize, to: usize) -> Vec<bool> {

}
*/

#[inline]
fn set_bit(byte: u8, bit_index: usize, value: bool) -> u8 {
    if value {
        (1 << bit_index) | byte
    } else {
        !(1 << bit_index) | byte
    }
}

#[inline]
fn toggle_bit(byte: u8, bit_index: usize) -> u8 {
    (1 << bit_index) ^ byte
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bit_test() {
        let b1 = 128;
        assert!(get_bit(b1, 7));

        let b2 = 136;
        assert!(get_bit(b2, 3));
    }

    #[test]
    fn set_bit_test() {
        let b1 = 128;
        assert_eq!(136, set_bit(b1, 3, true));

        let b2 = 8;
        assert_eq!(136, set_bit(b2, 7, true));
    }

    #[test]
    fn toggle_bit_test() {
        let b1 = 128;
        assert_eq!(136, toggle_bit(b1, 3));
    }

    #[test]
    fn grid_test() {
        let mut grid = Grid::new();

        let b4 = grid.get_item(10);
        assert_eq!(b4, Some(false));

        grid.set_item(10, true);
        assert_eq!(Some(true), grid.get_item(10));
    }
}
