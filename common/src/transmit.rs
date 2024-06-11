use embedded_hal::spi::{Operation, SpiDevice};

use crate::{command::Command, serialize::Serialize, DeviceError};

pub trait Transmit<SPI, RESP>: Serialize + Command
where
    SPI: SpiDevice,
{
    fn transmit<const REQ_MAX_LEN: usize, const RESP_MAX_LEN: usize>(
        &self,
        spi: &mut SPI,
    ) -> Result<[u8; RESP_MAX_LEN], DeviceError> {
        let opcode: [u8; 1] = [self.opcode()];

        let (mut size, mut data, provider) = self.serialize::<REQ_MAX_LEN>();

        for provided_element in provider {
            data[size] = provided_element;
            size += 1;
        }

        let mut response_buf = [0 as u8; RESP_MAX_LEN];
        spi.transaction(&mut [
            Operation::Write(&opcode),
            Operation::Write(&data[0..size]),
            Operation::Read(&mut response_buf),
        ])
        .map_err(|_| DeviceError::Transmit)?;
        // why not .map_err(DeviceError::Transmit)?

        Ok(response_buf)
    }

    #[allow(unused)]
    fn polled_transmit<const REQ_MAX_LEN: usize, const RESP_MAX_LEN: usize>(
        &self,
        spi: &mut SPI,
        f: fn() -> bool,
    ) -> Result<[u8; RESP_MAX_LEN], DeviceError> {
        Ok([0; RESP_MAX_LEN])
    }
}
