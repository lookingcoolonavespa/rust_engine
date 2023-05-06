pub mod castle;

use std::fmt;

use crate::{
    bitboard::BB,
    piece_type::{PieceType, PromoteType, PIECE_TYPE_MAP},
    square::{self, Square},
};

use self::castle::Castle;

pub trait Decode {
    fn decode_into_squares(&self) -> (Square, Square);
    fn decode_into_bb(&self) -> (BB, BB);
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Move {
    King(EncodedMove),
    Rook(EncodedMove),
    Pawn(EncodedMove),
    DoublePawnPush(EncodedMove),
    Piece(EncodedMove),
    Castle(Castle),
    Promotion(PromotionMove),
    EnPassant(EncodedMove),
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Move::King(encoded_mv)
            | Move::Rook(encoded_mv)
            | Move::Pawn(encoded_mv)
            | Move::Piece(encoded_mv)
            | Move::DoublePawnPush(encoded_mv)
            | Move::EnPassant(encoded_mv) => {
                let (from, to) = encoded_mv.decode_into_squares();
                write!(f, "{}{}", from, to)
            }

            Move::Castle(castle_mv) => {
                write!(f, "{}", castle_mv)
            }

            Move::Promotion(promotion_mv) => {
                write!(f, "{}", promotion_mv)
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct EncodedMove(u16);

impl EncodedMove {
    pub fn new(from: Square, to: Square, piece_type: PieceType, capture: bool) -> EncodedMove {
        EncodedMove(
            (if capture { 1u16 } else { 0u16 }) << 15
                | piece_type.to_u16() << 12
                | to.to_u16() << 6
                | from.to_u16(),
        )
    }

    pub fn piece_type(&self) -> PieceType {
        PIECE_TYPE_MAP[((self.0 >> 12) & 7) as usize]
    }

    pub fn is_capture(&self) -> bool {
        self.0 >> 15 == 1
    }
}
impl Decode for EncodedMove {
    fn decode_into_bb(&self) -> (BB, BB) {
        (
            BB::new(Square::new((self.0 & 63) as usize)),
            BB::new(Square::new((self.0 as square::Internal >> 6) & 63)),
        )
    }

    fn decode_into_squares(&self) -> (Square, Square) {
        (
            Square::new((self.0 & 63) as square::Internal),
            Square::new(((self.0 >> 6) & 63) as square::Internal),
        )
    }
}

impl fmt::Display for EncodedMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (from, to) = self.decode_into_squares();
        write!(f, "{}{}", from, to)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct PromotionMove(u16);

impl PromotionMove {
    pub fn new(
        from: Square,
        to: Square,
        promote_piece_type: &PromoteType,
        capture: bool,
    ) -> PromotionMove {
        PromotionMove(
            (if capture { 1 } else { 0 }) << 15
                | promote_piece_type.to_u16() << 12
                | to.to_u16() << 6
                | from.to_u16(),
        )
    }

    pub fn promote_piece_type(self) -> PieceType {
        PIECE_TYPE_MAP[((self.0 >> 12) & 7) as usize]
    }

    pub fn is_capture(&self) -> bool {
        self.0 >> 15 == 1
    }
}

impl Decode for PromotionMove {
    fn decode_into_bb(&self) -> (BB, BB) {
        (BB((self.0 & 63) as u64), BB((self.0 as u64 >> 6) & 63))
    }

    fn decode_into_squares(&self) -> (Square, Square) {
        (
            Square::new((self.0 & 63) as square::Internal),
            Square::new(((self.0 >> 6) & 63) as square::Internal),
        )
    }
}

impl fmt::Display for PromotionMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (from, to) = self.decode_into_squares();
        let promote_pc = self.promote_piece_type();
        write!(f, "{}{}={}", from, to, promote_pc)
    }
}
