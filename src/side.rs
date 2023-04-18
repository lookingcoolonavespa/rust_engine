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

    pub fn opposite(self) -> Side {
        if self == Side::White {
            Side::Black
        } else {
            Side::White
        }
    }
}
impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = if *self == Side::White {
            "white".to_string()
        } else {
            "black".to_string()
        };
        write!(f, "{}", str)
    }
}
