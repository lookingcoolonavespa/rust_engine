use crate::{
    piece_type::{PieceType, PIECE_TYPE_MAP},
    side::Side,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece(u8);

impl Piece {
    pub fn new(side: Side, piece_type: PieceType) -> Piece {
        Piece(side.to_u8() << 3 | piece_type.to_u8())
    }

    pub fn piece_type(&self) -> PieceType {
        PIECE_TYPE_MAP[(self.0 & 7) as usize]
    }

    pub fn side(&self) -> Side {
        if self.0 >> 3 == 1 {
            Side::Black
        } else {
            Side::White
        }
    }

    pub fn decode(&self) -> (Side, PieceType) {
        (self.side(), self.piece_type())
    }
}
