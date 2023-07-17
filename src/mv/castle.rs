use std::fmt;

use crate::{bitboard::BB, side::Side, square::*};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Castle {
    Queenside = 0,
    Kingside = 1,
}

pub const ROOK_START_SQUARES: [(Square, Square); 2] = [(A1, H1), (A8, H8)];
const CASTLE_KING_MOVES: [[(Square, Square); 2]; 2] = [[(E1, C1), (E8, C8)], [(E1, G1), (E8, G8)]];
const CASTLE_ROOK_MOVES: [[(Square, Square); 2]; 2] = [[(A1, D1), (A8, D8)], [(H1, F1), (H8, F8)]];
const CASTLE_PASS_THROUGH_SQUARES: [[BB; 2]; 2] = [
    [BB(12), BB(864691128455135232)],
    [BB(96), BB(6917529027641081856)],
];
const CASTLE_MUST_CLEAR_SQUARES: [[BB; 2]; 2] = [
    [BB(14), BB(1008806316530991104)],
    [BB(96), BB(6917529027641081856)],
];

impl Castle {
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn to_usize(self) -> usize {
        self as usize
    }

    pub fn king_squares(&self, side: Side) -> (Square, Square) {
        // returns the king squares in a tuple eg. (from, to)
        CASTLE_KING_MOVES[self.to_usize()][side.to_usize()]
    }

    pub fn rook_squares(&self, side: Side) -> (Square, Square) {
        // returns the rook squares in a tuple eg. (from, to)
        CASTLE_ROOK_MOVES[self.to_usize()][side.to_usize()]
    }

    pub fn must_clear_squares(&self, side: Side) -> BB {
        CASTLE_MUST_CLEAR_SQUARES[self.to_usize()][side.to_usize()]
    }
    pub fn pass_through_squares(&self, side: Side) -> BB {
        CASTLE_PASS_THROUGH_SQUARES[self.to_usize()][side.to_usize()]
    }
}

impl fmt::Display for Castle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Castle::Queenside => "0-0-0",
            Castle::Kingside => "0-0",
        };
        write!(f, "{}", str)
    }
}
