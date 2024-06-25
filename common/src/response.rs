// TODO make the word size generic

use crate::bits::Bits;

pub trait ResponseBit {
    // a_bit: data[0].deserialize_bit(4),
    fn deserialize_bit(&self, position: usize) -> bool;
}

impl ResponseBit for u8 {
    /// Get a bit as bool at a particular position
    fn deserialize_bit(&self, position: usize) -> bool {
        let mask: u8 = 1 << position;
        (*self & mask) > 0
    }
}

pub trait ResponseField {
    fn deserialize_field(&self, start: usize, end: usize) -> u8;
}

impl ResponseField for u8 {
    fn deserialize_field(&self, start: usize, end: usize) -> u8 {
        self.field(start, end)
    }
}

pub trait ResponseWord<T> {
    fn deserialize_word(&self) -> T;
}

impl ResponseWord<u8> for u8 {
    fn deserialize_word(&self) -> u8 {
        *self
    }
}

impl ResponseWord<i8> for u8 {
    fn deserialize_word(&self) -> i8 {
        *self as i8
    }
}

impl ResponseWord<u16> for [u8] {
    fn deserialize_word(&self) -> u16 {
        u16::from_le_bytes([self[0], self[1]])
    }
}

impl ResponseWord<i16> for [u8] {
    fn deserialize_word(&self) -> i16 {
        i16::from_le_bytes([self[0], self[1]])
    }
}

impl ResponseWord<u32> for [u8] {
    fn deserialize_word(&self) -> u32 {
        u32::from_le_bytes([self[0], self[1], self[2], self[4]])
    }
}

impl ResponseWord<i32> for [u8] {
    fn deserialize_word(&self) -> i32 {
        i32::from_le_bytes([self[0], self[1], self[2], self[4]])
    }
}

pub trait ResponseArray<T> {
    //  a_repeating_u8: data[0..=1].deserialize_repeating_word(2),
    // self is the u8 stream
    fn deserialize_repeating_words(&self, number: usize) -> T;
}

impl<const TARGET_LEN: usize> ResponseArray<[u8; TARGET_LEN]> for [u8] {
    fn deserialize_repeating_words(&self, number: usize) -> [u8; TARGET_LEN] {
        let mut target = [0; TARGET_LEN];
        self.iter()
            .take(number)
            .enumerate()
            .for_each(|(i, b)| target[i] = *b);
        target
    }
}

impl<const TARGET_LEN: usize> ResponseArray<[u16; TARGET_LEN]> for [u8] {
    fn deserialize_repeating_words(&self, number: usize) -> [u16; TARGET_LEN] {
        let mut target = [0; TARGET_LEN];
        self.chunks(2)
            .take(number)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .enumerate()
            .for_each(|(i, v)| target[i] = v);
        target
    }
}

impl<const TARGET_LEN: usize> ResponseArray<[u32; TARGET_LEN]> for [u8] {
    fn deserialize_repeating_words(&self, number: usize) -> [u32; TARGET_LEN] {
        let mut target = [0; TARGET_LEN];
        self.chunks(4)
            .take(number)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .enumerate()
            .for_each(|(i, v)| target[i] = v);

        target
    }
}

impl<const TARGET_LEN: usize> ResponseArray<[i8; TARGET_LEN]> for [u8] {
    fn deserialize_repeating_words(&self, number: usize) -> [i8; TARGET_LEN] {
        let mut target = [0; TARGET_LEN];

        self.iter()
            .take(number)
            .enumerate()
            .for_each(|(i, b)| target[i] = (*b) as i8);
        target
    }
}

impl<const TARGET_LEN: usize> ResponseArray<[i16; TARGET_LEN]> for [u8] {
    fn deserialize_repeating_words(&self, number: usize) -> [i16; TARGET_LEN] {
        let mut target = [0; TARGET_LEN];

        self.chunks(2)
            .take(number)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .enumerate()
            .for_each(|(i, v)| target[i] = v);

        target
    }
}

impl<const TARGET_LEN: usize> ResponseArray<[i32; TARGET_LEN]> for [u8] {
    fn deserialize_repeating_words(&self, number: usize) -> [i32; TARGET_LEN] {
        let mut target = [0; TARGET_LEN];

        self.chunks(4)
            .take(number)
            .map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .enumerate()
            .for_each(|(i, v)| target[i] = v);
        target
    }
}

#[cfg(test)]
mod tests {

    use crate::deserialize::Deserialize;
    use crate::error::DeviceError;

    use super::*;
    #[test]
    fn response_bits() {
        let source: u8 = 0b0011_0011;

        let r = source.deserialize_bit(1);
        assert_eq!(r, true);
        let r = source.deserialize_bit(6);
        assert_eq!(r, false);
        let r = source.deserialize_bit(4);
        assert_eq!(r, true);
    }

