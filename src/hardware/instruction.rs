use super::word::Word;

pub struct DecodedInstruction {
    pub opcode:DcpuInstruction,
    pub operand_b:Option<BOperand>,
    pub operand_a:AOperand,
}

#[repr(u8)]
pub enum AOperand {
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

    Literal(Word) = 0x20,
}

pub enum BOperand {
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

pub enum DcpuInstruction {
    /// Invalid instruction. May crash the CPU if you like.
    Undefined = 0x40, //For instructions that weren't filled in, like 0x18
    /// sets `b` to `a`
    Set = 0x01,
    /// sets `b` to `b+a`, sets EX to `0x0001` if there's an overflow, `0x0` otherwise
    Add = 0x02,
    /// sets `b` to `b-a`, sets EX to `0xffff` if there's an underflow, `0x0` otherwise
    Sub = 0x03,
    /// sets `b` to `b*a`, sets EX to `((b*a)>>16)&0xffff` (treats `b`,`a` as unsigned)
    Mul = 0x04,
    /// sets `b` to `b*a`, sets EX to `((b*a)>>16)&0xffff` (treats `b`,`a` as signed)
    Mli = 0x05,
    /// sets `b` to `b/a`, sets EX to `((b<<16)/a)&0xffff`. if `a==0`, 
    /// sets `b` and EX to 0 instead. (treats `b`, `a` as unsigned)
    Div = 0x06,
    /// sets `b` to `b/a`, sets EX to `((b<<16)/a)&0xffff`. if `a==0`, 
    /// sets `b` and EX to 0 instead. (treats `b`, `a` as signed)
    Dvi = 0x07,
    /// sets `b` to `b%a`. if `a==0`, sets `b` to 0 instead.
    Mod = 0x08,
    /// like MOD, but treat `b`, `a` as signed. `(MDI -7, 16 == -7)`
    Mdi = 0x09,
    /// sets `b` to `b&a`
    And = 0x0a,
    /// sets `b` to `b|a`
    Bor = 0x0b,
    /// sets `b` to `b^a`
    Xor = 0x0c,
    /// sets `b` to `b>>>a`, sets EX to `((b<<16)>>a)&0xffff` (logical shift; shifts in 0s from the left.)
    Shr = 0x0d,
    /// sets `b` to `b>>a`, sets EX to `((b<<16)>>>a)&0xffff` 
    /// (arithemtic shift; shifts the original MSB in from the left.)
    Asr = 0x0e,
    /// sets `b` to `b<<a`, sets EX to `((b<<a)>>16)&0xffff`
    Shl = 0x0f,
    /// performs next instruction only if `(b&a)!=0`
    Ifb = 0x10,
    /// performs next instruction only if `(b&a)==0`
    Ifc = 0x11,
    /// performs next instruction only if `b==a`
    Ife = 0x12,
    /// performs next instruction only if `b!=a`
    Ifn = 0x13,
    /// performs next instruction only if `b>a`
    Ifg = 0x14,
    /// performs next instruction only if `b>a` (signed)
    Ifa = 0x15,
    /// performs next instruction only if `b<a`
    Ifl = 0x16,
    /// performs next instruction only if `b<a` (signed)
    Ifu = 0x17,
    /// sets `b` to `b+a+EX`, sets EX to `0x0001` if there is an overflow, `0x0` otherwise
    Adx = 0x1a,
    /// sets `b` to `b-a+EX`, sets EX to `0xFFFF` if there is an underflow, `0x0` otherwise
    Sbx = 0x1b,
    /// sets `b` to `a`, then increases I and J by 1
    Sti = 0x1e,
    /// sets `b` to `a`, then decreases I and J by 1
    Std = 0x1f,
    //Special instructions. Set the 6th bit to tell them apart.
    /// pushes the address of the next instruction to the stack, then sets PC to `a`
    Jsr = 0x01 | 0x20,
    /// triggers a software interrupt with message `a`
    Int = 0x08 | 0x20,
    /// sets `a` to IA
    Iag = 0x09 | 0x20,
    /// sets IA to `a`
    Ias = 0x0a | 0x20,
    /// disables interrupt queueing, pops A from the stack, then pops PC from the stack. 
    /// `a` operand is not used in this.
    Rfi = 0x0b | 0x20,
    /// if `a` is nonzero, interrupts will be added to the queue instead of triggered. if 
    /// `a` is zero, interrupts will be triggered as normal again.
    Iaq = 0x0c | 0x20,
    /// sets `a` to number of connected hardware devices
    Hwn = 0x10 | 0x20,
    /// sets A, B, C, X, Y registers to information about hardware `a`.
    /// `A+(B<<16)` is a 32 bit word identifying the hardware id,
    /// `C` is the hardware version,
    /// `X+(Y<<16)` is a 32 bit word identifying the manufacturer.
    Hwq = 0x11 | 0x20,
    /// sends an interrupt to hardware `a`.
    Hwi = 0x12 | 0x20
}