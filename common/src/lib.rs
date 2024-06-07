pub mod bits;
pub mod command;
pub mod deserialize;
pub mod error;
pub mod request;
pub mod response;
pub mod serialize;
pub mod transmit;

pub use crate::error::DeviceError;

// #[cfg(test)]
// //mod test_bit_spec_impl;
// mod tests;
