use std::collections::VecDeque;

type word = u16;

pub struct Processor {
    //General-purpose registers.
    pub reg_a:word,
    pub reg_b:word,
    pub reg_c:word,
    pub reg_x:word,
    pub reg_y:word,
    pub reg_z:word,
    pub reg_i:word,
    pub reg_j:word,

    //"Special" registers.
    program_counter:word,
    stack_pointer:word,
    reg_excess:word,
    interrupt_address:word,

    interrupt_queue:VecDeque<word>,
    queue_incoming_interrupts:bool,

    current_instruction:Option<DecodedInstruction>,
    instruction_state:InstructionState,
}

struct DecodedInstruction {
    pub opcode:DcpuInstruction,
    pub operand_b:Option<BOperand>,
    pub operand_a:AOperand,
}

#[repr(u8)]
enum AOperand {
    RegA = 0x00,
    RegB = 0x01,
    RegC = 0x02,
    RegX = 0x03,
    RegY = 0x04,
    RegZ = 0x05,
    RegI = 0x06,
    RegJ = 0x07,
    
    DerefA = 0x08,
    DerefB = 0x09,
    DerefC = 0x0A,
    DerefX = 0x0B,
    DerefY = 0x0C,
    DerefZ = 0x0D,
    DerefI = 0x0E,
    DerefJ = 0x0F,

    OffsetA = 0x10,
    OffsetB = 0x11,
    OffsetC = 0x12,
    OffsetX = 0x13,
    OffsetY = 0x14,
    OffsetZ = 0x15,
    OffsetI = 0x16,
    OffsetJ = 0x17,

    Pop = 0x18,
    Peek = 0x19,
    Pick = 0x1A,

    StackPointer = 0x1B,
    ProgramCounter = 0x1C,
    Excess = 0x1D,

    DerefImmediate = 0x1E,
    ValueImmediate = 0x1F,

    Literal(word) = 0x20,
}

enum BOperand {
    RegA = 0x00,
    RegB = 0x01,
    RegC = 0x02,
    RegX = 0x03,
    RegY = 0x04,
    RegZ = 0x05,
    RegI = 0x06,
    RegJ = 0x07,
    
    DerefA = 0x08,
    DerefB = 0x09,
    DerefC = 0x0A,
    DerefX = 0x0B,
    DerefY = 0x0C,
    DerefZ = 0x0D,
    DerefI = 0x0E,
    DerefJ = 0x0F,

    OffsetA = 0x10,
    OffsetB = 0x11,
    OffsetC = 0x12,
    OffsetX = 0x13,
    OffsetY = 0x14,
    OffsetZ = 0x15,
    OffsetI = 0x16,
    OffsetJ = 0x17,

    Push = 0x18,
    Peek = 0x19,
    Pick = 0x1A,

    StackPointer = 0x1B,
    ProgramCounter = 0x1C,
    Excess = 0x1D,

    DerefImmediate = 0x1E,
    ValueImmediate = 0x1F,
}

enum DcpuInstruction {
    Undefined, //For instructions that weren't filled in, like 0x18
    Set,
    Add,
    Subtract,
    Multiply,
    SignedMultiply,
    Divide,
    SignedDivide,
    Modulo,
    SignedModulo,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LogicShiftRight,
    ArithmeticShiftRight,
    ShiftLeft,
    ConditionOverlap,
    ConditionExclusion,
    ConditionEqual,
    ConditionNotEqual,
    ConditionGreater,
    ConditionSignedGreater,
    ConditionSmaller,
    ConditionSignedSmaller,
    AddWithOverflow,
    SubtractWithOverflow,
    SetThenIncrement,
    SetThenDecrement,
    //Special instructions
    JumpSubroutine,
    Interrupt,
    InterruptAddressGet,
    InterruptAddressSet,
    ReturnFromInterrupt,
    InterruptQueueing,
    HardwareCount,
    HardwareQuery,
    HardwareInterrupt
}

#[derive(Default)]
enum InstructionState {
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