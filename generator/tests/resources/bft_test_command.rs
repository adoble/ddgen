#![allow(unused_imports)]
/// Command BFT_TEST_COMMAND
/// Use to test bits, fields and basic types
/// Generated with version 0.1.0 of ddgen

use crate::deserialize::Deserialize;
use crate::error::DeviceError;
use crate::request::{RequestArray, RequestBit, RequestField, RequestWord};
use crate::response::{ResponseArray, ResponseBit, ResponseField, ResponseWord};
use crate::serialize::Serialize;
use crate::types::*;

#[derive(Debug, PartialEq)]
pub struct BftTestCommandRequest {
    /// Some status bit
    a_bit: bool,
    a_field: TestField,
    a_u16: u16,
    a_u8: u8,
    a_i16: i16,
}

impl Serialize for BftTestCommandRequest {
    fn serialize<const N: usize>(&self) -> (u8, [u8; N]) {
        let mut data = [0u8; N];

        data[0].serialize_bit(self.a_bit, 4);
        data[0].serialize_field(self.a_field as u8, 5, 6);
        data[1..=2].serialize_word(self.a_u16);
        data[3].serialize_word(self.a_u8);
        data[4..=5].serialize_word(self.a_i16);

        (6, data)
    }

}

#[derive(Debug, PartialEq)]
pub struct BftTestCommandResponse {
    a_bit: bool,
    a_field: TestField,
    a_u16: u16,
    a_u8: u8,
    a_i16: i16,
}

impl Deserialize<BftTestCommandResponse> for [u8] {

    fn deserialize(&self) -> Result<BftTestCommandResponse, DeviceError> {
        Ok(BftTestCommandResponse {
            a_bit: self[0].deserialize_bit(4),
            a_field: self[0].deserialize_field(5, 6).try_into()?,
            a_u16: self[1..=2].deserialize_word(),
            a_u8: self[3].deserialize_word(),
            a_i16: self[4..=5].deserialize_word(),
        })
    }
}