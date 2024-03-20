// TODO make the word size generic

pub trait ResponseWord {
    fn word(&self) -> &u8;

    /// Get a bit as bool at a particular position
    fn bit(&self, position: u8) -> bool {
        let mask: u8 = 1 << position;
        (self.word() & mask) > 0
    }

    // Get the field at the specified position
    fn field(&self, start: u8, end: u8) -> u8 {
        let mut mask: u8 = 0;

        for count in start..=end {
            let b = 1 << count;
            mask |= b;
        }
        let v = self.word() & mask;
        v >> start
    }
}

impl ResponseWord for u8 {
    fn word(&self) -> &u8 {
        self
    }
}
pub trait ResponseArray {
    fn deserialize_repeating_words(&mut self, source: &[u8]);
}

impl<const TARGET_LEN: usize> ResponseArray for [u8; TARGET_LEN] {
    fn deserialize_repeating_words(&mut self, source: &[u8]) {
        source.iter().enumerate().for_each(|(i, b)| self[i] = *b);
    }
}

impl<const TARGET_LEN: usize> ResponseArray for [u16; TARGET_LEN] {
    fn deserialize_repeating_words(&mut self, source: &[u8]) {
        source
            .chunks(2)
            .enumerate()
            .for_each(|(i, b)| self[i] = u16::from_le_bytes([b[0], b[1]]));
    }
}

impl<const TARGET_LEN: usize> ResponseArray for [u32; TARGET_LEN] {
    fn deserialize_repeating_words(&mut self, source: &[u8]) {
        source
            .chunks(4)
            .enumerate()
            .for_each(|(i, b)| self[i] = u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_bits() {
        let r: u8 = 0b0011_0011;

        assert_eq!(true, r.bit(1));
        assert_eq!(false, r.bit(6));
        assert_eq!(true, r.bit(4));
    }

    #[test]
    fn response_fields() {
        let r: u8 = 0b0011_0011;

        assert_eq!(r.field(0, 3), 3);
        assert_eq!(r.field(1, 4), 9);
    }

    #[test]
    fn test_u16_array() {
        let expected_data_u8: [u8; 2] = [12, 13];
        let expected_data_u16: [u16; 7] = [22222, 33333, 0, 0, 0, 0, 0];
        let expected_data_u32: [u32; 2] = [4_200_000_000, 3_333_333_333];

        let source: [u8; 14] = [
            12, 13, 0xCE, 0x56, 0x35, 0x82, 0x00, 0xEA, 0x56, 0xFA, 0x55, 0xA1, 0xAE, 0xC6,
        ];

        struct A {
            data_u8: [u8; 2],
            data_u16: [u16; 7], // 7 is the max number of repeats
            data_u32: [u32; 2],
        }
        let mut a = A {
            data_u8: [0; 2],
            data_u16: [0; 7], // 7 is the max number of repeats
            data_u32: [0; 2],
        };

        a.data_u8.deserialize_repeating_words(&source[0..2]);
        assert_eq!(a.data_u8, expected_data_u8);
        // Location is slice range and number of repeats is derived from start_word_index + (bit_slice->repeats) / (word range)
        a.data_u16.deserialize_repeating_words(&source[2..6]);
        assert_eq!(a.data_u16, expected_data_u16);

        a.data_u32.deserialize_repeating_words(&source[6..]);
        assert_eq!(a.data_u32, expected_data_u32);
    }
}
