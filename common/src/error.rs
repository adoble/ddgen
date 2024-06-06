#[derive(Copy, Clone, Debug)]
pub enum DeviceError {
    EnumConversion,
    BitPositionOutOfRange,
    CsAssert,
    Transmit,
    Receive,
}
