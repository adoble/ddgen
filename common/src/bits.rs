/// Traits to add bit manuipulation to unsigned integers

pub trait Bits {
    fn bit(&self, position: usize) -> bool;

    fn field(&self, start: usize, end: usize) -> u8;

    fn modify_bit(&mut self, position: usize, state: bool);

    /// Modify the field as specified by the start and end bit positions.
    ///
    /// Warning: Attempting to modify the whole word or having end less then start
    /// will cause the function to panic!
    fn modify_field(&mut self, value: u8, start: usize, end: usize);

    // Get a boolean from a bit position
    //fn deserialize_bit(&mut self, source: u8, position: usize);
}

impl Bits for u8 {
    /// Get a boolean from a bit position
    fn bit(&self, position: usize) -> bool {
        let mask: u8 = 1 << position;
        (self & mask) > 0
    }

    fn field(&self, start: usize, end: usize) -> u8 {
        let mut mask: u8 = 0;

        for count in start..=end {
            let b = 1 << count;
            mask |= b;
        }

        let v = self & mask;
        v >> start
    }

    fn modify_bit(&mut self, position: usize, state: bool) {
        let mut mask: u8 = 1 << position;

        if state {
            // setting the bit
            *self |= mask
        } else {
            // clear the bit{
            mask = !mask;
            *self &= mask
        };
    }

    fn modify_field(&mut self, value: u8, start: usize, end: usize) {
        let mask = ((1 << (end - start + 1)) - 1) << start;
        let cleared_bits = *self & !mask;
        let new_bits = value << start;
        *self = cleared_bits | new_bits;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bit_u8() {
        let source: u8 = { 0b011_0111 };

        assert_eq!(true, source.bit(0));
        assert_eq!(false, source.bit(3));
        assert_eq!(true, source.bit(5));
        assert_eq!(false, source.bit(7));
    }

    #[test]
    fn field_u8() {
        let source: u8 = { 0b0011_0111 };

        assert_eq!(3, source.field(0, 1));
        assert_eq!(3, source.field(4, 5));
        assert_eq!(7, source.field(0, 2));
    }

    #[test]
    fn modified_bit_u8() {
        let mut source: u8 = 0b0011_0111;

        source.modify_bit(3, true);
        assert_eq!(source, 0b0011_1111);
        source.modify_bit(1, false);
        assert_eq!(source, 0b0011_1101);
        source.modify_bit(7, false);
        assert_eq!(source, 0b0011_1101);
    }

    #[test]
    fn modify_field_u8() {
        let mut source: u8 = 0b0011_0111;

        source.modify_field(0b11, 6, 7);
        assert_eq!(source, 0b1111_0111);
        source.modify_field(0b010, 0, 2);
        assert_eq!(source, 0b1111_0010);
    }
}
