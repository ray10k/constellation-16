use crate::hardware::{processor::VirtualCpu, word::Word};

pub trait Peripheral {
    /// Called after each processor tick. If the peripheral emits an interrupt, return Some(message).
    fn tick(&mut self, memory: &mut [Word]) -> Option<Word>;

    /// Called after the processor has performed a hardware interrupt targeting this peripheral. Must return the number
    /// of additional cycles that the processor gets stalled for, even if this is 0 additional cycles.
    fn interrupt(&mut self, memory: &mut [Word], registers: &mut VirtualCpu) -> u16;
}
