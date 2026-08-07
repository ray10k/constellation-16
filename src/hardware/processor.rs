use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::hardware::processor::TickStep::{FetchB, Stall};

use super::instruction::{AOperand, BOperand, DecodedInstruction};
use super::word::Word;

type Memory = Rc<RefCell<[Word]>>;

#[derive(Default)]
pub struct Registers {
    //General-purpose registers.
    pub reg_a: Word,
    pub reg_b: Word,
    pub reg_c: Word,
    pub reg_x: Word,
    pub reg_y: Word,
    pub reg_z: Word,
    pub reg_i: Word,
    pub reg_j: Word,
}

#[derive(Default)]
struct ProcessorHiddenState {
    //"Special" registers.
    /// Pointer to the next instruction.
    program_counter: Word,
    /// Pointer to the most recently used stack location.
    stack_pointer: Word,
    /// Special register that 'catches' mathematical overflows.
    reg_excess: Word,
    /// Address that program execution jumps to in order to handle an interrupt.
    interrupt_address: Word,

    /// Storage space for unhandled interrupts.
    interrupt_queue: VecDeque<Word>,
    /// Interrupt state. If `false`, the next tick will handle the front-most interrupt
    /// (if any are in the queue.)
    queue_incoming_interrupts: bool,

    /// Most recently decoded instruction that has not finished executing.
    current_instruction: Option<DecodedInstruction>,
    /// Progress towards executing the instruction.
    instruction_state: TickStep,
}

pub struct VirtualCpu {
    pub registers: Registers,
    hidden_state: ProcessorHiddenState,
}

#[derive(Default, Clone)]
enum TickStep {
    #[default]
    /// The processor is at the start of the current instruction.
    Ready,
    /// Fetching the `a` operand resulted in a delay.
    FetchA,
    /// Fetching the `b` operand resulted in a delay.
    FetchB,
    /// A conditional operation has resulted in a 1-cycle delay to skip an instruction.
    SkipCondition,
    /// Some other condition has resulted in a delay. Goes down by 1 until end of instruction.
    Stall(u16),
}

/*
impl ProcessorHiddenState {
    pub fn tick(&mut self, registers:&mut Registers, memory:&mut [Word]) {
        match self.instruction_state {
            TickStep::Ready => {
                if !self.queue_incoming_interrupts
                && !self.interrupt_queue.is_empty(){
                    todo!("Handle interrupt de-queueing.");
                    return;
                }
                //No waiting interrupts, ready to start executing an instruction. So, decode
                //the next instruction that PC points at.
                let mut instruction: Option<DecodedInstruction>  = memory[self.program_counter.to_usize()].try_into().ok();
                //Figure out what further steps (if any) are needed.

                match instruction.as_mut() {
                    None => todo!("Bad instruction handling."),
                    Some(instruction)
                        if instruction.operand_a.has_delay() => {
                            self.instruction_state = TickStep::FetchA
                    },
                    Some(instruction)
                        if let Some(op_b) = instruction.operand_b.as_ref()
                        && op_b.has_delay() => {
                            let (fetched_a, registers, processor, memory) = fetch_a_value(&instruction.operand_a, registers, self, memory);
                            instruction.fetched_a = fetched_a;
                            processor.instruction_state = TickStep::FetchB
                    },
                    Some(instruction) => {
                        let (fetched_a, registers, processor, memory) =
                            fetch_a_value(&instruction.operand_a, registers, self, memory);
                        let (fetched_b, registers, processor, memory) =
                            fetch_b_value(instruction.operand_b.as_ref().expect("Bad operand"), registers, processor, memory);
                        instruction.fetched_a = fetched_a;
                        instruction.fetched_b = fetched_b;
                        processor.instruction_state = TickStep::Stall(instruction.opcode.duration())
                    }
                }
                self.current_instruction = instruction;
            },

            TickStep::FetchA => {
                let instruction = self.current_instruction.as_mut();
                match instruction {
                    None => todo!("Bad instruction handling."),
                    Some(instruct) => {
                        let (fetched_a, registers, processor, memory) = fetch_a_value(&instruct.operand_a, registers, self, memory);

                        instruct.fetched_a = fetched_a;
                        if let Some(b_op) = instruct.operand_b {
                            if b_op.has_delay() {
                                processor.instruction_state = TickStep::FetchB;
                            } else {
                                let (fetched_b, registers, processor, memory) =
                                    fetch_b_value(&b_op, registers, processor, memory);

                                instruct.fetched_b = fetched_b;
                                processor.instruction_state = TickStep::Stall(instruct.opcode.duration());
                            }
                        }
                    }
                }
            },

            TickStep::FetchB => {
                let instruction = self.current_instruction.as_mut();
                match instruction {
                    None => todo!("Bad instruction handling."),
                    Some(instruct) => {
                        let (fetched_b, registers, processor, memory) =
                            fetch_b_value(instruct.operand_b.as_ref().expect("Bad operand"), registers, self, memory);

                        instruct.fetched_b = fetched_b;
                        processor.instruction_state = TickStep::Stall(instruct.opcode.duration());
                    }
                }
            },

            TickStep::SkipCondition => {
                //May need to skip multiple instructions.
                if check_for_jump(memory[self.program_counter.to_usize()]) {
                    let next_instruction:DecodedInstruction = memory[self.program_counter.to_usize()].try_into().expect("Error when decoding instruction to be skipped.");
                    self.program_counter += next_instruction.word_size();
                } else {
                    self.instruction_state = TickStep::Ready;
                }
            },

            TickStep::Stall(1) | TickStep::Stall(0) => {
                todo!("Perform actual instruction");
            }

            TickStep::Stall(ticks) =>
                self.instruction_state = TickStep::Stall(ticks - 1),
        }
    }
}*/

