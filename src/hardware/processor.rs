use std::collections::VecDeque;

use crate::hardware::instruction::check_for_jump;

use super::word::Word;
use super::instruction::{DecodedInstruction,AOperand,BOperand};

#[derive(Default)]
pub struct Registers {
    //General-purpose registers.
    pub reg_a:Word,
    pub reg_b:Word,
    pub reg_c:Word,
    pub reg_x:Word,
    pub reg_y:Word,
    pub reg_z:Word,
    pub reg_i:Word,
    pub reg_j:Word,
}

#[derive(Default)]
pub struct Processor {
    //"Special" registers.
    /// Pointer to the next instruction.
    program_counter:Word,
    /// Pointer to the most recently used stack location.
    stack_pointer:Word,
    /// Special register that 'catches' mathematical overflows.
    reg_excess:Word,
    /// Address that program execution jumps to in order to handle an interrupt.
    interrupt_address:Word,

    /// Storage space for unhandled interrupts.
    interrupt_queue:VecDeque<Word>,
    /// Interrupt state. If `false`, the next tick will handle the front-most interrupt
    /// (if any are in the queue.)
    queue_incoming_interrupts:bool,

    /// Most recently decoded instruction that has not finished executing.
    current_instruction:Option<DecodedInstruction>,
    /// Progress towards executing the instruction.
    instruction_state:ProcessorState,
}

#[derive(Default)]
enum ProcessorState {
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
    Stall(u16)
}


impl Processor {
    pub fn tick(&mut self, registers:&mut Registers, memory:&mut [Word]) {
        match self.instruction_state {

            ProcessorState::Ready => {
                if !self.queue_incoming_interrupts
                && !self.interrupt_queue.is_empty(){
                    todo!("Handle interrupt de-queueing.");
                    return;
                }
                //No waiting interrupts, ready to start executing an instruction. So, decode
                //the next instruction that PC points at.
                self.current_instruction = memory[self.program_counter.to_usize()].try_into().ok();
                //Figure out what further steps (if any) are needed.
                let instruction = &mut self.current_instruction;
                match instruction {
                    None => todo!("Bad instruction handling."),
                    Some(instruction) 
                        if instruction.operand_a.has_delay() => {
                            self.instruction_state = ProcessorState::FetchA
                    },
                    Some(instruction) 
                        if let Some(op_b) = instruction.operand_b 
                        && op_b.has_delay() => {
                            instruction.fetched_a = todo!("Fetch `a` operand.");
                            self.instruction_state = ProcessorState::FetchB
                    },
                    Some(instruction) => {
                        instruction.fetched_a = todo!("Fetch `a` operand.");
                        instruction.fetched_b = todo!("Fetch `b` operand.");
                        self.instruction_state = ProcessorState::Stall(instruction.opcode.duration())
                    }
                }
            },

            ProcessorState::FetchA => {    
                let instruction = self.current_instruction.as_mut();
                match instruction {
                    None => todo!("Bad instruction handling."),
                    Some(instruct) => {
                        instruct.fetched_a = todo!("Fetch `a` operand.");
                        if let Some(b_op) = instruct.operand_b {
                            if b_op.has_delay() {
                                self.instruction_state = ProcessorState::FetchB;
                            } else {
                                instruct.fetched_b = todo!("Fetch `b` operand.");
                                self.instruction_state = ProcessorState::Stall(instruct.opcode.duration());
                            }
                        }
                    }
                }
            },

            ProcessorState::FetchB => {
                let instruction = self.current_instruction.as_mut();
                match instruction {
                    None => todo!("Bad instruction handling."),
                    Some(instruct) => {
                        instruct.fetched_b = todo!("Fetch `b` operand.");
                        self.instruction_state = ProcessorState::Stall(instruct.opcode.duration());
                    }
                }
            },

            ProcessorState::SkipCondition => {
                //May need to skip multiple instructions.
                if check_for_jump(memory[self.program_counter.to_usize()]) {
                    let next_instruction:DecodedInstruction = memory[self.program_counter.to_usize()].try_into().expect("Error when decoding instruction to be skipped.");
                    self.program_counter += next_instruction.word_size();
                } else {
                    self.instruction_state = ProcessorState::Ready;
                }
            },
            ProcessorState::Stall(_) => todo!(),
        }
    }
}
