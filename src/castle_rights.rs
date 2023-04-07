use std::ops::{BitAnd, BitOr};

#[derive(PartialEq, Clone, Copy)]
pub struct CastleRights(u8);

pub const NONE: CastleRights = CastleRights::new(0);
pub const KINGSIDE: CastleRights = CastleRights::new(0b1010);
pub const QUEENSIDE: CastleRights = CastleRights::new(0b0101);
pub const WHITE: CastleRights = CastleRights::new(0b1100);
pub const BLACK: CastleRights = CastleRights::new(0b0011);

impl CastleRights {
    pub fn new(u8: u8) -> CastleRights {
        CastleRights(u8)
    }

    pub fn set(self, rights: CastleRights) -> CastleRights {
        CastleRights(self.0 | rights.0)
    }
}

impl BitOr for CastleRights {
    type Output = CastleRights;

    fn bitor(self, other: CastleRights) -> CastleRights {
        CastleRights(self.0 | other.0)
    }
}
impl BitAnd for CastleRights {
    type Output = CastleRights;

    fn bitand(self, other: CastleRights) -> CastleRights {
        CastleRights(self.0 & other.0)
    }
}
