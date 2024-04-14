#![allow(unused_imports)]
/// Command TEST_COMMAND
/// A simple command
/// Generated with version 0.1.0 of ddgen

use crate::deserialize::Deserialize;
use crate::error::DeviceError;
use crate::request::{RequestArray, RequestBit, RequestField, RequestWord};
use crate::response::{ResponseArray, ResponseBit, ResponseField, ResponseWord};
use crate::serialize::Serialize;
use crate::types::*;

#[derive(Debug, PartialEq)]
pub struct TestCommandRequest {
    a_word: u8,
}

impl Serialize for TestCommandRequest {
    fn serialize<const N: usize>(&self) -> (u8, [u8; N]) {
        let mut data = [0u8; N];

        data[0].serialize_word(self.a_word);

        (1, data)
    }

}

#[derive(Debug, PartialEq)]
pub struct TestCommandResponse {
    a_word: u8,
}

impl Deserialize<TestCommandResponse> for [u8] {

    fn deserialize(&self) -> Result<TestCommandResponse, DeviceError> {
        Ok(TestCommandResponse {
            a_word: self[0].deserialize_word(),
        })
    }
}