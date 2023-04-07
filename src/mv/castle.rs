use crate::{side::Side, square::*};

pub enum Castle {
    QueenSide = 0,
    KingSide = 1,
}

const CASTLE_KING_MOVES: [[(Square, Square); 2]; 2] = [[(E1, C1), (E8, C8)], [(E1, G1), (E8, G8)]];
const CASTLE_ROOK_MOVES: [[(Square, Square); 2]; 2] = [[(A1, D1), (A8, D8)], [(H1, F1), (H8, F8)]];

pub fn castle_king_squares(side: Side, castle: Castle) -> (Square, Square) {
    CASTLE_KING_MOVES[castle.to_usize()][side.to_usize()]
}

pub fn castle_rook_squares(side: Side, castle: Castle) -> (Square, Square) {
    CASTLE_ROOK_MOVES[castle.to_usize()][side.to_usize()]
}

impl Castle {
    pub fn to_u16(self) -> u16 {
        self as u16
    }
    pub fn to_usize(self) -> usize {
        self as usize
    }
}
