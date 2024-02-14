use crate::DeviceError;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Default)]
pub enum PowerUpState {
    #[default]
    Reset = 0,
    BootLoaderRunning = 2,
    AppRunning = 3,
}

impl TryFrom<u8> for PowerUpState {
    type Error = DeviceError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Reset),
            1 => Ok(Self::BootLoaderRunning),
            3 => Ok(Self::AppRunning),
            _ => Err(DeviceError::EnumConversion),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum ActiveProcessingImage {
    Bootloader = 0,
    Fmhd = 1,
    Dab = 2,
    TdmbOrDataOnlyDab = 3,
    FmhdDemod = 4,
    Amhd = 5,
    AmhdDemod = 6,
    DabDemod = 7,
}

impl TryFrom<u8> for ActiveProcessingImage {
    type Error = DeviceError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Bootloader),
            1 => Ok(Self::Fmhd),
            2 => Ok(Self::Dab),
            3 => Ok(Self::TdmbOrDataOnlyDab),
            4 => Ok(Self::FmhdDemod),
            5 => Ok(Self::Amhd),
            6 => Ok(Self::DabDemod),
            _ => Err(DeviceError::EnumConversion),
        }
    }
}
