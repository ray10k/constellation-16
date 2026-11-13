use crate::memory::Memory;
use crate::processor::Registers;
use crate::word::Word;

/// Description for a peripheral, contains the information needed for HWQ instructions.
pub struct PeripheralInfo {
    ///Unique identifier for the hardware, for the given manufacturer.
    pub hardware_id:u32,
    ///16-bit version number for the current firmware/revision.
    pub version_id:u16,
    ///Unique identifier for a given manufacturer.
    pub manufact_id:u32
}

impl PeripheralInfo {
    ///Convenience function to split the data in the struct into 16-bit `Word`s.
    pub fn to_words(&self) -> [Word;5] {
        [
            Word((self.hardware_id & 0xffff) as u16),
            Word(((self.hardware_id & 0xffff0000) >> 16) as u16),
            Word(self.version_id),
            Word((self.manufact_id & 0xffff) as u16),
            Word(((self.manufact_id & 0xffff0000) >> 16) as u16),
        ]
    }
}

pub trait Peripheral {
    /// Called when the peripheral is initialized, or when the clock speed of the system changes.
    fn init(&mut self, ticks_per_second:usize);

    /// Called after each processor tick. If the peripheral emits an interrupt, return Some(message).
    fn tick(&mut self, memory: &mut [Word]) -> Option<Word>;

    /// Called after the processor has performed a hardware interrupt targeting this peripheral. Must return the number
    /// of additional cycles that the processor gets stalled for, even if this is 0 additional cycles.
    fn interrupt(&mut self, memory: Memory, registers: &mut Registers) -> u16;

    /// Called in response to the `HWQ` instruction, expects the peripheral to provide hardware- and manufacturer
    /// information.
    fn query(&self) -> PeripheralInfo;
}
