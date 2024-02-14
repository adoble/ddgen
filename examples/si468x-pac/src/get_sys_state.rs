/// Command GET_SYS_STATE
///
/// Example client code
///    let request = GetSysState { ... };
///    let get_sys_state_command = GetSysState::new();
///    let response = get_sys_state_command.send(&mut spi, request).unwrap();
///
///
/// Really only want this and then CommandBlock can deal with the initialiation of SPI and CS
///
/// let commands = CommandBlock::new(spi, cs);  
///
/// let request = commands.get_sys_state.request()   // Cannot create GetSysState alone - look up factory methios in rust
/// request.arg1 = 0x0;
///
/// let response  = commands.get_sys_state.send(request).unwrap();
///
/// ALT
///
/// let device = SPI468x::new(spi, cs).unwrap();
///
/// let command = device.commands.get_sys_state // ()??
/// command.arg1 = 0x00
///
/// let response = command.send().unwrap();
/// let response = device.commands.get_sys_state.request(|req| {req.args1 = 0x00}).send().unwrap();
///
/// ALT
/// let device = SPI468x::new(spi, cs).unwrap();
/// let response = device.commands.get_sys_state.send(|req| {req.args1 = 0x00}).unwrap();
///
///  TODO FIRST, WRITE SOME TESTS FOR THIS API AND WORK DOWN FROM THERE.
use embedded_hal::digital::OutputPin;

use embedded_hal::spi::SpiBus;

//use crate::command::Command;
use crate::command::Command;
use crate::command::CommandRequest;
use crate::command::CommandResponse;
use crate::error::DeviceError;
use crate::response::ResponseWord;
use crate::types::{ActiveProcessingImage, PowerUpState};

// These are generated
const OPCODE: u8 = 0x09;

pub struct GetSysState<'a, SPI, CS> {
    // Need this so that the compile accepts the generic SPI type
    // even though the struct has no data entries that use it.
    //_marker: core::marker::PhantomData<SPI>,
    spi: &'a SPI,
    cs: &'a CS,
}

// impl<SPI, CS> Command<SPI, CS, GetSysStateRequest, 2, GetSysStateResponse, 6>
//     for GetSysState<'_, SPI, CS>
// where
//     SPI: SpiBus,
//     CS: OutputPin,
// {
// }

impl<'a, SPI, CS> GetSysState<'a, SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    pub fn new(spi: &'a SPI, cs: &'a CS) -> Self {
        GetSysState {
            //_marker: core::marker::PhantomData,
            spi,
            cs,
        }
    }

    // pub fn send<F>(&mut self, f: F) -> Result<GetSysStateResponse, DeviceError>
    // where
    //     F: FnOnce(&mut GetSysStateRequest), //-> &mut GetSysStateRequest,
    // {
    //     let mut request = GetSysStateRequest::new();
    //     f(&mut request);

    //     let request_buf: [u8; 2] = request.serialize();

    //     let mut response_buf: [u8; 6] = [0; 6];

    //     self.cs_pin.set_low().map_err(|_| DeviceError::Sbb)?;

    //     self.spi
    //         .write(&request_buf)
    //         .map_err(|e| DeviceError::Transfer(e.kind()))?;
    //     self.spi
    //         .read(&mut response_buf)
    //         .map_err(|e| DeviceError::Transfer(e.kind()))?;

    //     self.cs_pin.set_high().map_err(|_| DeviceError::Sbb)?;

    //     let response = GetSysStateResponse::deserialize(&response_buf)?;
    //     Ok(response)
    // }
}

impl<'a, SPI, CS> Command<SPI, CS, GetSysStateRequest, 2, GetSysStateResponse, 6>
    for GetSysState<'a, SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    fn cs(&self) -> &'a CS {
        self.cs
    }

    fn spi(&self) -> &'a SPI {
        self.spi
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct GetSysStateRequest {
    // This is generated from the TOML definition
    pub arg1: u8, // Defaults to zero. How??
                  //buffer: [u8; 2],
}

