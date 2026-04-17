use std::collections::VecDeque;
use super::word::Word;
use super::instruction::DecodedInstruction;

pub struct Processor {
    //General-purpose registers.
    pub reg_a:Word,
    pub reg_b:Word,
    pub reg_c:Word,
    pub reg_x:Word,
    pub reg_y:Word,
    pub reg_z:Word,
    pub reg_i:Word,
    pub reg_j:Word,

    //"Special" registers.
    program_counter:Word,
    stack_pointer:Word,
    reg_excess:Word,
    interrupt_address:Word,

    interrupt_queue:VecDeque<Word>,
    queue_incoming_interrupts:bool,

    current_instruction:Option<DecodedInstruction>,
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