use core::fmt;

use crate::attack_table::AttackTable;
use crate::bitboard::BB;
use crate::piece_type::{PieceType, PIECE_TYPE_MAP};
use crate::side::{self, *};
use crate::square::Square;
use crate::util::grid_to_string;

#[derive(Clone, PartialEq, Copy)]
pub struct Position {
    bb_sides: [BB; 2],
    bb_pieces: [BB; 6],
    attack_tables: AttackTable,
}
impl Position {
    pub fn new(bb_sides: [BB; 2], bb_pieces: [BB; 6]) -> Position {
        Position {
            bb_sides,
            bb_pieces,
            attack_tables: AttackTable::new(),
        }
    }

    pub fn bb_occupied(self) -> BB {
        self.bb_sides[Side::White.to_usize()] | (self.bb_sides[Side::Black.to_usize()])
    }

    pub fn bb_pieces(&self) -> [BB; 6] {
        self.bb_pieces
    }

    pub fn bb_sides(&self) -> [BB; 2] {
        self.bb_sides
    }

    pub fn bb_side(&self, side: Side) -> BB {
        self.bb_sides[side.to_usize()]
    }

    pub fn king_sq_bb(&self, side: Side) -> BB {
        self.bb_pieces[PieceType::King.to_usize()] & self.bb_sides[side.to_usize()]
    }

    pub fn king_sq(&self, side: Side) -> Square {
        (self.bb_pieces[PieceType::King.to_usize()] & self.bb_sides[side.to_usize()]).bitscan()
    }

    pub fn bb_pc(&self, piece_type: PieceType, side: Side) -> BB {
        self.bb_pieces[piece_type.to_usize()] & self.bb_sides[side.to_usize()]
    }

    pub fn bb_sliders(&self, side: Side) -> (BB, BB) {
        let queens = self.bb_pc(PieceType::Queen, side);
        let rooks = self.bb_pc(PieceType::Rook, side);
        let bishops = self.bb_pc(PieceType::Bishop, side);
        (queens | bishops, queens | rooks)
    }

    pub fn at(self, sq: Square) -> Option<(PieceType, Side)> {
        let sq_bb = BB::new(sq);
        for (i, bb) in self.bb_pieces.iter().enumerate() {
            if (*bb & sq_bb).not_empty() {
                let piece_type = PIECE_TYPE_MAP[i];
                let side = if (sq_bb & self.bb_sides[Side::White.to_usize()]).not_empty() {
                    Side::White
                } else {
                    Side::Black
                };
                return Some((piece_type, side));
            }
        }

        None
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = grid_to_string(|sq: Square| -> char {
            let result = self.at(sq);
            if result.is_none() {
                '.'
            } else {
                let (pc, side) = result.unwrap();

                return match side {
                    Side::White => pc.to_char().to_uppercase().nth(0).unwrap(),
                    Side::Black => pc.to_char(),
                };
            }
        });

        write!(f, "{}", &s)
    }
}
