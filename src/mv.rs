pub mod castle;

use crate::{
    bitboard::BB,
    piece_type::{PromoteType, PROMOTE_TYPE_MAP},
    square::{self, Square},
};

use self::castle::Castle;

pub trait Decode {
    fn decode_into_squares(&self) -> (Square, Square);
    fn decode_into_bb(&self) -> (BB, BB);
}

pub enum Move {
    King(EncodedMove),
    Piece(EncodedMove),
    Castle(CastleMove),
    Promotion(PromotionMove),
    EnPassant(EncodedMove),
}

pub struct EncodedMove(u16);

impl EncodedMove {
    pub fn new(from: Square, to: Square, capture: bool) -> EncodedMove {
        EncodedMove(
            ((if capture { 1u16 } else { 0u16 }) << 15 | to.to_u16() << 6 | from.to_u16()) as u16,
        )
    }
}
impl Decode for EncodedMove {
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

pub struct CastleMove(Castle);
pub const QUEEN_SIDE_CASTLE: CastleMove = CastleMove(Castle::QueenSide);
pub const KING_SIDE_CASTLE: CastleMove = CastleMove(Castle::KingSide);

impl CastleMove {
    pub fn decode(&self) -> Castle {
        self.0
    }
}

pub struct PromotionMove(u16);

impl PromotionMove {
    pub fn new(
        from: Square,
        to: Square,
        promote_piece_type: &PromoteType,
        capture: bool,
    ) -> PromotionMove {
        PromotionMove(
            ((if capture { 1 } else { 0 }) << 15
                | promote_piece_type.to_u16() << 12
                | to.to_u16() << 6
                | from.to_u16()) as u16,
        )
    }

    pub fn promote_piece_type(self) -> PromoteType {
        PROMOTE_TYPE_MAP[(self.0 >> 12) as usize].unwrap()
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
