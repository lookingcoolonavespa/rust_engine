use core::fmt;

use crate::bitboard::BB;
use crate::move_gen::is_sq_attacked;
use crate::piece::Piece;
use crate::piece_type::PieceType;
use crate::side::*;
use crate::square::Square;
use crate::util::grid_to_string;

#[derive(Clone, PartialEq, Copy)]
pub struct Position {
    bb_sides: [BB; 2],
    bb_pieces: [BB; 6],
    board: [Option<Piece>; 64],
}
impl Position {
    pub fn new(bb_sides: [BB; 2], bb_pieces: [BB; 6], board: [Option<Piece>; 64]) -> Position {
        Position {
            bb_sides,
            bb_pieces,
            board,
        }
    }

    pub fn bb_occupied(&self) -> BB {
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

    pub fn at(&self, sq: Square) -> Option<Piece> {
        self.board[sq.to_usize()]
    }

    pub fn remove_piece(&mut self, piece_type: PieceType, from: Square, side: Side) {
        let from_bb = BB::new(from);

        self.bb_pieces[piece_type.to_usize()] ^= from_bb;
        self.bb_sides[side.to_usize()] ^= from_bb;

        self.board[from.to_usize()] = None;
    }

    pub fn remove_at(&mut self, sq: Square) -> Option<Piece> {
        let result = self.board[sq.to_usize()];
        if let Some(pc) = result {
            self.remove_piece(pc.piece_type(), sq, pc.side());
        }

        result
    }

    pub fn place_piece(&mut self, piece_type: PieceType, to: Square, side: Side) {
        let to_bb = BB::new(to);

        self.bb_pieces[piece_type.to_usize()] |= to_bb;
        self.bb_sides[side.to_usize()] |= to_bb;

        self.board[to.to_usize()] = Some(Piece::new(side, piece_type));
    }

    pub fn move_piece(&mut self, piece_type: PieceType, from: Square, to: Square, side: Side) {
        self.remove_piece(piece_type, from, side);
        self.place_piece(piece_type, to, side);
    }

    pub fn in_check(&self, side: Side) -> bool {
        let king_sq = self.king_sq(side);
        is_sq_attacked(self, king_sq, side.opposite())
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = grid_to_string(|sq: Square| -> char {
            let result = self.at(sq);
            if let Some(_) = result {
                let piece = result.unwrap();
                let (side, pc) = piece.decode();

                match side {
                    Side::White => pc.to_char().to_uppercase().next().unwrap(),
                    Side::Black => pc.to_char(),
                }
            } else {
                '.'
            }
        });

        write!(f, "{}", &s)
    }
}
