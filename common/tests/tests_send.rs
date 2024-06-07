use common::transmit::Transmit;
use common::DeviceError;
use common::{
    command::Command, deserialize::Deserialize, request::RequestWord, response::ResponseBit,
    serialize::Serialize,
};
//use embedded_hal::digital::{InputPin, OutputPin, StatefulOutputPin};
//use embedded_hal::spi::{Operation, SpiDevice};
use embedded_hal::spi::SpiDevice;

use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
// use embedded_hal_mock::eh1::{
//     digital::{Mock as PinMock, State as PinState, Transaction as PinTransaction},
//     MockError,
// };

const SIMPLE_REQUEST_OPCODE: u8 = 0x09;

struct SimpleRequest {
    arg1: u8,
}

impl SimpleRequest {
    // This needs to be generated
    pub fn send<SPI: SpiDevice>(&self, spi: &mut SPI) -> Result<SimpleResponse, DeviceError> {
        let response_buf = self.transmit::<2, 1>(spi)?;

        let response: SimpleResponse = response_buf.deserialize()?;

        Ok(response)
    }
}

impl<SPI: SpiDevice> Transmit<SPI, SimpleResponse> for SimpleRequest {}

impl Command for SimpleRequest {
    fn opcode(&self) -> u8 {
        SIMPLE_REQUEST_OPCODE
    }
}

impl Serialize for SimpleRequest {
    fn serialize<const LEN: usize>(&self) -> (usize, [u8; LEN], impl Iterator<Item = u8>) {
        let mut data = [0u8; LEN];
        #[allow(unused_variables)]
        let provider = core::iter::empty::<u8>();

        data[0].serialize_word(self.arg1);

        (1, data, provider)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct SimpleResponse {
    status: bool,
}

impl Deserialize<SimpleResponse> for [u8] {
    fn deserialize(&self) -> Result<SimpleResponse, common::DeviceError> {
        let status = self[0].deserialize_bit(0);

        Ok(SimpleResponse { status })
    }
}

#[test]
fn test_simple_request() {
    let request = SimpleRequest { arg1: 8 };

    let expected_response = SimpleResponse { status: true };

    //let (size, data, _) = request.serialize::<2>();

    let spi_expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write(0x09),
        SpiTransaction::write(8),
        SpiTransaction::read(0b000_0001), // Ths status bit
        SpiTransaction::transaction_end(),
    ];

    // let cs_expectations = [
    //     PinTransaction::set(PinState::Low),
    //     PinTransaction::set(PinState::High),
    // ];

    let mut spi = SpiMock::new(&spi_expectations);

    // let mut cs = PinMock::new(&cs_expectations);

    let response: SimpleResponse = request.send(&mut spi).unwrap();

    assert_eq!(response, expected_response);

    spi.done();
}
