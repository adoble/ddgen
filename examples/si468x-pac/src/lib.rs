#![cfg_attr(not(test), no_std)]
use dab_set_freq_list::DabSetFreqList;
use embedded_hal::digital::OutputPin;

use embedded_hal::spi::SpiBus;
use get_sys_state::GetSysState;

pub use crate::command_block::CommandBlock;
pub use crate::error::DeviceError;

pub mod command;
pub mod command_block;

pub mod types;

pub mod dab_set_freq_list;
pub mod get_sys_state;
// Other commands follow
// ...

pub mod response;

// pub mod types;

pub mod error;

pub struct Si468xPac<'a, SPI, CS> {
    spi: SPI,
    cs: CS,
    pub get_sys_state: &'a GetSysState<'a, SPI, CS>,
    pub dab_set_freq_list: &'a DabSetFreqList<'a, SPI, CS>,
    //commands: &'a CommandBlock<SPI, CS>,
}

impl<'a, SPI, CS> Si468xPac<'a, SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    pub fn new(spi: SPI, cs: CS) -> &'a Self {
        //let commands = CommandBlock::new(&mut spi, &mut cs_pin);

        let mut device = Self {
            spi,
            cs,
            get_sys_state: &GetSysState::new(&spi, &cs),
            dab_set_freq_list: &DabSetFreqList::new(&spi, &cs),
        };

        &device
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
}

#[cfg(test)]
mod tests;
