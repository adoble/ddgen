/// DAB_SET_FREQ_LIST command sets the DAB frequency table. The frequencies are in units
/// of 1 kHz. The table can be populated with a single entry or a regional list (for
/// example 5 or 6 entries). It is recommended to make the list regional to increase
/// scanning speed. The command is complete when the CTS bit (and optional interrupt) is
/// set. The ERR bit (and optional interrupt) is set if an invalid argument is sent.
/// Note that only a single interrupt occurs if both the CTS and ERR bits are set.
/// The command may only be sent in powerup mode.
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;

//use crate::command::Command;
use crate::command::Command;
use crate::command::CommandRequest;
use crate::command::CommandResponse;
use crate::error::DeviceError;
use crate::response::ResponseWord;
use crate::types::PowerUpState;

// These are generated
const OPCODE: u8 = 0xB8;

pub struct DabSetFreqList<'a, SPI, CS> {
    spi: &'a SPI,
    cs: &'a CS,
}

impl<'a, SPI, CS> DabSetFreqList<'a, SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    pub fn new(spi: &'a SPI, cs: &'a CS) -> Self {
        Self {
            //_marker: core::marker::PhantomData,
            spi,
            cs,
        }
    }

    // pub fn send<F>(&mut self, f: F) -> Result<DabSetFreqListResponse, DeviceError>
    // where
    //     F: FnOnce(&mut DabSetFreqListRequest), //-> &mut GetSysStateRequest,
    // {
    //     let mut request = DabSetFreqListRequest::new();
    //     f(&mut request);

    //     let request_buf: [u8; 98] = request.serialize();

    //     let mut response_buf: [u8; 4] = [0; 4]; // RESPONSE_SIZE
    //     self.cs_pin.set_low().map_err(|_| DeviceError::Sbb)?;

    //     let request_buf_len = request.byte_len();

    //     self.spi
    //         .write(&request_buf[..request_buf_len])
    //         .map_err(|e| DeviceError::Transfer(e.kind()))?;

    //     self.spi
    //         .read(&mut response_buf)
    //         .map_err(|e| DeviceError::Transfer(e.kind()))?;

    //     self.cs_pin.set_high().map_err(|_| DeviceError::Sbb)?;

    //     let response = DabSetFreqListResponse::deserialize(&response_buf)?;
    //     Ok(response)
    // }
}

impl<'a, SPI, CS> Command<SPI, CS, DabSetFreqListRequest, 98, DabSetFreqListResponse, 4>
    for DabSetFreqList<'a, SPI, CS>
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

#[derive(Debug, PartialEq, Eq)]
pub struct DabSetFreqListRequest {
    // This should be generated from the TOML definition
    pub number_frequencies: u8,
    pub frequencies: [u16; 48],
    buffer: [u8; (48 * 2) + 1 + 1],
}

impl Default for DabSetFreqListRequest {
    fn default() -> Self {
        Self {
            number_frequencies: Default::default(),
            frequencies: [0; 48],
            buffer: [0; (48 * 2) + 2],
        }
    }
}

impl CommandRequest<98> for DabSetFreqListRequest {
    fn new() -> Self {
        Self {
            number_frequencies: 0,
            frequencies: [0; 48],
            buffer: [0; (48 * 2) + 1 + 1],
        }
    }
    fn serialize(&mut self) -> [u8; (48 * 2) + 1 + 1] {
        self.buffer[0] = OPCODE;
        self.buffer[1] = self.number_frequencies;

        // MSB first according to data sheet
        for i in 0..self.number_frequencies {
            let index = i as usize;
            let words = self.frequencies[index].to_be_bytes();
            self.buffer[(index * 2) + 2] = words[0];
            self.buffer[(index * 2) + 3] = words[1];
        }

        self.buffer
    }

