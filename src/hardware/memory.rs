use std::cell::RefCell;
use std::rc::Rc;

use crate::hardware::word::Word;

pub type Memory = Rc<RefCell<Box<[Word]>>>;

pub(crate) trait MemOp {
    fn dup(&self) -> Self;
    fn new_empty() -> Self;
    fn new_preinit(initial:&[Word]) -> Self;
}

impl MemOp for Memory{
    fn dup(&self) -> Self {
        Rc::clone(self)
    }

    fn new_empty() -> Self {
        let memory = [Word(0); 0xffff];
        Rc::new(RefCell::new(Box::from(memory)))
    }

    fn new_preinit(initial:&[Word]) -> Self {
        let mut memory = [Word(0); 0xffff];
        for (index, value) in initial.iter().enumerate() {
            memory[index] = *value;
        }
        Rc::new(RefCell::new(Box::from(memory)))
    }
}