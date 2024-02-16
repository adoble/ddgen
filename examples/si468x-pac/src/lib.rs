#![cfg_attr(not(test), no_std)]
use dab_set_freq_list::DabSetFreqList;
use embedded_hal::digital::OutputPin;

use embedded_hal::spi::SpiBus;
use get_sys_state::GetSysState;

pub use crate::error::DeviceError;

pub mod command;

pub mod types;

pub mod dab_set_freq_list;
pub mod get_sys_state;
// Other commands follow
// ...

pub mod response;

// pub mod types;

pub mod error;

pub struct Si468xPac<SPI, CS> {
    spi: SPI,
    cs: CS,
    // pub get_sys_state: &'a GetSysState<'a, SPI, CS>,
    // pub dab_set_freq_list: &'a DabSetFreqList<'a, SPI, CS>,
    //commands: &'a CommandBlock<SPI, CS>,
}

impl<SPI, CS> Si468xPac<SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> Self {
        //let commands = CommandBlock::new(&mut spi, &mut cs_pin);

        Self {
            spi,
            cs,
            // get_sys_state: &GetSysState::new(&mut spi, &mut cs),
            // dab_set_freq_list: &DabSetFreqList::new(&mut spi, &mut cs),
        }

        //commands,
        // get_sys_state: GetSysState::new(&self.spi, &self.cs),
        // dab_set_freq_list: DabSetFreqList::new(&self.spi, &self.cs),
    }

    // pub fn commands(&self) -> &CommandBlock<SPI, CS> {
    //     &self.commands
    // }

    // pub fn get_sys_state(&mut self) -> GetSysState<SPI, CS> {
    //     GetSysState::new(self.spi, self.cs_pin)
    // }

    pub fn get_sys_state(&mut self) -> GetSysState<SPI, CS> {
        GetSysState::new(&mut self.spi, &mut self.cs)
    }

    pub fn dab_set_freq_list(&mut self) -> DabSetFreqList<SPI, CS> {
        DabSetFreqList::new(&mut self.spi, &mut self.cs)
    }
}

#[cfg(test)]
mod tests;
