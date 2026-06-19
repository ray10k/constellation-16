use std::ops::{AddAssign, BitAnd, Deref, DerefMut};


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

impl From<Word> for u16 {
    fn from(value: Word) -> Self {
        value.0
    }
}

impl BitAnd<u16> for Word {
    type Output=Self;

    fn bitand(self, rhs: u16) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl AddAssign<u16> for Word {
    fn add_assign(&mut self, rhs: u16) {
        self.0 += rhs
    }
}

impl Default for Word {
    fn default() -> Self {
        Self(0)
    }
}

impl Word {
    /// Casts the current `Word` to a signed representation.
    pub fn to_signed(&self) -> i16 {
        self.0 as i16
    }

    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}