impl VirtualCpu {
    /// Update the state of the processor by one clock-step. This may result in an instruction
    /// executing all in one go, or result in part of the instruction getting executed for
    /// longer-running operations.
    ///
    /// Returns the `VirtualCpu` itself, and if parsing the instruction failed, the value that
    /// caused the parse-failure.
    pub fn processor_step(mut self, memory: Memory) -> (Self, Option<Word>) {
        loop {
            let tick_step = self.hidden_state.instruction_state.clone();
            match tick_step {
                TickStep::Ready => {
                    //Check if an interrupt is waiting and needs to be handled first!
                    if !self.hidden_state.queue_incoming_interrupts
                        && !self.hidden_state.interrupt_queue.is_empty()
                    {
                        todo!("Handle interrupts.");
                    }
                    //No waiting interrupts, ready to start executing an instruction. So, decode
                    //the next instruction that PC points at.
                    let instruction: Option<DecodedInstruction> = memory.borrow_mut()
                        [self.hidden_state.program_counter.to_usize()]
                    .try_into()
                    .ok();
                    if let None = instruction {
                        let err_location = self.hidden_state.program_counter.to_usize();
                        return (self, Some(memory.borrow()[err_location]));
                    }
                    self.hidden_state.current_instruction = instruction;
                    self.hidden_state.instruction_state = TickStep::FetchA;
                    continue;
                }
                TickStep::FetchA => {
                    let instruction = self
                        .hidden_state
                        .current_instruction
                        .as_mut()
                        .expect("No instruction decoded.");
                    let operand = instruction.operand_a.clone();
                    let opcode = instruction.opcode.clone();

                    let op_a = self.fetch_a_value(operand.clone(), Rc::clone(&memory));
                    self.hidden_state
                        .current_instruction
                        .as_mut()
                        .unwrap()
                        .fetched_a = op_a;

                    //Time to figure out what the next step is.
                    //Option 1: this is a 'special' opcode, and there is no
                    //operand b. Jump straight to `Stall`.
                    if opcode.is_special() {
                        self.hidden_state.instruction_state = Stall(opcode.duration());
                    }
                    //Option 2: this is a 'normal' opcode, so jump to FetchB.
                    else {
                        self.hidden_state.instruction_state = FetchB;
                    }

                    //next: either the current operand has a Cycle-cost (C value in the table) of 0,
                    //or of 1. So, either break out of the loop and pick up next tick, or just continue
                    //to handle the next step.
                    if operand.has_delay() {
                        break;
                    }
                    continue;
                }
                TickStep::FetchB => {
                    let instruction = self
                        .hidden_state
                        .current_instruction
                        .as_mut()
                        .expect("No instruction decoded.");
                    let operand = instruction
                        .operand_b
                        .as_ref()
                        .expect("Attempted to fetch `b` operand for operation without `b` operand.")
                        .clone();
                    let opcode = instruction.opcode.clone();

                    let op_b = self.fetch_b_value(&operand, Rc::clone(&memory));
                    self.hidden_state
                        .current_instruction
                        .as_mut()
                        .unwrap()
                        .fetched_b = op_b;

                    //Next step planning time again!
                    //This time, the only decision is 'Does the `b` operand incur a delay?'
                    self.hidden_state.instruction_state = Stall(opcode.duration());

                    if operand.has_delay() {
                        break;
                    }
                    continue;
                }
                TickStep::SkipCondition => todo!(),
                TickStep::Stall(_) => todo!(),
            }
        }

        (self, None)
    }

