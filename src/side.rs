use core::fmt;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
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

pub const WHITE: Side = Side::White;
pub const BLACK: Side = Side::Black;

pub const SIDE_MAP: HashMap<&str, Side> = HashMap::from([("w", Side::White), ("b", Side::Black)]);

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {}
}
