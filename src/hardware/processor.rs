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
                        if let Some(op_b) = instruction.operand_b.as_ref() 
                        && op_b.has_delay() => {
                            let (fetched_a, registers, processor, memory) = fetch_a_value(&instruction.operand_a, registers, self, memory);
                            instruction.fetched_a = fetched_a;
                            processor.instruction_state = ProcessorState::FetchB
                    },
                    Some(instruction) => {
                        let (fetched_a, registers, processor, memory) = 
                            fetch_a_value(&instruction.operand_a, registers, self, memory);
                        let (fetched_b, registers, processor, memory) = 
                            fetch_b_value(instruction.operand_b.as_ref().expect("Bad operand"), registers, processor, memory);
                        instruction.fetched_a = fetched_a;
                        instruction.fetched_b = fetched_b;
                        processor.instruction_state = ProcessorState::Stall(instruction.opcode.duration())
                    }
                }
            },

            ProcessorState::FetchA => {    
                let instruction = self.current_instruction.as_mut();
                match instruction {
                    None => todo!("Bad instruction handling."),
                    Some(instruct) => {
                        let (fetched_a, registers, processor, memory) = fetch_a_value(&instruct.operand_a, registers, self, memory);
                            
                        instruct.fetched_a = fetched_a;
                        if let Some(b_op) = instruct.operand_b {
                            if b_op.has_delay() {
                                processor.instruction_state = ProcessorState::FetchB;
                            } else {
                                let (fetched_b, registers, processor, memory) = 
                                    fetch_b_value(&b_op, registers, processor, memory);
                                
                                instruct.fetched_b = fetched_b;
                                processor.instruction_state = ProcessorState::Stall(instruct.opcode.duration());
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
                        let (fetched_b, registers, processor, memory) = 
                            fetch_b_value(instruct.operand_b.as_ref().expect("Bad operand"), registers, self, memory);
                                
                        instruct.fetched_b = fetched_b;
                        processor.instruction_state = ProcessorState::Stall(instruct.opcode.duration());
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
                        
            ProcessorState::Stall(1) | ProcessorState::Stall(0) => {
                todo!("Perform actual instruction");
            }

            ProcessorState::Stall(ticks) => 
                self.instruction_state = ProcessorState::Stall(ticks - 1),
        }
    }
}

fn fetch_a_value<'a>(operand:&'a AOperand, registers:&'a mut Registers, processor:&'a mut Processor, memory:&mut [Word]) 
    -> (Word, &'a mut Registers, &'a mut Processor, &'a mut [Word]) {
    let retval = match operand {
        AOperand::RegA => registers.reg_a,
        AOperand::RegB => registers.reg_b,
        AOperand::RegC => registers.reg_c,
        AOperand::RegX => registers.reg_x,
        AOperand::RegY => registers.reg_y,
        AOperand::RegZ => registers.reg_z,
        AOperand::RegI => registers.reg_i,
        AOperand::RegJ => registers.reg_j,
        AOperand::DerefA => {
            let reg_temp = registers.reg_a.to_usize();
            memory[reg_temp]
        },
        AOperand::DerefB => {
            let reg_temp = registers.reg_b.to_usize();
            memory[reg_temp]
        },
        AOperand::DerefC => {
            let reg_temp = registers.reg_c.to_usize();
            memory[reg_temp]
        },
        AOperand::DerefX => {
            let reg_temp = registers.reg_x.to_usize();
            memory[reg_temp]
        },
        AOperand::DerefY => {
            let reg_temp = registers.reg_y.to_usize();
            memory[reg_temp]
        },
        AOperand::DerefZ => {
            let reg_temp = registers.reg_z.to_usize();
            memory[reg_temp]
        },
        AOperand::DerefI => {
            let reg_temp = registers.reg_i.to_usize();
            memory[reg_temp]
        },
        AOperand::DerefJ => {
            let reg_temp = registers.reg_j.to_usize();
            memory[reg_temp]
        },
        AOperand::OffsetA => {
            let reg_temp = registers.reg_a.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        AOperand::OffsetB => {
            let reg_temp = registers.reg_b.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        AOperand::OffsetC => {
            let reg_temp = registers.reg_c.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        AOperand::OffsetX => {
            let reg_temp = registers.reg_x.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        AOperand::OffsetY => {
            let reg_temp = registers.reg_y.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        AOperand::OffsetZ => {
            let reg_temp = registers.reg_z.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        AOperand::OffsetI => {
            let reg_temp = registers.reg_i.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        AOperand::OffsetJ => {
            let reg_temp = registers.reg_j.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        AOperand::Pop => {
            let stack_pointer = processor.stack_pointer.to_usize();
            processor.stack_pointer -= 1;
            memory[stack_pointer]
        },
        AOperand::Peek => memory[processor.stack_pointer.to_usize()],
        AOperand::Pick => {
            let stack_pointer = processor.stack_pointer.to_usize();
            let next_word = memory[processor.program_counter.to_usize()];
            processor.program_counter += 1;
            memory[stack_pointer + next_word.to_usize()]
        },
        AOperand::StackPointer => processor.stack_pointer,
        AOperand::ProgramCounter => processor.program_counter,
        AOperand::Excess => processor.reg_excess,
        AOperand::DerefImmediate => {
            let next_word = memory[processor.program_counter.to_usize()];
            processor.program_counter += 1;
            memory[next_word.to_usize()]
        },
        AOperand::ValueImmediate => {
            let next_word = memory[processor.program_counter.to_usize()];
            processor.program_counter += 1;
            next_word
        },
        AOperand::Literal(word) => *word,
    };
    (retval, registers, processor, memory)
}

fn fetch_b_value<'a>(operand:&BOperand, registers:&'a mut Registers, processor:&'a mut Processor, memory:&'a mut [Word]) 
    -> (Word, &'a mut Registers, &'a mut Processor, &'a mut [Word]) {
    let retval = match operand {
        BOperand::RegA => registers.reg_a,
        BOperand::RegB => registers.reg_b,
        BOperand::RegC => registers.reg_c,
        BOperand::RegX => registers.reg_x,
        BOperand::RegY => registers.reg_y,
        BOperand::RegZ => registers.reg_z,
        BOperand::RegI => registers.reg_i,
        BOperand::RegJ => registers.reg_j,
        BOperand::DerefA => {
            let reg_temp = registers.reg_a.to_usize();
            memory[reg_temp]
        },
        BOperand::DerefB => {
            let reg_temp = registers.reg_b.to_usize();
            memory[reg_temp]
        },
        BOperand::DerefC => {
            let reg_temp = registers.reg_c.to_usize();
            memory[reg_temp]
        },
        BOperand::DerefX => {
            let reg_temp = registers.reg_x.to_usize();
            memory[reg_temp]
        },
        BOperand::DerefY => {
            let reg_temp = registers.reg_y.to_usize();
            memory[reg_temp]
        },
        BOperand::DerefZ => {
            let reg_temp = registers.reg_z.to_usize();
            memory[reg_temp]
        },
        BOperand::DerefI => {
            let reg_temp = registers.reg_i.to_usize();
            memory[reg_temp]
        },
        BOperand::DerefJ => {
            let reg_temp = registers.reg_j.to_usize();
            memory[reg_temp]
        },
        BOperand::OffsetA => {
            let reg_temp = registers.reg_a.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        BOperand::OffsetB => {
            let reg_temp = registers.reg_b.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        BOperand::OffsetC => {
            let reg_temp = registers.reg_c.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        BOperand::OffsetX => {
            let reg_temp = registers.reg_x.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        BOperand::OffsetY => {
            let reg_temp = registers.reg_y.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        BOperand::OffsetZ => {
            let reg_temp = registers.reg_z.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        BOperand::OffsetI => {
            let reg_temp = registers.reg_i.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        BOperand::OffsetJ => {
            let reg_temp = registers.reg_j.to_usize();
            let pc = processor.program_counter.to_usize();
            processor.program_counter += 1;
            memory[reg_temp + pc]
        },
        BOperand::Push => {
            //Note to self: If used as a destination value, the stack pointer
            //gets updated when writing the result value.
            let stack_pointer = processor.stack_pointer.to_usize();
            memory[stack_pointer]
        },
        BOperand::Peek => memory[processor.stack_pointer.to_usize()],
        BOperand::Pick => {
            let stack_pointer = processor.stack_pointer.to_usize();
            let next_word = memory[processor.program_counter.to_usize()];
            processor.program_counter += 1;
            memory[stack_pointer + next_word.to_usize()]
        },
        BOperand::StackPointer => processor.stack_pointer,
        BOperand::ProgramCounter => processor.program_counter,
        BOperand::Excess => processor.reg_excess,
        BOperand::DerefImmediate => {
            let next_word = memory[processor.program_counter.to_usize()];
            processor.program_counter += 1;
            memory[next_word.to_usize()]
        },
        BOperand::ValueImmediate => {
            let next_word = memory[processor.program_counter.to_usize()];
            processor.program_counter += 1;
            next_word
        },
    };
    (retval, registers, processor, memory)
}
