use subenum::subenum;

#[subenum(PromoteType)]
#[derive(Copy, Clone)]
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

pub const PROMOTE_TYPE_ARR: [PromoteType; 4] = [
    PromoteType::Knight,
    PromoteType::Bishop,
    PromoteType::Rook,
    PromoteType::Queen,
];
const PIECE_CHARS: [char; 6] = ['p', 'n', 'b', 'r', 'q', 'k'];

impl PieceType {
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

impl PromoteType {
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn to_usize(self) -> usize {
        self as usize
    }
}

impl TryFrom<usize> for PieceType {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(PieceType::Pawn),
            1 => Ok(PieceType::Knight),
            2 => Ok(PieceType::Bishop),
            3 => Ok(PieceType::Rook),
            4 => Ok(PieceType::Queen),
            5 => Ok(PieceType::King),
            _ => Err(()),
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
impl TryFrom<u16> for PromoteType {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(PromoteType::Knight),
            2 => Ok(PromoteType::Bishop),
            3 => Ok(PromoteType::Rook),
            4 => Ok(PromoteType::Queen),
            _ => Err(()),
        }
    }
}
