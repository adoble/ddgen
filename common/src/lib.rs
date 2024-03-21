mod deserialize;
mod error;
mod request;
mod response;
mod serialize;

pub use crate::error::DeviceError;

#[cfg(test)]
//mod test_bit_spec_impl;
mod tests;
