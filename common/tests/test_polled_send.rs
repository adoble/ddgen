use common::response::ResponseWord;
use common::transmit::Transmit;
use common::DeviceError;
use common::{
    command::Command, deserialize::Deserialize, request::RequestBit, request::RequestWord,
    response::ResponseBit, serialize::Serialize,
};
//use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
//use embedded_hal::spi::{Operation, SpiDevice};
use embedded_hal::spi::{Operation, SpiDevice};

use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
// use embedded_hal_mock::eh1::{
//     digital::{Mock as PinMock, State as PinState, Transaction as PinTransaction},
//     MockError,
// };

const POLLED_REQUEST_OPCODE: u8 = 0x10;

struct PolledRequest {
    arg1: u8,
}

impl PolledRequest {
    // This needs to be generated and made more general
    pub fn send<SPI: SpiDevice>(&self, spi: &mut SPI) -> Result<PolledResponse, DeviceError> {
        // For the si468x need to read in the first 4 bytes each time and see if the CTS bit is set.
        // How to generalise this?
        // Could make the status part a header, but this then gives the header concept new semantics.

        let opcode: [u8; 1] = [self.opcode()];

        const REQ_MAX_LEN: usize = 1;

        let (mut size, mut data, provider) = self.serialize::<REQ_MAX_LEN>();
        for provided_element in provider {
            data[size] = provided_element;
            size += 1;
        }

        const RESP_MAX_LEN: usize = 2;
        let mut response_buf = [0 as u8; RESP_MAX_LEN];

        const STATUS_HEADER_LEN: usize = 1;

        // Read the first header
        spi.transaction(&mut [
            Operation::Write(&opcode),
            Operation::Write(&data[0..size]),
            Operation::Read(&mut response_buf[0..STATUS_HEADER_LEN]),
        ])
        .map_err(|_| DeviceError::Transmit)?;

        loop {
            let header = StatusHeader::deserialize(&response_buf[0..STATUS_HEADER_LEN])
                .map_err(|_| DeviceError::Receive)?;

            // TODO Timeout and delay between iterations
            if header.status {
                spi.transaction(&mut [Operation::Read(&mut response_buf[STATUS_HEADER_LEN..])])
                    .map_err(|_| DeviceError::Transmit)?;
                break;
            } else {
                spi.transaction(&mut [Operation::Read(&mut response_buf[0..STATUS_HEADER_LEN])])
                    .map_err(|_| DeviceError::Receive)?;
            }
        }

        Ok(PolledResponse::deserialize(&response_buf)?)
    }
}

impl<SPI: SpiDevice> Transmit<SPI, PolledResponse> for PolledRequest {}

impl Command for PolledRequest {
    fn opcode(&self) -> u8 {
        POLLED_REQUEST_OPCODE
    }
}

impl Serialize for PolledRequest {
    fn serialize<const LEN: usize>(&self) -> (usize, [u8; LEN], impl Iterator<Item = u8>) {
        let mut data = [0u8; LEN];
        #[allow(unused_variables)]
        let provider = core::iter::empty::<u8>();

        data[0].serialize_word(self.arg1);

        (1, data, provider)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct PolledResponse {
    status_header: StatusHeader,
    some_data: u8,
}

// This is just one byte to keep it simple
#[derive(Debug, PartialEq, Clone, Copy)]
struct StatusHeader {
    status: bool,
}

impl Serialize for StatusHeader {
    fn serialize<const LEN: usize>(&self) -> (usize, [u8; LEN], impl Iterator<Item = u8>) {
        let mut data = [0u8; LEN];
        #[allow(unused_variables)]
        let provider = core::iter::empty::<u8>();

        data[0].serialize_bit(self.status, 0);

        (1, data, provider)
    }
}

impl Deserialize<Self> for StatusHeader {
    fn deserialize(buf: &[u8]) -> Result<StatusHeader, DeviceError> {
        let status = buf[0].deserialize_bit(0);

        Ok(Self { status })
    }
}

impl Deserialize<Self> for PolledResponse {
    fn deserialize(buf: &[u8]) -> Result<PolledResponse, common::DeviceError> {
        let status_header = StatusHeader::deserialize(&buf[0..=0])?;
        let some_data = buf[1].deserialize_word();

        Ok(Self {
            status_header,
            some_data,
        })
    }
}

#[test]
// Single poll
fn test_polled_request_1() {
    let request = PolledRequest { arg1: 8 };

    let expected_response = PolledResponse {
        status_header: StatusHeader { status: true },
        some_data: 0xAA,
    };

    //let (size, data, _) = request.serialize::<2>();

    let spi_expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write(0x10),
        SpiTransaction::write(8),
        SpiTransaction::read_vec(vec![0b000_0001]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::read_vec(vec![0xAA]),
        SpiTransaction::transaction_end(),
    ];

    // let cs_expectations = [
    //     PinTransaction::set(PinState::Low),
    //     PinTransaction::set(PinState::High),
    // ];

    let mut spi = SpiMock::new(&spi_expectations);

    // let mut cs = PinMock::new(&cs_expectations);

    let response: PolledResponse = request.send(&mut spi).unwrap();

    assert_eq!(response, expected_response);

    spi.done();
}

#[test]
// Multiple polls
fn test_polled_request_n() {
    let request = PolledRequest { arg1: 8 };

    let expected_response = PolledResponse {
        status_header: StatusHeader { status: true },
        some_data: 0xAA,
    };

    //let (size, data, _) = request.serialize::<2>();

    let spi_expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write(0x10),
        SpiTransaction::write(8),
        SpiTransaction::read_vec(vec![0b000_0000]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::read_vec(vec![0b000_0000]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::read_vec(vec![0b000_0001]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::read_vec(vec![0xAA]),
        SpiTransaction::transaction_end(),
    ];

    // let cs_expectations = [
    //     PinTransaction::set(PinState::Low),
    //     PinTransaction::set(PinState::High),
    // ];

    let mut spi = SpiMock::new(&spi_expectations);

    // let mut cs = PinMock::new(&cs_expectations);

    let response: PolledResponse = request.send(&mut spi).unwrap();

    assert_eq!(response, expected_response);

    spi.done();
}
