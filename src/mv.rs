pub mod castle;

use std::fmt;

use crate::{
    bitboard::BB,
    piece_type::{PieceType, PromoteType, PIECE_TYPE_MAP},
    square::{self, Square},
};

use self::castle::Castle;

pub trait Decode {
    fn decode_into_squares(&self) -> (Square, Square);
    fn decode_into_bb(&self) -> (BB, BB);
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Move {
    King(EncodedMove),
    Rook(EncodedMove),
    Pawn(EncodedMove),
    DoublePawnPush(EncodedMove),
    Piece(EncodedMove),
    Castle(Castle),
    Promotion(PromotionMove),
    EnPassant(EncodedMove),
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Move::King(mv)
            | Move::Rook(mv)
            | Move::Pawn(mv)
            | Move::DoublePawnPush(mv)
            | Move::Piece(mv)
            | Move::EnPassant(mv) => write!(f, "{}", mv),
            Move::Castle(castle_mv) => write!(f, "{}", castle_mv),
            Move::Promotion(promote_mv) => write!(f, "{}", promote_mv),
        }
    }
}

impl Move {
    pub fn to_algebra(self) -> String {
        match self {
            Move::King(mv)
            | Move::Rook(mv)
            | Move::Pawn(mv)
            | Move::DoublePawnPush(mv)
            | Move::Piece(mv)
            | Move::EnPassant(mv) => mv.to_algebra(),
            Move::Castle(castle_mv) => castle_mv.to_string(),
            Move::Promotion(promote_mv) => promote_mv.to_algebra(),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct EncodedMove(u16);

impl EncodedMove {
    pub fn new(from: Square, to: Square, piece_type: PieceType, capture: bool) -> EncodedMove {
        EncodedMove(
            (if capture { 1u16 } else { 0u16 }) << 15
                | piece_type.to_u16() << 12
                | to.to_u16() << 6
                | from.to_u16(),
        )
    }

    pub fn piece_type(&self) -> PieceType {
        PIECE_TYPE_MAP[((self.0 >> 12) & 7) as usize]
    }

    pub fn is_capture(&self) -> bool {
        self.0 >> 15 == 1
    }

    fn to_algebra(&self) -> String {
        let piece_type = self.piece_type();
        let capture = self.is_capture();
        let (_, to) = self.decode_into_squares();
        let mut algebra = piece_type.to_algebra();

        if capture {
            algebra += "x";
        }

        format!("{algebra}{to}")
    }
}
impl Decode for EncodedMove {
    fn decode_into_bb(&self) -> (BB, BB) {
        (
            BB::new(Square::new((self.0 & 63) as usize)),
            BB::new(Square::new((self.0 as square::Internal >> 6) & 63)),
        )
    }

    fn decode_into_squares(&self) -> (Square, Square) {
        (
            Square::new((self.0 & 63) as square::Internal),
            Square::new(((self.0 >> 6) & 63) as square::Internal),
        )
    }
}

impl fmt::Display for EncodedMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (from, to) = self.decode_into_squares();
        write!(f, "{}{}", from, to)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct PromotionMove(u16);

impl PromotionMove {
    pub fn new(
        from: Square,
        to: Square,
        promote_piece_type: &PromoteType,
        capture: bool,
    ) -> PromotionMove {
        PromotionMove(
            (if capture { 1 } else { 0 }) << 15
                | promote_piece_type.to_u16() << 12
                | to.to_u16() << 6
                | from.to_u16(),
        )
    }

    pub fn promote_piece_type(self) -> PieceType {
        PIECE_TYPE_MAP[((self.0 >> 12) & 7) as usize]
    }

    pub fn is_capture(&self) -> bool {
        self.0 >> 15 == 1
    }

    fn to_algebra(&self) -> String {
        let capture = self.is_capture();
        let (from, to) = self.decode_into_squares();
        let promote = self.promote_piece_type().to_algebra();
        let mut algebra = String::new();

        if capture {
            let from_file = from.to_string().chars().next().unwrap();
            algebra.push(from_file);
            algebra.push('x');
        }

        format!("{}{}={}", algebra, to, promote)
    }
}

impl Decode for PromotionMove {
    fn decode_into_bb(&self) -> (BB, BB) {
        (BB((self.0 & 63) as u64), BB((self.0 as u64 >> 6) & 63))
    }

    fn decode_into_squares(&self) -> (Square, Square) {
        (
            Square::new((self.0 & 63) as square::Internal),
            Square::new(((self.0 >> 6) & 63) as square::Internal),
        )
    }
}

impl fmt::Display for PromotionMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (from, to) = self.decode_into_squares();
        let promote_pc = self.promote_piece_type();
        write!(f, "{}{}{}", from, to, promote_pc)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_algebra() {
        let tests: Vec<(Move, &str)> = vec![
            (
                Move::Piece(EncodedMove::new(
                    square::A1,
                    square::A8,
                    PieceType::Rook,
                    false,
                )),
                "Ra8",
            ),
            (
                Move::Piece(EncodedMove::new(
                    square::E2,
                    square::E4,
                    PieceType::Bishop,
                    true,
                )),
                "Bxe4",
            ),
            (
                Move::Promotion(PromotionMove::new(
                    square::E7,
                    square::F8,
                    &PromoteType::Queen,
                    true,
                )),
                "exf8=Q",
            ),
            (
                Move::Promotion(PromotionMove::new(
                    square::E7,
                    square::E8,
                    &PromoteType::Queen,
                    false,
                )),
                "e8=Q",
            ),
            (Move::Castle(Castle::Queenside), "0-0-0"),
            (
                Move::Pawn(EncodedMove::new(
                    square::E2,
                    square::E4,
                    PieceType::Pawn,
                    false,
                )),
                "e4",
            ),
        ];

        for (mv, expected) in tests {
            assert_eq!(mv.to_algebra(), expected)
        }
    }
}
