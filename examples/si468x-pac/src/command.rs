use embedded_hal::digital::OutputPin;
// use embedded_hal::spi::Error;
// use embedded_hal::spi::Operation;
use embedded_hal::spi::Error;
use embedded_hal::spi::SpiBus;

use crate::error::DeviceError;

pub trait Command<SPI, CS, REQUEST, const REQUEST_SIZE: usize, RESPONSE, const RESPONSE_SIZE: usize>
where
    SPI: SpiBus,
    CS: OutputPin,
    REQUEST: CommandRequest<REQUEST_SIZE>,
    RESPONSE: CommandResponse<RESPONSE_SIZE>,
{
    // fn send(&self, mut spi: SPI, mut request: REQUEST) -> Result<RESPONSE, DeviceError> {
    //     let request_buf: [u8; REQUEST_SIZE] = request.serialize();

    //     let mut response_buf = [0; RESPONSE_SIZE];

    //     self.cs_pin.set_low().map_err(|_| DeviceError::Sbb);

    //     spi.write(&request_buf);
    //     spi.read(&response_buf);

    //     self.cs_pin.set_high.map_err(|_| DeviceError::Sbb);

    //     let response = RESPONSE::deserialize(&response_buf)?;
    //     Ok(response)
    // }

    //fn send<F>(&mut self, f: F) -> Result<CommandResponse<RESPONSE, RESPONSE_SIZE>, DeviceError>
    fn send<F>(&self, f: F) -> Result<RESPONSE, DeviceError>
    where
        // F: FnOnce(&mut CommandRequest<REQUEST_SIZE>), //-> &mut GetSysStateRequest,
        F: FnOnce(&mut REQUEST), //-> &mut GetSysStateRequest,
    {
        let mut request = CommandRequest::<REQUEST_SIZE>::new();
        f(&mut request);

        // TODO? REQUEST_SIZE is really MAX_REQUEST_SIZE?
        let request_buf: [u8; REQUEST_SIZE] = request.serialize();
        let mut response_buf: [u8; RESPONSE_SIZE] = [0; RESPONSE_SIZE];

        let request_buf_len = request.byte_len();

        self.cs().set_low().map_err(|_| DeviceError::Sbb)?;

        self.spi()
            .write(&request_buf[..request_buf_len])
            .map_err(|e| DeviceError::Transfer(e.kind()))?;

        self.spi()
            .read(&mut response_buf)
            .map_err(|e| DeviceError::Transfer(e.kind()))?;

        self.cs().set_high().map_err(|_| DeviceError::Sbb)?;

        let response = CommandResponse::deserialize(&response_buf)?;
        Ok(response)
    }

    fn cs(&self) -> &CS;

    fn spi(&self) -> &SPI;
}

pub trait CommandRequest<const REQUEST_SIZE: usize> {
    fn new() -> Self;

    fn serialize(&mut self) -> [u8; REQUEST_SIZE];

    /// This the size of the request (excluding the opcode) in bytes.
    /// If the size is variabe depending on values in the request,
    /// then this should be overwritten to calculate the actual
    /// value.
    // TODO this should handle different word sizes, not just bytes
    fn byte_len(&self) -> usize {
        // Returns usize as this is often used to index buffer arrays
        REQUEST_SIZE
    }
}

// pub trait CommandResponse<RESPONSE, const RESPONSE_SIZE: usize> {
//     fn deserialize(buffer: &[u8; RESPONSE_SIZE]) -> Result<RESPONSE, DeviceError>;
// }

pub trait CommandResponse<const RESPONSE_SIZE: usize> {
    fn deserialize(buffer: &[u8; RESPONSE_SIZE]) -> Result<Self, DeviceError>
    where
        Self: Sized;

    fn byte_len(&self) -> usize {
        todo!()
    }
}