impl CommandRequest<2> for GetSysStateRequest {
    fn new() -> Self {
        Self {
            arg1: 0,
            //buffer: [0, 0],
        }
    }
    fn serialize(&mut self) -> [u8; 2] {
        // self.buffer = [OPCODE, self.arg1];
        // self.buffer
        [OPCODE, self.arg1]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GetSysStateResponse {
    pub cts: bool,
    pub err_cmd: bool,
    pub pup_state: PowerUpState,
    pub repo_fatal_error: bool,
    pub cmdo_fatal_error: bool,
    pub arb_error: bool,
    pub error_nr: bool,
    pub image: ActiveProcessingImage,
}

impl Default for GetSysStateResponse {
    fn default() -> Self {
        Self {
            cts: false,
            err_cmd: false,
            pup_state: PowerUpState::Reset,
            repo_fatal_error: false,
            cmdo_fatal_error: false,
            arb_error: false,
            error_nr: false,
            image: ActiveProcessingImage::Bootloader,
        }
    }
}

impl<const RESPONSE_SIZE: usize> CommandResponse<RESPONSE_SIZE> for GetSysStateResponse {
    fn deserialize(response_buf: &[u8; RESPONSE_SIZE]) -> Result<Self, DeviceError> {
        let cts = response_buf[0].bit(7);
        let err_cmd = response_buf[0].bit(6);
        let pup_state: PowerUpState = response_buf[3].field(6, 7).try_into()?;
        let repo_fatal_error = response_buf[3].bit(3); // { words = "3", bits = "3" }
        let cmdo_fatal_error = response_buf[3].bit(2);

        let arb_error = response_buf[3].bit(1); // { words = "3", bits = "1" }
        let error_nr = response_buf[3].bit(0); // { words = "3", bits = "0" }
        let image: ActiveProcessingImage = response_buf[4].field(0, 7).try_into()?;
        // { words = "4", bits = "0..7", enum = "active_processing_image" }

        Ok(Self {
            cts,
            err_cmd,
            pup_state,
            repo_fatal_error,
            cmdo_fatal_error,
            arb_error,
            error_nr,
            image,
        })
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1::pin::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

    use super::*;
    #[test]
    fn serialise_request() {
        let mut request = GetSysStateRequest {
            ..Default::default()
        };

        let buf = request.serialize();

        assert_eq!(buf, [0x09, 0]);
    }

    #[test]
    fn deserialize_response() {
        let response_buf: [u8; 6] = [
            0b1000_0000,
            0b0000_0000,
            0b0000_0000,
            0b1100_0000,
            0b0000_0010,
            0b0000_0000,
        ];

        let response = GetSysStateResponse::deserialize(&response_buf).unwrap();

        assert_eq!(
            response,
            GetSysStateResponse {
                cts: true,
                pup_state: PowerUpState::AppRunning,
                image: ActiveProcessingImage::Dab,
                ..Default::default()
            }
        )
    }

    #[test]
    fn send_command() {
        let spi_expectations = [
            SpiTransaction::write_vec(vec![0x09, 0x00]),
            SpiTransaction::read_vec(vec![
                0b1000_0000,
                0b0000_0000,
                0b0000_0000,
                0b1100_0000,
                0b0000_0010,
                0b0000_0000,
            ]),
        ];

        let pin_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];

        let mut cs = PinMock::new(&pin_expectations);

        let mut spi = SpiMock::new(&spi_expectations);

        let mut get_sys_state_command = GetSysState::new(&mut spi, &mut cs);

        let response = get_sys_state_command
            .send(|req| {
                req.arg1 = 0;
            })
            .unwrap();

        assert_eq!(
            response,
            GetSysStateResponse {
                cts: true,
                pup_state: PowerUpState::AppRunning,
                image: ActiveProcessingImage::Dab,
                ..Default::default()
            }
        );

        spi.done();
        cs.done();
    }
}
