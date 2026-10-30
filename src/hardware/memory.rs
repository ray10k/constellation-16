use std::cell::RefCell;
use std::rc::Rc;

use crate::hardware::word::Word;

pub type Memory = Rc<RefCell<Box<[Word]>>>;

pub(crate) trait MemOp {
    /// Duplicate the memory-reference for later use. Wrapper for `rc::clone()`.
    fn dup(&self) -> Self;
    /// Initialize a new Memory struct, with 65535 Words all set to 0.
    fn new_empty() -> Self;
    /// Initialize a new Memory struct. The first words will be set to match what is
    /// in `initial`, with the remainder filled up with 0's. Again, 65535 Words total.
    fn new_preinit(initial: &[Word]) -> Self;
}

impl MemOp for Memory {
    fn dup(&self) -> Self {
        Rc::clone(self)
    }

    fn new_empty() -> Self {
        let memory = [Word(0); 0xffff];
        Rc::new(RefCell::new(Box::from(memory)))
    }

    fn new_preinit(initial: &[Word]) -> Self {
        let mut memory = [Word(0); 0xffff];
        for (index, value) in initial.iter().enumerate() {
            memory[index] = *value;
        }
        Rc::new(RefCell::new(Box::from(memory)))
    }
}
