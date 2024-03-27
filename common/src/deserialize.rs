use crate::error::DeviceError;

pub trait Deserialize<T> {
    fn deserialize(&self) -> Result<T, DeviceError>;
}
