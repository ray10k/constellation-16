use std::ops::{Deref, DerefMut};


#[derive(Debug,PartialEq, Eq, PartialOrd, Ord)]
pub struct Word(u16);

impl Deref for Word {
    type Target = u16;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Word {    
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    } 
}

impl From<u16> for Word {
    fn from(value: u16) -> Self {
        Word(value)
    }
}

impl Word {
    /// Casts the current `Word` to a signed representation.
    pub fn to_signed(&self) -> i16 {
        self.0 as i16
    }
}