    fn byte_len(&self) -> usize {
        ((self.number_frequencies * 2) + 2) as usize
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct DabSetFreqListResponse {
    pub cts: bool,
    pub err_cmd: bool,
    pub dacqint: bool,
    pub dsrvint: bool,
    pub stcint: bool,
    pub devntint: bool,
    pub pup_state: PowerUpState,
    pub dsp_err: bool,
    pub repo_fatal_error: bool,
    pub cmdo_fatal_error: bool,
    pub arb_error: bool,
    pub error_nr: bool,
}

impl<const RESPONSE_SIZE: usize> CommandResponse<RESPONSE_SIZE> for DabSetFreqListResponse {
    fn deserialize(response_buf: &[u8; RESPONSE_SIZE]) -> Result<Self, DeviceError> {
        let mut response = Self {
            ..Default::default()
        };

        response.cts = response_buf[0].bit(7);
        response.err_cmd = response_buf[0].bit(6);
        response.dacqint = response_buf[0].bit(7); //{ words = "0", bits = "5", description = "Digital radio link change interrupt indicator." }
        response.dsrvint = response_buf[0].bit(4); //{ words = "0", bits = "4", description = "An enabled data component of one of the digital services requires attention." }
        response.stcint = response_buf[0].bit(0); //{ words = "0", bits = "0", description = "Seek / tune complete" }
        response.devntint = response_buf[1].bit(5); // { words = "1", bits = "5", description = "Digital radio event change interrupt indicator." } # ??
        response.pup_state = response_buf[3].field(6, 7).try_into()?;
        response.dsp_err = response_buf[3].bit(5);
        response.repo_fatal_error = response_buf[3].bit(3); // { words = "3", bits = "3" }
        response.cmdo_fatal_error = response_buf[3].bit(2);
        response.arb_error = response_buf[3].bit(1); // { words = "3", bits = "1" }
        response.error_nr = response_buf[3].bit(0); // { words = "3", bits = "0" }

        Ok(response)
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
        let frequencies = {
            let mut frequencies = [0 as u16; 48];
            frequencies[..4].clone_from_slice(&[15000, 15450, 16200, 18900]);
            frequencies
        };

        let mut request = DabSetFreqListRequest {
            number_frequencies: 4,
            frequencies,
            ..Default::default()
        };

        let buf = request.serialize();

        let expected = {
            let mut expected = [0; 98];

            //  Override beginning of array.
            expected[..10]
                .clone_from_slice(&[0xB8, 4, 0x3A, 0x98, 0x3C, 0x5A, 0x3F, 0x48, 0x49, 0xD4]);

            expected
        };

        assert_eq!(buf, expected);
    }

    #[test]
    fn deserialize_response() {
        let response_buf: [u8; 4] = [
            0b1010_0000, // CTS and DACQINT set
            0b0010_0000, // DEVNTINI set
            0b0000_0000, // dummy byte
            0b1100_0001, // PUP_STATE = APP_RUNNING, ERRNR set
        ];

        let response = DabSetFreqListResponse::deserialize(&response_buf).unwrap();

        assert_eq!(
            response,
            DabSetFreqListResponse {
                cts: true,
                dacqint: true,
                devntint: true,
                pup_state: PowerUpState::AppRunning,
                error_nr: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn send_command() {
        let frequencies = {
            let mut frequencies = [0 as u16; 48];
            frequencies[..4].clone_from_slice(&[15000, 15450, 16200, 18900]);
            frequencies
        };

        // Sending the command with four frequencies - see variable frequencies
        // Cannot use write_vec as these are all given as individual bytes due to
        // the variable length of the packet.
        let spi_expectations = [
            // SpiTransaction::write(0xB8),
            // SpiTransaction::write(4),
            // SpiTransaction::write(0x3A),
            // SpiTransaction::write(0x98),
            // SpiTransaction::write(0x3C),
            // SpiTransaction::write(0x5A),
            // SpiTransaction::write(0x3F),
            // SpiTransaction::write(0x48),
            // SpiTransaction::write(0x49),
            // SpiTransaction::write(0xD4),
            SpiTransaction::write_vec(vec![
                0xB8, 4, 0x3A, 0x98, 0x3C, 0x5A, 0x3F, 0x48, 0x49, 0xD4,
            ]),
            SpiTransaction::read_vec(vec![
                0b1010_0000, // CTS and DACQINT set
                0b0010_0000, // DEVNTINI set
                0b0000_0000, // dummy byte
                0b1100_0001, // PUP_STATE = APP_RUNNING, ERRNR set
            ]),
        ];

        let pin_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];

        let mut cs = PinMock::new(&pin_expectations);

        let mut spi = SpiMock::new(&spi_expectations);

        let mut dab_set_freq_list_command = DabSetFreqList::new(&mut spi, &mut cs);

        let response = dab_set_freq_list_command
            .send(|req| {
                req.number_frequencies = 4;
                req.frequencies = frequencies;
            })
            .unwrap();

        assert_eq!(
            response,
            DabSetFreqListResponse {
                cts: true,
                dacqint: true,
                devntint: true,
                pup_state: PowerUpState::AppRunning,
                error_nr: true,
                ..Default::default()
            }
        );

        spi.done();
        cs.done();
    }
}
