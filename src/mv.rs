pub mod castle;

use crate::{
    piece_type::PromoteType,
    square::{self, Square},
};

use self::castle::Castle;

pub trait Decode {
    fn from_sq(self) -> Square;
    fn to_sq(self) -> Square;
}

pub enum Move {
    Regular(EncodedMove),
    Capture(EncodedMove),
    Castle(CastleMove),
    Promotion(PromotionMove),
    EnPassant(EncodedMove),
}

pub struct EncodedMove(u16);

impl EncodedMove {
    pub fn new(from: u8, to: u8) -> EncodedMove {
        EncodedMove((to << 6 | from) as u16)
    }
}
impl Decode for EncodedMove {
    fn from_sq(self) -> Square {
        Square::new((self.0 & 63) as square::Internal)
    }

    fn to_sq(self) -> Square {
        Square::new(((self.0 >> 6) & 63) as square::Internal)
    }
}

pub struct CastleMove(u16);
pub const QUEEN_SIDE_CASTLE: CastleMove = CastleMove(Castle::QueenSide as u16);
pub const KING_SIDE_CASTLE: CastleMove = CastleMove(Castle::KingSide as u16);

pub struct PromotionMove(u16);

impl PromotionMove {
    pub fn new(from: u8, to: u8, promote_piece_type: &PromoteType, capture: bool) -> PromotionMove {
        PromotionMove(
            (if capture { 1 } else { 0 } << 15 | promote_piece_type.to_u8() << 12 | to << 6 | from)
                as u16,
        )
    }

    pub fn promote_piece_type(self) -> PromoteType {
        PromoteType::try_from(self.0 >> 12).unwrap()
    }
}

impl Decode for PromotionMove {
    fn from_sq(self) -> Square {
        Square::new((self.0 & 63) as square::Internal)
    }
    fn to_sq(self) -> Square {
        Square::new(((self.0 >> 6) & 63) as square::Internal)
    }
}
