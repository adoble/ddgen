pub trait RequestWord {
    fn new(bits: u8) -> Self;
    fn bits(&self) -> u8;

    // TODO this should be private?
    fn set_bits(&mut self, bits: u8) -> &mut Self;

    // Default implementations
    fn modify_bit(&mut self, position: u8, state: bool) -> &mut Self {
        let mut mask: u8 = 1 << position;

        let modified_bits = if state {
            // setting the bit
            self.bits() | mask
        } else {
            // clear the bit{
            mask = !mask;
            self.bits() & mask
        };

        self.set_bits(modified_bits);

        self
    }

    /// Modify the field as specified by the start and end bit positions.
    ///
    /// Warning: Attempting to modify the whole word or having end less then start
    /// will cause the function to panic!
    fn modify_field(&mut self, value: u8, start: u8, end: u8) -> &mut Self {
        let mask = ((1 << (end - start + 1)) - 1) << start;
        let cleared_bits = self.bits() & !mask;
        let new_bits = value << start;
        self.set_bits(cleared_bits | new_bits);

        self
    }
}

impl RequestWord for u8 {
    fn new(bits: u8) -> Self {
        bits
    }

    fn bits(&self) -> u8 {
        *self
    }

    fn set_bits(&mut self, bits: u8) -> &mut Self {
        *self = bits;
        self
    }
    // fn word(&self) -> &u8 {
    //     self
    // }
}

pub trait RequestArray {
    fn serialize_repeating_words<const N: usize>(&self, number: usize) -> [u8; N];
}

impl<const SOURCE_LEN: usize> RequestArray for [u16; SOURCE_LEN] {
    fn serialize_repeating_words<const N: usize>(&self, number: usize) -> [u8; N] {
        let mut data = [0u8; N];
        let mut target_position = 0;
        for i in 0..number {
            data[target_position + i] = self[i].to_le_bytes()[0];
            data[target_position + i + 1] = self[i].to_le_bytes()[1];
            target_position += 1;
        }
        data
    }
}

impl<const SOURCE_LEN: usize> RequestArray for [u8; SOURCE_LEN] {
    fn serialize_repeating_words<const N: usize>(&self, number: usize) -> [u8; N] {
        let mut data = [0u8; N];

        data.copy_from_slice(&self[0..number]);
        data
    }
}

#[cfg(test)]
mod tests {
    use crate::request::RequestArray;

    #[test]
    fn test_u16_array() {
        let source = [22222u16, 33333];

        // let mut serial_data = [0u8; 4];
        let serial_data: [u8; 4] = source.serialize_repeating_words(2);

        let expected_data: [u8; 4] = [0xCE, 0x56, 0x35, 0x82];

        assert_eq!(serial_data, expected_data);
    }

    #[test]
    fn test_u8_array() {
        let source = [123u8, 33];

        // let mut serial_data = [0u8; 4];
        let serial_data: [u8; 2] = source.serialize_repeating_words(2);

        let expected_data: [u8; 2] = [123, 33];

        assert_eq!(serial_data, expected_data);
    }
}