    #[test]
    fn test_deserialize_word() {
        // 5u8, 31313u16
        let data: [u8; 3] = [0x05, 0x51, 0x7A];

        let a_u8: u8 = data[0].deserialize_word();
        assert_eq!(a_u8, 5);
        let a_u16: u16 = data[1..=2].deserialize_word();
        assert_eq!(a_u16, 31313);
    }

    #[test]
    fn test_unsigned_arrays() {
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

        a.data_u8 = source[0..2].deserialize_repeating_words(2);
        assert_eq!(a.data_u8, expected_data_u8);
        // Location is slice range and number of repeats is derived from start_word_index + (bit_slice->repeats) / (word range)
        a.data_u16 = source[2..6].deserialize_repeating_words(2);
        assert_eq!(a.data_u16, expected_data_u16);

        a.data_u32 = source[6..].deserialize_repeating_words(2);
        assert_eq!(a.data_u32, expected_data_u32);
    }

    #[test]
    fn test_signed_arrays() {
        let expected_data_i8: [i8; 2] = [12, -13];
        let expected_data_i16: [i16; 7] = [22222, -31313, 0, 0, 0, 0, 0];
        let expected_data_i32: [i32; 2] = [200_000_000, -333_333_333];

        #[rustfmt::skip]
        let mut source: [u8; 14] = [12,  0xF3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,0 ];

        source[2] = 22222i16.to_le_bytes()[0];
        source[3] = 22222i16.to_le_bytes()[1];
        source[4] = (-31313i16).to_le_bytes()[0];
        source[5] = (-31313i16).to_le_bytes()[1];

        source[6] = 200_000_000i32.to_le_bytes()[0];
        source[7] = 200_000_000i32.to_le_bytes()[1];
        source[8] = 200_000_000i32.to_le_bytes()[2];
        source[9] = 200_000_000i32.to_le_bytes()[3];
        source[10] = (-333_333_333i32).to_le_bytes()[0];
        source[11] = (-333_333_333i32).to_le_bytes()[1];
        source[12] = (-333_333_333i32).to_le_bytes()[2];
        source[13] = (-333_333_333i32).to_le_bytes()[3];

        struct A {
            data_i8: [i8; 2],
            data_i16: [i16; 7], // 7 is the max number of repeats
            data_i32: [i32; 2],
        }
        let mut a = A {
            data_i8: [0; 2],
            data_i16: [0; 7], // 7 is the max number of repeats
            data_i32: [0; 2],
        };

        a.data_i8 = source[0..2].deserialize_repeating_words(2);
        assert_eq!(a.data_i8, expected_data_i8);
        // Location is slice range and number of repeats is derived from start_word_index + (bit_slice->repeats) / (word range)
        a.data_i16 = source[2..6].deserialize_repeating_words(2);
        assert_eq!(a.data_i16, expected_data_i16);

        a.data_i32 = source[6..].deserialize_repeating_words(2);
        assert_eq!(a.data_i32, expected_data_i32);
    }

    #[derive(Debug, PartialEq)]
    struct TestCommonStruct {
        c_bool: bool,
        c_test_field: CommonTestField,
        c_u16: u16,
    }

    impl Deserialize<Self> for TestCommonStruct {
        fn deserialize(buf: &[u8]) -> Result<TestCommonStruct, DeviceError> {
            Ok(Self {
                c_bool: buf[0].deserialize_bit(1),
                c_test_field: buf[0].deserialize_field(3, 5).try_into()?,
                c_u16: buf[1..=2].deserialize_word(),
            })
        }
    }

    // impl Deserialize for [u8] {
    //     fn deserialize(&self) -> Result<TestCommonStruct, DeviceError> {
    //         Ok(TestCommonStruct {
    //             c_bool: self[0].deserialize_bit(1),
    //             c_test_field: self[0].deserialize_field(3, 5).try_into()?,
    //             c_u16: self[1..=2].deserialize_word(),
    //         })
    //     }
    // }

    #[allow(dead_code)]
    #[derive(PartialEq, Debug, Copy, Clone)]
    enum CommonTestField {
        TestZero = 0,
        TestOne = 1,
        TestTwo = 2,
        TestThree = 3,
    }

    impl TryFrom<u8> for CommonTestField {
        type Error = DeviceError;
        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(Self::TestZero),
                1 => Ok(Self::TestOne),
                2 => Ok(Self::TestTwo),
                3 => Ok(Self::TestThree),
                _ => Err(DeviceError::EnumConversion),
            }
        }
    }

    #[test]
    fn test_common_struct_deserialization() {
        let source_data = [0b0000_1010 as u8, 0x6E, 0xB2];

        let expected_struct = TestCommonStruct {
            c_bool: true,
            c_test_field: CommonTestField::TestOne,
            c_u16: 45678,
        };

        let actual_struct = TestCommonStruct::deserialize(&source_data[0..3]).unwrap();

        assert_eq!(actual_struct, expected_struct);
    }
}
