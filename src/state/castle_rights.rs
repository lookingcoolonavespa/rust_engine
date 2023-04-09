use core::fmt;
use std::ops::{BitAnd, BitOr};

use crate::side::{self, Side};

#[derive(Debug, PartialEq, Clone)]
pub struct CastleRights(u8);

pub const NONE: CastleRights = CastleRights::new(0);
pub const KINGSIDE: CastleRights = CastleRights::new(0b1010);
pub const QUEENSIDE: CastleRights = CastleRights::new(0b0101);
pub const WHITE: CastleRights = CastleRights::new(0b1100);
pub const BLACK: CastleRights = CastleRights::new(0b0011);

impl CastleRights {
    pub const fn new(u8: u8) -> CastleRights {
        CastleRights(u8)
    }

    pub fn set(self, rights: CastleRights) -> CastleRights {
        CastleRights(self.0 | rights.0)
    }

    pub fn can(&self, side: &Side, rights: &CastleRights) -> bool {
        let side_rights = if *side == side::WHITE { WHITE } else { BLACK };
        let mut rights = rights.0 & side_rights.0;

        (self.0 & rights) != 0
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    pub fn set1() {
        let cr = NONE.set(KINGSIDE);
        let expected = "Kk";

        assert_eq!(cr.to_string(), expected);
    }

    #[test]
    pub fn set2() {
        let cr = NONE.set(KINGSIDE & WHITE);
        let expected = "K";

        assert_eq!(cr.to_string(), expected);
    }

    #[test]
    pub fn set3() {
        let cr = NONE.set(KINGSIDE & WHITE);
        let cr = cr.set(QUEENSIDE & WHITE);
        let expected = "KQ";

        assert_eq!(cr.to_string(), expected);
    }

    #[test]
    pub fn can1() {
        let cr = NONE.set(KINGSIDE);

        assert_eq!(cr.can(&side::WHITE, &KINGSIDE), true);
    }

    #[test]
    pub fn can2() {
        let cr = NONE.set(KINGSIDE);

        assert_eq!(cr.can(&side::WHITE, &QUEENSIDE), false);
    }
    #[test]
    pub fn can3() {
        let cr = NONE.set(WHITE);

        assert_eq!(cr.can(&side::WHITE, &QUEENSIDE), true);
    }
    #[test]
    pub fn can4() {
        let cr = NONE.set(BLACK);

        assert_eq!(cr.can(&side::BLACK, &KINGSIDE), true);
    }
    #[test]
    pub fn can5() {
        let cr = NONE.set(BLACK);

        assert_eq!(cr.can(&side::WHITE, &KINGSIDE), false);
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

impl fmt::Display for CastleRights {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        if self.0 == 0 {
            str += "-"
        } else {
            if (self.0 & (KINGSIDE & WHITE).0) != 0 {
                str += "K"
            }
            if (self.0 & (QUEENSIDE & WHITE).0) != 0 {
                str += "Q"
            }
            if (self.0 & (KINGSIDE & BLACK).0) != 0 {
                str += "k"
            }
            if (self.0 & (QUEENSIDE & BLACK).0) != 0 {
                str += "q"
            }
        }
        write!(f, "{}", str)
    }
}

#[cfg(test)]
pub mod test_display {
    use super::*;

    #[test]
    pub fn none() {
        let fmt_str = NONE.to_string();
        let expected = "-";

        assert_eq!(fmt_str, expected);
    }
    #[test]
    pub fn kingside() {
        let fmt_str = KINGSIDE.to_string();
        let expected = "Kk";

        assert_eq!(fmt_str, expected);
    }
    #[test]
    pub fn queenside() {
        let fmt_str = QUEENSIDE.to_string();
        let expected = "Qq";

        assert_eq!(fmt_str, expected);
    }
    #[test]
    pub fn white() {
        let fmt_str = WHITE.to_string();
        let expected = "KQ";

        assert_eq!(fmt_str, expected);
    }
    #[test]
    pub fn black() {
        let fmt_str = BLACK.to_string();
        let expected = "kq";

        assert_eq!(fmt_str, expected);
    }
}
