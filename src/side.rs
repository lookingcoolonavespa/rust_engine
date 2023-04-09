use core::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Side {
    White = 0,
    Black = 1,
}

impl Side {
    pub fn to_usize(self) -> usize {
        self as usize
    }

    pub fn other_side(self) -> Side {
        if self == WHITE {
            BLACK
        } else {
            WHITE
        }
    }
}
impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = if *self == WHITE {
            "white".to_string()
        } else {
            "black".to_string()
        };
        write!(f, "{}", str)
    }
}

pub const WHITE: Side = Side::White;
pub const BLACK: Side = Side::Black;
