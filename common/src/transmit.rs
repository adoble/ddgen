use embedded_hal::spi::{Operation, SpiDevice};

use crate::{command::Command, deserialize::Deserialize, serialize::Serialize, DeviceError};

pub trait Transmit<SPI, RESP>: Serialize + Command
where
    SPI: SpiDevice,
    RESP: Deserialize<RESP>,
{
    fn transmit<const REQ_MAX_LEN: usize, const RESP_MAX_LEN: usize>(
        &self,
        spi: &mut SPI,
    ) -> Result<RESP, DeviceError> {
        let opcode: [u8; 1] = [self.opcode()];

        let (mut size, mut data, provider) = self.serialize::<REQ_MAX_LEN>();

        for provided_element in provider {
            data[size] = provided_element;
            size += 1;
        }
        let mut response_buf = [0_u8; RESP_MAX_LEN];

        spi.transaction(&mut [
            Operation::Write(&opcode),
            Operation::Write(&data[0..size]),
            Operation::Read(&mut response_buf),
        ])
        .map_err(|_| DeviceError::Transmit)?;
        // why not .map_err(DeviceError::Transmit)?

        // let response = response_buf[0..].deserialize()
        let response = RESP::deserialize(&response_buf[0..])?;

        Ok(response)
    }

    fn polled_transmit<
        const REQ_MAX_LEN: usize,
        const RESP_MAX_LEN: usize,
        HEADER: Deserialize<HEADER>,
        const STATUS_HEADER_LEN: usize,
    >(
        &self,
        spi: &mut SPI,
        //status_header: HEADER,
        status_fn: fn(HEADER) -> bool,
    ) -> Result<RESP, DeviceError> {
        let opcode: [u8; 1] = [self.opcode()];

        let (mut size, mut data, provider) = self.serialize::<REQ_MAX_LEN>();
        // TODO Should the followng code be added to the serialize function? Signature
        // would then be (size, data) = self.serialize::<REQ_MAX_LEN>()
        for provided_element in provider {
            data[size] = provided_element;
            size += 1;
        }

        let mut response_buf = [0_u8; RESP_MAX_LEN];

        // Read the first header
        spi.transaction(&mut [
            Operation::Write(&opcode),
            Operation::Write(&data[0..size]),
            Operation::Read(&mut response_buf[0..STATUS_HEADER_LEN]),
        ])
        .map_err(|_| DeviceError::Transmit)?;

        loop {
            let header = HEADER::deserialize(&response_buf[0..STATUS_HEADER_LEN])
                .map_err(|_| DeviceError::Receive)?;

            if status_fn(header) {
                // Read in the rest of the response
                spi.transaction(&mut [Operation::Read(&mut response_buf[STATUS_HEADER_LEN..])])
                    .map_err(|_| DeviceError::Transmit)?;
                break;
            } else {
                // Repeat the read
                spi.transaction(&mut [Operation::Read(&mut response_buf[0..STATUS_HEADER_LEN])])
                    .map_err(|_| DeviceError::Receive)?;
            }
        }

        RESP::deserialize(&response_buf)
    }
}