    fn fetch_a_value(&mut self, operand: AOperand, memory: Memory) -> Word {
        let mem_ref = memory.borrow_mut();
        let retval = match operand {
            AOperand::RegA => self.registers.reg_a,
            AOperand::RegB => self.registers.reg_b,
            AOperand::RegC => self.registers.reg_c,
            AOperand::RegX => self.registers.reg_x,
            AOperand::RegY => self.registers.reg_y,
            AOperand::RegZ => self.registers.reg_z,
            AOperand::RegI => self.registers.reg_i,
            AOperand::RegJ => self.registers.reg_j,
            AOperand::DerefA => {
                let reg_temp = self.registers.reg_a.to_usize();
                mem_ref[reg_temp]
            }
            AOperand::DerefB => {
                let reg_temp = self.registers.reg_b.to_usize();
                mem_ref[reg_temp]
            }
            AOperand::DerefC => {
                let reg_temp = self.registers.reg_c.to_usize();
                mem_ref[reg_temp]
            }
            AOperand::DerefX => {
                let reg_temp = self.registers.reg_x.to_usize();
                mem_ref[reg_temp]
            }
            AOperand::DerefY => {
                let reg_temp = self.registers.reg_y.to_usize();
                mem_ref[reg_temp]
            }
            AOperand::DerefZ => {
                let reg_temp = self.registers.reg_z.to_usize();
                mem_ref[reg_temp]
            }
            AOperand::DerefI => {
                let reg_temp = self.registers.reg_i.to_usize();
                mem_ref[reg_temp]
            }
            AOperand::DerefJ => {
                let reg_temp = self.registers.reg_j.to_usize();
                mem_ref[reg_temp]
            }
            AOperand::OffsetA => {
                let reg_temp = self.registers.reg_a.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            AOperand::OffsetB => {
                let reg_temp = self.registers.reg_b.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            AOperand::OffsetC => {
                let reg_temp = self.registers.reg_c.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            AOperand::OffsetX => {
                let reg_temp = self.registers.reg_x.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            AOperand::OffsetY => {
                let reg_temp = self.registers.reg_y.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            AOperand::OffsetZ => {
                let reg_temp = self.registers.reg_z.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            AOperand::OffsetI => {
                let reg_temp = self.registers.reg_i.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            AOperand::OffsetJ => {
                let reg_temp = self.registers.reg_j.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            AOperand::Pop => {
                let stack_pointer = self.hidden_state.stack_pointer.to_usize();
                self.hidden_state.stack_pointer -= 1;
                mem_ref[stack_pointer]
            }
            AOperand::Peek => mem_ref[self.hidden_state.stack_pointer.to_usize()],
            AOperand::Pick => {
                let stack_pointer = self.hidden_state.stack_pointer.to_usize();
                let next_word = mem_ref[self.hidden_state.program_counter.to_usize()];
                self.hidden_state.program_counter += 1;
                mem_ref[stack_pointer + next_word.to_usize()]
            }
            AOperand::StackPointer => self.hidden_state.stack_pointer,
            AOperand::ProgramCounter => self.hidden_state.program_counter,
            AOperand::Excess => self.hidden_state.reg_excess,
            AOperand::DerefImmediate => {
                let next_word = mem_ref[self.hidden_state.program_counter.to_usize()];
                self.hidden_state.program_counter += 1;
                mem_ref[next_word.to_usize()]
            }
            AOperand::ValueImmediate => {
                let next_word = mem_ref[self.hidden_state.program_counter.to_usize()];
                self.hidden_state.program_counter += 1;
                next_word
            }
            AOperand::Literal(word) => word,
        };
        retval
            .try_into()
            .expect("impossible Word conversion error.")
    }

    fn fetch_b_value(&mut self, operand: &BOperand, memory: Memory) -> Word {
        let mem_ref = memory.borrow_mut();
        let retval = match operand {
            BOperand::RegA => self.registers.reg_a,
            BOperand::RegB => self.registers.reg_b,
            BOperand::RegC => self.registers.reg_c,
            BOperand::RegX => self.registers.reg_x,
            BOperand::RegY => self.registers.reg_y,
            BOperand::RegZ => self.registers.reg_z,
            BOperand::RegI => self.registers.reg_i,
            BOperand::RegJ => self.registers.reg_j,
            BOperand::DerefA => {
                let reg_temp = self.registers.reg_a.to_usize();
                mem_ref[reg_temp]
            }
            BOperand::DerefB => {
                let reg_temp = self.registers.reg_b.to_usize();
                mem_ref[reg_temp]
            }
            BOperand::DerefC => {
                let reg_temp = self.registers.reg_c.to_usize();
                mem_ref[reg_temp]
            }
            BOperand::DerefX => {
                let reg_temp = self.registers.reg_x.to_usize();
                mem_ref[reg_temp]
            }
            BOperand::DerefY => {
                let reg_temp = self.registers.reg_y.to_usize();
                mem_ref[reg_temp]
            }
            BOperand::DerefZ => {
                let reg_temp = self.registers.reg_z.to_usize();
                mem_ref[reg_temp]
            }
            BOperand::DerefI => {
                let reg_temp = self.registers.reg_i.to_usize();
                mem_ref[reg_temp]
            }
            BOperand::DerefJ => {
                let reg_temp = self.registers.reg_j.to_usize();
                mem_ref[reg_temp]
            }
            BOperand::OffsetA => {
                let reg_temp = self.registers.reg_a.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            BOperand::OffsetB => {
                let reg_temp = self.registers.reg_b.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            BOperand::OffsetC => {
                let reg_temp = self.registers.reg_c.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            BOperand::OffsetX => {
                let reg_temp = self.registers.reg_x.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            BOperand::OffsetY => {
                let reg_temp = self.registers.reg_y.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            BOperand::OffsetZ => {
                let reg_temp = self.registers.reg_z.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            BOperand::OffsetI => {
                let reg_temp = self.registers.reg_i.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            BOperand::OffsetJ => {
                let reg_temp = self.registers.reg_j.to_usize();
                let pc = self.hidden_state.program_counter.to_usize();
                self.hidden_state.program_counter += 1;
                mem_ref[reg_temp + pc]
            }
            BOperand::Push => {
                //Note to self: If used as a destination value, the stack pointer
                //gets updated when writing the result value.
                let stack_pointer = self.hidden_state.stack_pointer.to_usize();
                mem_ref[stack_pointer]
            }
            BOperand::Peek => mem_ref[self.hidden_state.stack_pointer.to_usize()],
            BOperand::Pick => {
                let stack_pointer = self.hidden_state.stack_pointer.to_usize();
                let next_word = mem_ref[self.hidden_state.program_counter.to_usize()];
                self.hidden_state.program_counter += 1;
                mem_ref[stack_pointer + next_word.to_usize()]
            }
            BOperand::StackPointer => self.hidden_state.stack_pointer,
            BOperand::ProgramCounter => self.hidden_state.program_counter,
            BOperand::Excess => self.hidden_state.reg_excess,
            BOperand::DerefImmediate => {
                let next_word = mem_ref[self.hidden_state.program_counter.to_usize()];
                self.hidden_state.program_counter += 1;
                mem_ref[next_word.to_usize()]
            }
            BOperand::ValueImmediate => {
                let next_word = mem_ref[self.hidden_state.program_counter.to_usize()];
                self.hidden_state.program_counter += 1;
                next_word
            }
        };
        retval
    }
}
