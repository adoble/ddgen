#![cfg(test)]

use crate::error::DeviceError;
use crate::request::RequestWord;
use crate::response::ResponseWord;
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
struct TestRequest {
    a_bit: bool,
    a_field: TestField,
    a_u8: u8,
    a_u16: u16,
    a_count: u8,
    a_repeating_u16: [u16; 6],
}

impl Serialize<16> for TestRequest {
    fn serialize(&self) -> (u8, [u8; 16]) {
        // The size is calculated from the bit specs.
        let mut data = [0u8; 16];

        data[0].modify_bit(4, self.a_bit);
        data[0].modify_field(self.a_field as u8, 5, 6);
        data[1] = self.a_u16.to_le_bytes()[0];
        data[2] = self.a_u16.to_le_bytes()[1];
        data[3] = self.a_u8;
        data[4] = self.a_count;

        let mut target_index = 5; // a_repeating_u16 =  {bits = "5[]..6[];(4[])<=6" }
        for i in 0..(self.a_count as usize) {
            data[target_index + i] = self.a_repeating_u16[i].to_le_bytes()[0];
            data[target_index + i + 1] = self.a_repeating_u16[i].to_le_bytes()[1];
            target_index += 1;
        }

        ((self.a_count * 2) + 5, data)
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

    let data: [u8; 4] = [0b0001_0000 | 0b0010_0000, 0xCE, 0x56, 0x64]; // 4 is the calculated size

    // deserialize
    let deserialized_test_struct = TestStruct {
        a_bit: data[0].bit(4),
        a_field: data[0].field(5, 6).try_into().unwrap(),
        a_u8: data[3],
        a_u16: u16::from_le_bytes([data[1], data[2]]),
    };

    assert_eq!(deserialized_test_struct, expected_test_struct);
}

#[test]
fn serialize_struct() {
    #[derive(PartialEq, Debug, Copy, Clone)]
    struct TestStruct {
        a_bit: bool,
        a_field: TestField,
        a_u8: u8,
        a_u16: u16,
    }

    let test_struct = TestStruct {
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

    let expected_data: [u8; 4] = [0b0001_0000 | 0b0010_0000, 0xCE, 0x56, 0x64]; // 4 is the calculated size

    // The size is calculated from the bit specs.
    let mut data = [0u8; 4];

    data[0].modify_bit(4, test_struct.a_bit);
    data[0].modify_field(test_struct.a_field as u8, 5, 6);
    data[1] = test_struct.a_u16.to_le_bytes()[0];
    data[2] = test_struct.a_u16.to_le_bytes()[1];
    data[3] = test_struct.a_u8;

    assert_eq!(data, expected_data);
}

#[test]
fn serialize_struct2() {
    let test_request = TestRequest {
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
