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

impl<const TARGET_LEN: usize> ResponseArray for [u16; TARGET_LEN] {
    fn deserialize_repeating_words(&mut self, source: &[u8]) {
        //let mut buf: [u16; TARGET_LEN] = [0; TARGET_LEN];
        let mut i = 0;

        for b in source.chunks(2) {
            self[i] = u16::from_le_bytes([b[0], b[1]]);
            i += 1;
        }

        //buf
    }
}

// pub fn deserialize_repeating_words_u16<const LEN: usize>(
//     source: &[u8],
//     start: usize,
//     repeats: usize,
// ) -> [u16; LEN] {
//     let mut buf: [u16; LEN] = [0; LEN];
//     let mut i = 0;

//     for b in source[start..].chunks(2).take(repeats) {
//         buf[i] = u16::from_le_bytes([b[0], b[1]]);
//         i += 1;
//     }

//     buf
// }

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
        let source: [u8; 10] = [0, 0, 0xCE, 0x56, 0x35, 0x82, 0, 0, 0, 0]; // Contains other data

        let expected_data: [u16; 7] = [22222, 33333, 0, 0, 0, 0, 0];

        struct A {
            data: [u16; 7], // 7 is the max number of repeats
        }
        let mut a = A {
            data: [0; 7], // 7 is the max number of repeats
        };
        // Location is slice range and number of repeats is derived from start_word_index + (bit_slice->repeats) / (word range)
        a.data.deserialize_repeating_words(&source[2..6]);
        assert_eq!(a.data, expected_data);
    }
}
