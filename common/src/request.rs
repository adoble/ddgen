use crate::{bits::Bits, serialize::Serialize};

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

impl RequestWord<u32> for [u8] {
    fn serialize_word(&mut self, source: u32) {
        self[0] = source.to_le_bytes()[0];
        self[1] = source.to_le_bytes()[1];
        self[2] = source.to_le_bytes()[2];
        self[3] = source.to_le_bytes()[3];
    }
}

impl RequestWord<i32> for [u8] {
    fn serialize_word(&mut self, source: i32) {
        self[0] = source.to_le_bytes()[0];
        self[1] = source.to_le_bytes()[1];
        self[2] = source.to_le_bytes()[2];
        self[3] = source.to_le_bytes()[3];
    }
}

pub trait RequestArray<T> {
    // Usage : data[5..=10].serialize_repeating_words(self.a_repeating_u16, self.a_count.into());
    fn serialize_repeating_words(&mut self, source: T, number: usize);
}

impl<const SOURCE_LEN: usize> RequestArray<[u8; SOURCE_LEN]> for [u8] {
    fn serialize_repeating_words(&mut self, source: [u8; SOURCE_LEN], number: usize) {
        self.copy_from_slice(&source[0..number]);
    }
}

impl<const SOURCE_LEN: usize> RequestArray<[u16; SOURCE_LEN]> for [u8] {
    fn serialize_repeating_words(&mut self, source: [u16; SOURCE_LEN], number: usize) {
        let mut target_position = 0;
        #[allow(clippy::explicit_counter_loop)]
        for i in 0..number {
            self[target_position + i] = source[i].to_le_bytes()[0];
            self[target_position + i + 1] = source[i].to_le_bytes()[1];
            target_position += 1;
        }
    }
}

impl<const SOURCE_LEN: usize> RequestArray<[u32; SOURCE_LEN]> for [u8] {
    fn serialize_repeating_words(&mut self, source: [u32; SOURCE_LEN], number: usize) {
        for (i, source_value) in source.iter().enumerate().take(number) {
            let target_position = i * 4;
            self[target_position..(target_position + 4)]
                .copy_from_slice(&source_value.to_le_bytes());
        }
    }
}

pub trait RequestStruct<T: Serialize> {
    // Usage : data[0..].serialize_struct(self.a_struct, 0);
    fn serialize_struct<const TARGET_LEN: usize>(&mut self, source: T);
}

impl<T: Serialize> RequestStruct<T> for [u8] {
    fn serialize_struct<const TARGET_LEN: usize>(&mut self, source: T) {
        // let (size, data): (usize, [u8; TARGET_LEN]) = source.serialize();
        let (size, data, _): (usize, [u8; TARGET_LEN], _) = source.serialize();
        self.copy_from_slice(&data[0..size]);
    }
}

#[cfg(test)]
mod tests {
    // use serde::Serialize;

    use crate::serialize::Serialize;

    use super::{RequestArray, RequestBit, RequestField, RequestStruct, RequestWord};

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
    fn test_serialize_word_i16() {
        let mut data = [0u8; 4];
        let w: i16 = -222;
        data[2..].serialize_word(w);

        let mut expected = [0u8; 2];

        expected[0] = (-222 as i16).to_le_bytes()[0];
        expected[1] = (-222 as i16).to_le_bytes()[1];

        assert_eq!(data, [0, 0, expected[0], expected[1]]);
    }

    #[test]
    fn test_serialize_word_u32() {
        let mut data = [0u8; 6];
        let w: u32 = 2_123_967_295;
        data[2..].serialize_word(w);

        assert_eq!(data, [0, 0, 0x3F, 0x2B, 0x99, 0x7E]);
    }

    #[test]
    fn test_serialize_word_i32() {
        let mut data = [0u8; 6];
        let w: i32 = -2_123_967_295;
        data[2..].serialize_word(w);

        assert_eq!(data, [0, 0, 0xC1, 0xD4, 0x66, 0x81]);
    }

    #[test]
    fn test_u8_array() {
        let source = [123u8, 33];

        let mut serial_data = [0u8; 4];
        serial_data[0..=1].serialize_repeating_words(source, 2);

        let expected_data: [u8; 4] = [123, 33, 0, 0];

        assert_eq!(serial_data, expected_data);
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
    fn test_u32_array() {
        let source = [0x12345678u32, 0xFEDCBA98];
        let mut serial_data = [0u8; 10];

        serial_data[2..=9].serialize_repeating_words(source, 2);

        let expected_data: [u8; 10] = [0, 0, 0x78, 0x56, 0x34, 0x12, 0x98, 0xBA, 0xDC, 0xFE];

        assert_eq!(serial_data, expected_data);
    }

    struct TestCommonStruct {
        c_bool: bool,
        c_test_field: CommonTestField,
        c_u16: u16,
    }

    impl Serialize for TestCommonStruct {
        fn serialize<const N: usize>(&self) -> (usize, [u8; N], impl Iterator<Item = u8>) {
            let mut data = [0u8; N];

            data[0].serialize_bit(self.c_bool, 1);
            data[0].serialize_field(self.c_test_field as u8, 3, 5);
            data[1..=2].serialize_word(self.c_u16);

            (3, data, std::iter::empty::<u8>())
        }
    }
    #[allow(dead_code)]
    #[derive(PartialEq, Debug, Copy, Clone)]
    enum CommonTestField {
        TestZero = 0,
        TestOne = 1,
        TestTwo = 2,
        TestThree = 3,
    }

    #[test]
    fn test_common_struct_serialization() {
        let test_common_struct = TestCommonStruct {
            c_bool: true,
            c_test_field: CommonTestField::TestOne,
            c_u16: 45678,
        };

        let mut data: [u8; 3] = [0; 3];
        // data[0..].serialize_struct::<3>(test_common_struct);
        data[0..].serialize_struct::<3>(test_common_struct);

        assert_eq!([0b0000_1010, 0x6E, 0xB2], data);
    }
}
