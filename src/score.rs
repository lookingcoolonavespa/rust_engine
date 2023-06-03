use std::fmt;

use crate::phase::Phase;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Score(pub i32, pub i32, pub i32);

impl Score {
    pub fn get(self, phase: Phase) -> i32 {
        match phase {
            Phase::Opening => self.0,
            Phase::Middle => self.1,
            Phase::End => self.2,
        }
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Opening: {}, Middle: {}, End: {}",
            self.0, self.1, self.2
        )
    }
}
