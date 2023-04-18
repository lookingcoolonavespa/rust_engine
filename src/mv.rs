pub mod castle;

use crate::{
    bitboard::BB,
    piece_type::{self, PieceType, PromoteType, PIECE_TYPE_MAP, PROMOTE_TYPE_MAP},
    square::{self, Square},
};

use self::castle::Castle;

pub trait Decode {
    fn decode_into_squares(self) -> (Square, Square);
    fn decode_into_bb(self) -> (BB, BB);
}

pub enum Move {
    Regular(EncodedMove),
    Capture(EncodedMove),
    Castle(CastleMove),
    Promotion(PromotionMove),
    PromotionCapture(PromotionMove),
    EnPassant(EncodedMove),
}

pub struct EncodedMove(u16);

impl EncodedMove {
    pub fn new(from: u8, to: u8, piece_type: PieceType) -> EncodedMove {
        EncodedMove((piece_type.to_u8() << 12 | to << 6 | from) as u16)
    }

    pub fn piece_type(&self) -> PieceType {
        PIECE_TYPE_MAP[(self.0 >> 12) as usize]
    }
}
impl Decode for EncodedMove {
    fn decode_into_bb(self) -> (BB, BB) {
        (BB((self.0 & 63) as u64), BB((self.0 as u64 >> 6) & 63))
    }

    fn decode_into_squares(self) -> (Square, Square) {
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
    pub fn new(from: u8, to: u8, promote_piece_type: &PromoteType) -> PromotionMove {
        PromotionMove((promote_piece_type.to_u8() << 12 | to << 6 | from) as u16)
    }

    pub fn promote_piece_type(self) -> PromoteType {
        PROMOTE_TYPE_MAP[(self.0 >> 12) as usize].unwrap()
    }
}

impl Decode for PromotionMove {
    fn decode_into_bb(self) -> (BB, BB) {
        (BB((self.0 & 63) as u64), BB((self.0 as u64 >> 6) & 63))
    }

    fn decode_into_squares(self) -> (Square, Square) {
        (
            Square::new((self.0 & 63) as square::Internal),
            Square::new(((self.0 >> 6) & 63) as square::Internal),
        )
    }
}
