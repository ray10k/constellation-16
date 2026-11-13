use crate::hardware::memory::{Memory, MemOp};
use crate::hardware::peripheral::Peripheral;
use crate::hardware::processor::{TickResult, VirtualCpu};
use crate::hardware::instruction::AOperand;
use crate::hardware::word::{Word, ZERO};

use std::fmt::Debug;

pub struct VirtualComputer {
    cpu: VirtualCpu,
    memory: Memory,
    peripherals: Vec<Box<dyn Peripheral>>,
}

impl Default for VirtualComputer {
    fn default() -> Self {
        Self { cpu: Default::default(), memory: Memory::new_empty(), peripherals: Default::default() }
    }
}

impl Debug for VirtualComputer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("System")
            .field("CPU-registers", &self.cpu.registers)
            .field("CPU-hidden",&self.cpu.hidden_state)
            .field("peripheral-count",&self.peripherals.len())
            .field("memory-size",&self.memory.borrow().len())
            .finish()
    }
}

impl VirtualComputer {
    fn new_with_initial(initial_memory:&[Word]) -> Self {
        Self {
            cpu: Default::default(),
            memory: Memory::new_preinit(initial_memory),
            peripherals: Default::default()
        }
    }

    fn push_peripheral(&mut self, peripheral:Box<dyn Peripheral>) {
        self.peripherals.push(peripheral);
    }

    fn run_tick(mut self) -> Result<(),()> {
        let tick = self.cpu.processor_step(self.memory.dup());
        self.cpu = tick.0;
        match tick.1 {
            Ok(t_result) => {
                match t_result {
                    TickResult::PartialInstr => {/* Currently, a do-nothing.*/},
                    TickResult::Instruction => {/* Currently, a do-nothing.*/},
                    TickResult::FakeInterrupt => {/* Currently, a do-nothing.*/},
                    TickResult::Interrupt => {/* Currently, a do-nothing.*/},
                    TickResult::QueryHardware(word) => {
                        if word.to_usize() < self.peripherals.len() {
                            let peripheral_info = self.peripherals[word.to_usize()].query().to_words();
                            self.cpu.registers.reg_a = peripheral_info[0];
                            self.cpu.registers.reg_b = peripheral_info[1];
                            self.cpu.registers.reg_c = peripheral_info[2];
                            self.cpu.registers.reg_x = peripheral_info[3];
                            self.cpu.registers.reg_y = peripheral_info[4];
                        } else { //nonexistant peripheral. Just set the registers to 0, rather than (for instance) raising an interrupt.
                            self.cpu.registers.reg_a = ZERO;
                            self.cpu.registers.reg_b = ZERO;
                            self.cpu.registers.reg_c = ZERO;
                            self.cpu.registers.reg_x = ZERO;
                            self.cpu.registers.reg_y = ZERO;
                        }
                    },
                    TickResult::NumberHardware(to_where) => {
                        let peripheral_count = Word(self.peripherals.len() as u16);
                        if let Ok(where_) = to_where.try_into() {
                            self.cpu.store_value(&where_, self.memory.dup(), false, peripheral_count);
                        } //Trying to write to an invalid location will silently fail anyway.
                    },
                    TickResult::InterruptHardware(hw_id) => {
                        if hw_id.to_usize() < self.peripherals.len() {
                            let hw_target = &mut self.peripherals[hw_id.to_usize()];
                            hw_target.interrupt(self.memory.dup(), &mut self.cpu.registers);
                        }
                    },
                }
            },
            Err(fail_addr) => {
                println!("Error processing word with value {fail_addr}");
                return Err(())
            }
        }

        Ok(())
    }
}
