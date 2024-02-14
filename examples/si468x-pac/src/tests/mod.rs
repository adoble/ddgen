#[cfg(test)]
use super::*;

use crate::command::Command;
use crate::get_sys_state::*;
use crate::types::*;

use embedded_hal::spi::SpiBus;
use embedded_hal_mock::eh1::pin::{
    Mock as PinMock, State as PinState, Transaction as PinTransaction,
};
use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
use embedded_hal_nb::spi::FullDuplex;

#[test]
fn an_spi_test() {
    let expectations = [
        SpiTransaction::write(0x09),
        SpiTransaction::read(0x0A),
        SpiTransaction::write(0xFE),
        SpiTransaction::read(0xFF),
        SpiTransaction::write_vec(vec![1, 2]),
        SpiTransaction::transfer_in_place(vec![3, 4], vec![5, 6]),
    ];

    let mut spi = SpiMock::new(&expectations);
    // FullDuplex transfers
    FullDuplex::write(&mut spi, 0x09).unwrap();
    assert_eq!(FullDuplex::read(&mut spi).unwrap(), 0x0A);
    FullDuplex::write(&mut spi, 0xFE).unwrap();
    assert_eq!(FullDuplex::read(&mut spi).unwrap(), 0xFF);

    // Writing
    SpiBus::write(&mut spi, &vec![1, 2]).unwrap();

    // Transferring
    let mut buf = vec![3, 4];
    spi.transfer_in_place(&mut buf).unwrap();
    assert_eq!(buf, vec![5, 6]);

    // Finalise expectations
    spi.done();
}
#[test]
fn integration_test_spi() {
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

    let mut spi = SpiMock::new(&spi_expectations);
    let mut cs = PinMock::new(&pin_expectations);

    // Need to clone the parameters as Si486x takes ownership which cause problems later with
    // spi.done() and cs.done()
    let mut device = Si468xPac::new(spi.clone(), cs.clone());
    let response = device.get_sys_state.send(|req| req.arg1 = 0x00).unwrap();

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
