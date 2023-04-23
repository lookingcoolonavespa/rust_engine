use std::fmt;

use subenum::subenum;

#[subenum(PromoteType)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PieceType {
    Pawn = 0,
    #[subenum(PromoteType)]
    Knight = 1,
    #[subenum(PromoteType)]
    Bishop = 2,
    #[subenum(PromoteType)]
    Rook = 3,
    #[subenum(PromoteType)]
    Queen = 4,
    King = 5,
}

pub const PIECE_TYPE_MAP: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King,
];

pub const PROMOTE_TYPE_ARR: [PromoteType; 4] = [
    PromoteType::Knight,
    PromoteType::Bishop,
    PromoteType::Rook,
    PromoteType::Queen,
];
const PIECE_CHARS: [char; 6] = ['p', 'n', 'b', 'r', 'q', 'k'];

impl PieceType {
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn to_usize(self) -> usize {
        self as usize
    }

    pub fn to_char(self) -> char {
        PIECE_CHARS[self.to_usize()]
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl PromoteType {
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn to_usize(self) -> usize {
        self as usize
    }
}

impl TryFrom<char> for PromoteType {
    type Error = &'static str;

    fn try_from(v: char) -> Result<Self, Self::Error> {
        match v {
            'n' => Ok(PromoteType::Knight),
            'b' => Ok(PromoteType::Bishop),
            'r' => Ok(PromoteType::Rook),
            'q' => Ok(PromoteType::Queen),
            _ => Err("not a valid piece"),
        }
    }
}

impl TryFrom<char> for PieceType {
    type Error = &'static str;

    fn try_from(v: char) -> Result<Self, Self::Error> {
        match v {
            'p' => Ok(PieceType::Pawn),
            'n' => Ok(PieceType::Knight),
            'b' => Ok(PieceType::Bishop),
            'r' => Ok(PieceType::Rook),
            'q' => Ok(PieceType::Queen),
            'k' => Ok(PieceType::King),
            _ => Err("not a valid piece"),
        }
    }
}
