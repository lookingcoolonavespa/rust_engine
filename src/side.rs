use crate::square::*;
use core::fmt;

const ROOK_START_SQUARES: [(Square, Square); 2] = [(A1, H1), (A8, H8)];

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Side {
    White = 0,
    Black = 1,
}

pub const SIDE_MAP: [Side; 2] = [Side::White, Side::Black];

impl Side {
    pub fn to_usize(self) -> usize {
        self as usize
    }

    pub fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn rook_start_squares(&self) -> (Square, Square) {
        ROOK_START_SQUARES[self.to_usize()]
    }

    pub fn opposite(self) -> Side {
        if self == Side::White {
            Side::Black
        } else {
            Side::White
        }
    }
}
impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = if *self == Side::White {
            "white".to_string()
        } else {
            "black".to_string()
        };
        write!(f, "{}", str)
    }
}
