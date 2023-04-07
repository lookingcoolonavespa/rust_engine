mod castle;

use crate::{
    piece_type::PromoteType,
    square::{self, Square},
};

pub trait Decode {
    fn from_sq(self) -> Square;
    fn to_sq(self) -> Square;
}

pub enum Move {
    Regular(RegularMove),
    Capture(CaptureMove),
    Castle(CastleMove),
    Promotion(PromotionMove),
    EnPassant(EnPassantMove),
}

pub struct RegularMove(u16);

impl RegularMove {
    pub fn new(from: u8, to: u8) -> RegularMove {
        RegularMove((to << 6 | from) as u16)
    }
}
impl Decode for RegularMove {
    fn from_sq(self) -> Square {
        Square::new((self.0 & 63) as square::Internal)
    }

    fn to_sq(self) -> Square {
        Square::new(((self.0 >> 6) & 63) as square::Internal)
    }
}

pub struct CaptureMove(u16);

impl CaptureMove {
    pub fn new(from: u8, to: u8, capture_piece: u8) -> CaptureMove {
        CaptureMove((capture_piece << 12 | to << 6 | from) as u16)
    }
}

impl Decode for CaptureMove {
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
    pub fn new(from: u8, to: u8, promote_piece_type: PromoteType) -> PromotionMove {
        PromotionMove((promote_piece_type.to_u8() << 12 | to << 6 | from) as u16)
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

pub struct EnPassantMove(u16);

impl EnPassantMove {
    pub fn new(from: u8, to: u8, en_passant: bool) -> EnPassantMove {
        EnPassantMove((if en_passant { 1 } else { 0 } << 15 | to << 6 | from) as u16)
    }

    pub fn is_en_passant(self) -> bool {
        if self.0 >> 15 == 1 {
            true
        } else {
            false
        }
    }
}

impl Decode for EnPassantMove {
    fn from_sq(self) -> Square {
        Square::new((self.0 & 63) as square::Internal)
    }
    fn to_sq(self) -> Square {
        Square::new(((self.0 >> 6) & 63) as square::Internal)
    }
}
