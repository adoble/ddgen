use crate::bits::Bits;

// pub trait RequestWord {

// fn new(bits: u8) -> Self;
// fn bits(&self) -> u8;

// // TODO this should be private?
// fn set_bits(&mut self, bits: u8) -> &mut Self;

// // Default implementations
// fn modify_bit(&mut self, position: u8, state: bool) -> &mut Self {
//     let mut mask: u8 = 1 << position;

//     let modified_bits = if state {
//         // setting the bit
//         self.bits() | mask
//     } else {
//         // clear the bit{
//         mask = !mask;
//         self.bits() & mask
//     };

//     self.set_bits(modified_bits);

//     self
// }

// /// Modify the field as specified by the start and end bit positions.
// ///
// /// Warning: Attempting to modify the whole word or having end less then start
// /// will cause the function to panic!
// fn modify_field(&mut self, value: u8, start: u8, end: u8) -> &mut Self {
//     let mask = ((1 << (end - start + 1)) - 1) << start;
//     let cleared_bits = self.bits() & !mask;
//     let new_bits = value << start;
//     self.set_bits(cleared_bits | new_bits);

//     self
// }
// }

// impl RequestWord for u8 {
//     fn new(bits: u8) -> Self {
//         bits
//     }

//     fn bits(&self) -> u8 {
//         *self
//     }

//     fn set_bits(&mut self, bits: u8) -> &mut Self {
//         *self = bits;
//         self
//     }
//     // fn word(&self) -> &u8 {
//     //     self
//     // }
// }

pub trait RequestBit {
    fn serialize_bit(&mut self, source: bool, position: usize);
}

impl RequestBit for u8 {
    fn serialize_bit(&mut self, source: bool, position: usize) {
        self.modify_bit(position, source);
    }
}

pub trait RequestField {
    fn serialize_field(&mut self, source: u8, start: usize, end: usize);
}

impl RequestField for u8 {
    fn serialize_field(&mut self, source: u8, start: usize, end: usize) {
        self.modify_field(source, start, end);
    }
}

pub trait RequestWord<T> {
    //data[1..2].serialize_word(self.a_u16);
    fn serialize_word(&mut self, source: T);
}

impl RequestWord<u8> for u8 {
    fn serialize_word(&mut self, source: u8) {
        *self = source;
    }
}

impl RequestWord<i8> for u8 {
    fn serialize_word(&mut self, source: i8) {
        *self = source as u8;
    }
}

impl RequestWord<u16> for [u8] {
    fn serialize_word(&mut self, source: u16) {
        self[0] = source.to_le_bytes()[0];
        self[1] = source.to_le_bytes()[1];
    }
}

impl RequestWord<i16> for [u8] {
    fn serialize_word(&mut self, source: i16) {
        self[0] = source.to_le_bytes()[0];
        self[1] = source.to_le_bytes()[1];
    }
}

pub trait RequestArray<T> {
    // Usage : data[5..=10].serialize_repeating_words(self.a_repeating_u16, self.a_count.into());

    fn serialize_repeating_words(&mut self, source: T, number: usize);

    //fn serialize_repeating_words<const N: usize>(&self, number: usize) -> [u8; N];
}

impl<const SOURCE_LEN: usize> RequestArray<[u16; SOURCE_LEN]> for [u8] {
    fn serialize_repeating_words(&mut self, source: [u16; SOURCE_LEN], number: usize) {
        let mut target_position = 0;
        for i in 0..number {
            self[target_position + i] = source[i].to_le_bytes()[0];
            self[target_position + i + 1] = source[i].to_le_bytes()[1];
            target_position += 1;
        }
    }
    //     impl<const SOURCE_LEN: usize> RequestArray for [u16; SOURCE_LEN] {
    // fn serialize_repeating_words<const N: usize>(&self, number: usize) -> [u8; N] {
    //     let mut data = [0u8; N];
    //     let mut target_position = 0;
    //     for i in 0..number {
    //         data[target_position + i] = self[i].to_le_bytes()[0];
    //         data[target_position + i + 1] = self[i].to_le_bytes()[1];
    //         target_position += 1;
    //     }
    //     data
    // }
}

impl<const SOURCE_LEN: usize> RequestArray<[u8; SOURCE_LEN]> for [u8] {
    fn serialize_repeating_words(&mut self, source: [u8; SOURCE_LEN], number: usize) {
        self.copy_from_slice(&source[0..number]);
    }
    //impl<const SOURCE_LEN: usize> RequestArray for [u8; SOURCE_LEN] {
    // fn serialize_repeating_words<const N: usize>(&self, number: usize) -> [u8; N] {
    //     let mut data = [0u8; N];

    //     data.copy_from_slice(&self[0..number]);
    //     data
    // }
}

#[cfg(test)]
mod tests {
    use crate::request::RequestArray;

    use super::{RequestBit, RequestWord};

    #[test]
    fn test_serialize_bool() {
        let mut data = [0u8; 4];
        let b = true;
        data[2].serialize_bit(b, 5);

        assert_eq!(data, [0, 0, 0b0010_0000, 0]);
    }

    #[test]
    fn test_serialize_word_u8() {
        let mut data = [0u8; 4];
        let w: u8 = 42;
        data[2].serialize_word(w);

        assert_eq!(data, [0, 0, 42, 0]);
    }

    #[test]
    fn test_serialize_word_i8() {
        let mut data = [0u8; 4];
        let w: i8 = -42;
        data[2].serialize_word(w);

        let expected_u8 = w as u8;

        assert_eq!(data, [0, 0, expected_u8, 0]);
    }

    #[test]
    fn test_serialize_word_u16() {
        let mut data = [0u8; 4];
        let w: u16 = 22222;
        data[2..].serialize_word(w);

        assert_eq!(
            data,
            [0, 0, 22222u16.to_le_bytes()[0], 22222u16.to_le_bytes()[1]]
        );
    }

    #[test]
    fn test_u16_array() {
        let source = [22222u16, 33333];
        let mut serial_data = [0u8; 7];

        serial_data[2..=5].serialize_repeating_words(source, 2);

        let expected_data: [u8; 7] = [0, 0, 0xCE, 0x56, 0x35, 0x82, 0];

        assert_eq!(serial_data, expected_data);
    }

    #[test]
    fn test_u8_array() {
        let source = [123u8, 33];

        let mut serial_data = [0u8; 4];
        serial_data[0..=1].serialize_repeating_words(source, 2);

        let expected_data: [u8; 4] = [123, 33, 0, 0];

        assert_eq!(serial_data, expected_data);
    }
}
