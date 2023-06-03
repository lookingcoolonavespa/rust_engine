use std::fmt;

use subenum::subenum;

pub const PAWN_ID: isize = 0;
pub const KNIGHT_ID: isize = 1;
pub const BISHOP_ID: isize = 2;
pub const ROOK_ID: isize = 3;
pub const QUEEN_ID: isize = 4;
pub const KING_ID: isize = 5;

#[subenum(PromoteType)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PieceType {
    Pawn = PAWN_ID,
    #[subenum(PromoteType)]
    Knight = KNIGHT_ID,
    #[subenum(PromoteType)]
    Bishop = BISHOP_ID,
    #[subenum(PromoteType)]
    Rook = ROOK_ID,
    #[subenum(PromoteType)]
    Queen = QUEEN_ID,
    King = KING_ID,
}

pub const PIECE_TYPE_MAP: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King,
];

pub const PIECE_TYPE_SCORE_MAP: [u32; 6] = [100, 300, 350, 500, 900, 10000];

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

    pub fn score(self) -> u32 {
        PIECE_TYPE_SCORE_MAP[self.to_usize()]
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
