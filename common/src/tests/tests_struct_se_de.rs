#![cfg(test)]

use crate::deserialize::Deserialize;
use crate::error::DeviceError;
use crate::request::{RequestArray, RequestBit, RequestField, RequestWord};
use crate::response::{ResponseArray, ResponseBit, ResponseField, ResponseWord};
use crate::serialize::Serialize;

// An enum for testing
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TestField {
    Disabled = 0,
    Enabled = 1,
    Tristate = 2,
}

impl TryFrom<u8> for TestField {
    type Error = DeviceError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Disabled),
            1 => Ok(Self::Enabled),
            2 => Ok(Self::Tristate),
            _ => Err(DeviceError::EnumConversion),
        }
    }
}

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
    a_field: TestField,
    a_u8: u8,
    a_u16: u16,
    a_count: u8,
    a_repeating_u16: [u16; 6],
}

impl Serialize<16> for TestStruct {
    fn serialize(&self) -> (u8, [u8; 16]) {
        // The size is calculated from the bit specs.
        let mut data = [0u8; 16];

        //  TODO what happens if the end:word of first elemens is incorrecty set?

        data[0].serialize_bit(self.a_bit, 4);
        data[0].serialize_field(self.a_field as u8, 5, 6);
        data[1..=2].serialize_word(self.a_u16);
        data[3].serialize_word(self.a_u8);
        data[4].serialize_word(self.a_count);
        data[5..=10].serialize_repeating_words(self.a_repeating_u16, self.a_count.into());

        ((self.a_count * 2) + 5, data)
    }
}

// A struct for testing request/serialization
// Bit structure is:
// a_bit =  {bits = "4"}
// a_field =  {bits = "[5..6]"}
// a_u16 =  {bits = "1[]..2[]"}
// a_u8 =  {bits = "3[]"}
// a_count =  {bits = "4[]"}
// a_repeating_u16 =  {bits = "5[]..6[];(4[])<=6" }

impl Deserialize<TestStruct> for [u8] {
    fn deserialize(&self) -> Result<TestStruct, DeviceError> {
        let test_struct = TestStruct {
            a_bit: self[0].deserialize_bit(4),
            a_field: self[0].deserialize_field(5, 6).try_into()?,
            a_u16: self[1..=2].deserialize_word(),
            a_u8: self[3].deserialize_word(),
            a_count: self[4].deserialize_word(),
            a_repeating_u16: self[5..(5 + self[4].deserialize_word() * 2) as usize]
                .deserialize_repeating_words(self[4].deserialize_word() as usize),
        };
        Ok(test_struct)
    }
}

#[test]
fn deserialize_struct() {
    #[derive(PartialEq, Debug, Copy, Clone)]
    struct TestStruct {
        a_bit: bool,
        a_field: TestField,
        a_u8: u8,
        a_u16: u16,
    }

    let expected_test_struct = TestStruct {
        a_bit: true,
        a_field: TestField::Enabled,
        a_u8: 100,
        a_u16: 22222,
    };

    // Bit structure is:
    // a_bit: {bits = "4"}
    // a_field: {bits = "[5..6]"}
    // a_u16: {bits = "1[]..2[]"}
    // a_u8: {bits = "3[]"}
    // a_u16_repeating: {bits = 4[]..5[];4}

    #[rustfmt::skip]
    let data: [u8; 12] = [
        0b0001_0000 | 0b0010_0000, 
        
        0x64, 
        
        0xCE, 0x56,
        
        //[11111, 22222, 33333, 44444],
        0x67, 0x2B, 
        0xCE, 0x56,
        0x35, 0x82,
        0x9C, 0xAD,

        ]; // 12 is the calculated size

    // Deserialize

    let deserialized_test_struct = TestStruct {
        a_bit: data[0].deserialize_bit(4),
        a_field: data[0].deserialize_field(5, 6).try_into().unwrap(), // TODO Altough, is this what we really want?
        a_u8: data[1].deserialize_word(),
        a_u16: data[2..=3].deserialize_word(),
    };

    assert_eq!(deserialized_test_struct, expected_test_struct);
}

#[test]

fn test_deserialize_repeating_word() {
    #[derive(PartialEq, Debug, Copy, Clone)]
    struct TestStruct {
        a_repeating_u8: [u8; 2],
        a_repeating_u16: [u16; 4],
    }

    let expected_struct = TestStruct {
        a_repeating_u8: [206, 86],
        a_repeating_u16: [11111, 22222, 33333, 44444],
    };

    // Bit structure is:
    // a_repeating_u8 = {bits = "0[];2"}
    // a_repeating_u16: {bits = "2[]..3[];4"}
    #[rustfmt::skip]
    let data: [u8; 10] = [
        // [206, 86],
        0xCE, 
        0x56,
        
        //[11111, 22222, 33333, 44444],
        0x67, 0x2B, 
        0xCE, 0x56,
        0x35, 0x82,
        0x9C, 0xAD,

        ]; // 12 is the calculated size

    let deserialized_test_struct = TestStruct {
        a_repeating_u8: data[0..=1].deserialize_repeating_words(2),
        a_repeating_u16: data[2..=9].deserialize_repeating_words(4),
    };

    assert_eq!(deserialized_test_struct, expected_struct);
}

#[test]
fn test_serialize_struct() {
    let test_request = TestStruct {
        a_bit: true,
        a_field: TestField::Enabled,
        a_u8: 100,
        a_u16: 22222,
        a_count: 3,
        a_repeating_u16: [44444, 33333, 22222, 0, 0, 0],
    };

    let expected_data: [u8; 16] = [
        0b0001_0000 | 0b0010_0000,
        0xCE,
        0x56,
        0x64,
        0x03,
        0x9C,
        0xAD,
        0x35,
        0x82,
        0xCE,
        0x56,
        0,
        0,
        0,
        0,
        0,
    ];

    let (count, data) = test_request.serialize();

    assert_eq!(count, 11);
    assert_eq!(data, expected_data);
}

#[test]
fn test_deserialize_struct() {
    // Bit structure is:
    // a_bit: {bits = "4"}
    // a_field: {bits = "[5..6]"}
    // a_u16: {bits = "1[]..2[]"}
    // a_u8: {bits = "3[]"}
    // a_count: {bits= "4{]"}
    // a_u16_repeating: {bits = 5[]..6[];(4[])<=6)}
    let data: [u8; 13] = [
        // a_bit | a_field
        0b0001_0000 | 0b0010_0000,
        // a_u16
        0xCE,
        0x56,
        // a_u8
        0x64,
        //a_count
        0x4,
        // a_u16_repeating [11111, 22222, 33333, 44444],
        0x67,
        0x2B,
        0xCE,
        0x56,
        0x35,
        0x82,
        0x9C,
        0xAD,
    ];

    let expected = TestStruct {
        a_bit: true,
        a_field: TestField::Enabled,
        a_u8: 100,
        a_u16: 22222,
        a_count: 4,
        a_repeating_u16: [11111, 22222, 33333, 44444, 0, 0],
    };

    let test_struct: TestStruct = data.deserialize().unwrap();

    assert_eq!(test_struct, expected);
}
