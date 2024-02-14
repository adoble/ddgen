use embedded_hal::spi::ErrorKind;

#[derive(Copy, Clone, Debug)]
pub enum DeviceError {
    EnumConversion,
    Transfer(ErrorKind),

    /// The SBB (chip select) pin cannot be set
    Sbb,
}
