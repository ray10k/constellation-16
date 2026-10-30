use crate::hardware::memory::Memory;
use crate::hardware::peripheral::Peripheral;
use crate::hardware::processor::{TickResult, VirtualCpu};

pub struct System {
    cpu: VirtualCpu,
    memory: Memory,
    peripherals: Vec<Box<dyn Peripheral>>,
}
