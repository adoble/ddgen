use crate::error::DeviceError;

// pub trait Deserialize<T> {
//     fn deserialize(&self) -> Result<T, DeviceError>;
// }

pub trait Deserialize<T> {
    //fn deserialize_old(&self) -> Result<T, DeviceError>;
    fn deserialize(buf: &[u8]) -> Result<T, DeviceError>;
}
