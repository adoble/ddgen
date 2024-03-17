use crate::error::DeviceError;

pub trait Deserialize<T> {
    fn deserialize(bytes: &[u8]) -> Result<T, DeviceError>;
}
