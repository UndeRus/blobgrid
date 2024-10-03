
#[inline]
pub fn get_bit(byte: u8, bit_index: usize) -> bool {
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
pub fn set_bit(byte: u8, bit_index: usize, value: bool) -> u8 {
    if value {
        (1 << bit_index) | byte
    } else {
        !(1 << bit_index) | byte
    }
}

#[inline]
pub fn toggle_bit(byte: u8, bit_index: usize) -> u8 {
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

}