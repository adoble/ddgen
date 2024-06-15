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

struct ProviderTestRequest {
    arg1: u8,
    provided_field: Provider,
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct Provider {
    number: u8,
    limit: u8,
}
impl Provider {
    pub fn new(limit: u8) -> Provider {
        Provider { number: 0, limit }
    }
}

impl Iterator for Provider {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.number < self.limit {
            self.number += 1;
            Some(self.number)
        } else {
            None
        }
    }
}

impl ProviderTestRequest {
    // This needs to be generated
    pub fn send<SPI: SpiDevice>(&self, spi: &mut SPI) -> Result<ProviderTestResponse, DeviceError> {
        let response = self.transmit::<10, 1>(spi)?;

        Ok(response)
    }
}

impl<SPI: SpiDevice> Transmit<SPI, ProviderTestResponse> for ProviderTestRequest {}

impl Command for ProviderTestRequest {
    fn opcode(&self) -> u8 {
        SIMPLE_REQUEST_OPCODE
    }
}

impl Serialize for ProviderTestRequest {
    fn serialize<const LEN: usize>(&self) -> (usize, [u8; LEN], impl Iterator<Item = u8>) {
        let mut data = [0u8; LEN];
        #[allow(unused_variables)]
        let provider = core::iter::empty::<u8>();

        data[0].serialize_word(self.arg1);
        let provider = self.provided_field;

        (1, data, provider)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct ProviderTestResponse {
    status: bool,
}

// impl Deserialize<ProviderTestResponse> for [u8] {
//     fn deserialize(&self) -> Result<ProviderTestResponse, common::DeviceError> {
//         let status = self[0].deserialize_bit(0);

//         Ok(ProviderTestResponse { status })
//     }
// }

impl Deserialize<Self> for ProviderTestResponse {
    fn deserialize(buf: &[u8]) -> Result<ProviderTestResponse, common::DeviceError> {
        let status = buf[0].deserialize_bit(0);

        Ok(Self { status })
    }
}
#[test]
fn test_provider_request() {
    let request = ProviderTestRequest {
        arg1: 8,
        provided_field: Provider::new(5),
    };

    let expected_response = ProviderTestResponse { status: true };

    //let (size, data, _) = request.serialize::<2>();

    let spi_expectations = [
        SpiTransaction::transaction_start(),
        // Opcode
        SpiTransaction::write(0x09),
        // All arguments
        SpiTransaction::write_vec(vec![8, 1, 2, 3, 4, 5]),
        // Read response
        SpiTransaction::read(0b000_0001), // Ths status bit
        SpiTransaction::transaction_end(),
    ];

    let mut spi = SpiMock::new(&spi_expectations);

    // let mut cs = PinMock::new(&cs_expectations);

    let response: ProviderTestResponse = request.send(&mut spi).unwrap();

    assert_eq!(response, expected_response);

    spi.done();
}

#[test]
fn test_provider_to_see_if_i_have_it_right() {
    let mut provider = Provider::new(5);

    for i in 1..=5 {
        assert_eq!(provider.next().unwrap(), i);
    }

    let provider = Provider::new(5);
    let mut sum = 0;
    for i in provider {
        sum += i;
    }
    assert_eq!(sum, 15);

    let provider = Provider::new(5);

    let sum: u8 = provider.into_iter().sum();
    assert_eq!(sum, 15);
}
