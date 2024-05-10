#![cfg(test)]

use core::iter::Iterator;

use crate::request::{RequestBit, RequestWord};
use crate::serialize::Serialize;

// A struct for testing request/serialization
// Bit structure is:
// a_bit =  {bits = "4"}
// a_field =  {bits = "[5..6]"}
// a_u16 =  {bits = "1[]..2[]"}
// a_u8 =  {bits = "3[]"}
// a_count =  {bits = "4[]"}
// a_repeating_u16 =  {bits = "5[]..6[];(4[])<=6" }
//
#[derive(PartialEq, Debug, Copy, Clone)]
struct TestStruct {
    a_bit: bool,
    a_u8: u8,
    a_reader: Reader,
}

impl Serialize for TestStruct {
    fn serialize<const N: usize>(&self) -> (usize, [u8; N], impl Iterator<Item = u8>) {
        let mut data = [0u8; N];
        #[allow(unused_variables)]
        let provider = std::iter::empty::<u8>();

        data[0].serialize_bit(self.a_bit, 7);
        //data[1..=1].serialize_word(self.a_u8);  //TODO this does not work for u8
        data[1] = self.a_u8;
        let provider = self.a_reader;

        (2, data, provider)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct Reader {
    number: u8,
    limit: u8,
}
impl Reader {
    pub fn new(limit: u8) -> Reader {
        Reader { number: 0, limit }
    }
}

impl Iterator for Reader {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.number < self.limit {
            self.number += 1;
            Some(self.number)
        } else {
            None
        }
    }
}

#[test]
fn test_reader() {
    let reader = Reader::new(10);

    let mut results: [u8; 10] = [0; 10];
    reader
        .into_iter()
        .enumerate()
        .for_each(|v| results[v.0] = v.1);

    assert_eq!(results, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[test]
fn test_serialize() {
    let test_struct = TestStruct {
        a_bit: true,
        a_u8: 42,
        a_reader: Reader::new(5),
    };
    let (size, data, reader) = test_struct.serialize();

    assert_eq!(size, 2);
    assert_eq!(data, [0b1000_0000, 42]);

    let mut serial_data_unspecified_len: [u8; 5] = [0; 5];
    reader
        .into_iter()
        .enumerate()
        .for_each(|v| serial_data_unspecified_len[v.0] = v.1);

    assert_eq!(serial_data_unspecified_len, [1, 2, 3, 4, 5]);
}

// A more complicated reader. Done to simluate reading from an external source

#[derive(PartialEq, Debug, Copy, Clone)]
struct ExtTestStruct {
    a_bit: bool,
    a_u16: u16,
    a_reader: ExtReader,
}

impl Serialize for ExtTestStruct {
    fn serialize<const N: usize>(&self) -> (usize, [u8; N], impl Iterator<Item = u8>) {
        let mut data = [0u8; N];

        data[0].serialize_bit(self.a_bit, 7);
        data[1..=2].serialize_word(self.a_u16);
        let provider = self.a_reader;

        (3, data, provider)
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct ExtReader {
    word_pos: usize,
    byte_pos: usize,
    source: &'static [u32],
}

impl ExtReader {
    pub fn new(source: &'static [u32]) -> ExtReader {
        ExtReader {
            word_pos: 0,
            byte_pos: 0,
            source,
        }
    }
}

impl Iterator for ExtReader {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.word_pos < self.source.len() {
            let word = self.source[self.word_pos];
            let bytes = word.to_be_bytes();
            let byte = bytes[self.byte_pos];

            if self.byte_pos == 3 {
                self.word_pos += 1;
                self.byte_pos = 0;
            } else {
                self.byte_pos += 1;
            };
            Some(byte)
        } else {
            None
        }
    }
}

#[test]
fn test_ext_reader() {
    const SOURCE_DATA: [u32; 5] = [10, 11, 12, 13, 14];
    let reader = ExtReader::new(&SOURCE_DATA);

    let mut data: [u8; 20] = [0; 20];
    for byte in reader.enumerate() {
        data[byte.0] = byte.1
    }

    assert_eq![data[3], 10];
    assert_eq![data[7], 11];
    assert_eq![data[11], 12];
    assert_eq![data[15], 13];
    assert_eq![data[19], 14];
}

#[test]
fn test_serialise_with_complexer_data() {
    const EXTERNAL_DATA: [u32; 5] = [0x12345678, 0xabcdef01, 0x87654321, 0xfedcba98, 0x13579bdf];

    let reader = ExtReader::new(&EXTERNAL_DATA);

    let ext_test_struct = ExtTestStruct {
        a_bit: true,
        a_u16: 4711,
        a_reader: reader,
    };
    let (size, data, reader) = ext_test_struct.serialize();

    assert_eq!(size, 3);
    assert_eq!(data, [0b1000_0000, 0x67, 0x12]);

    let expected = [
        0x12u8, 0x34, 0x56, 0x78, 0xab, 0xcd, 0xef, 0x01, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba,
        0x98, 0x13, 0x57, 0x9b, 0xdf,
    ];

    let mut actual = [0u8; 20];
    for byte in reader.enumerate() {
        actual[byte.0] = byte.1
    }

    assert_eq!(actual, expected);
}
