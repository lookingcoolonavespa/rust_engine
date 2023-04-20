pub mod castle;

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

#[derive(Clone, Copy)]
pub enum Move {
    King(EncodedMove),
    Piece(EncodedMove),
    Castle(Castle),
    Promotion(PromotionMove),
    EnPassant(EncodedMove),
}

#[derive(Clone, Copy)]
pub struct EncodedMove(u16);

impl EncodedMove {
    pub fn new(from: Square, to: Square, piece_type: PieceType, capture: bool) -> EncodedMove {
        EncodedMove(
            ((if capture { 1u16 } else { 0u16 }) << 15
                | piece_type.to_u16() << 12
                | to.to_u16() << 6
                | from.to_u16()) as u16,
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
        (BB((self.0 & 63) as u64), BB((self.0 as u64 >> 6) & 63))
    }

    fn decode_into_squares(&self) -> (Square, Square) {
        (
            Square::new((self.0 & 63) as square::Internal),
            Square::new(((self.0 >> 6) & 63) as square::Internal),
        )
    }
}

#[derive(Clone, Copy)]
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
