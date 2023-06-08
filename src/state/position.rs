use core::fmt;

use crate::bitboard::{self, BB};
use crate::move_gen::is_sq_attacked;
use crate::phase::Phase;
use crate::piece::Piece;
use crate::piece_type::PieceType;
use crate::psqt::PSQT;
use crate::side::*;
use crate::square::Square;
use crate::util::grid_to_string;

pub type Board = [Option<Piece>; 64];
pub type Scores = [i32; 2];

#[derive(Clone, PartialEq, Copy)]
pub struct Position {
    bb_sides: [BB; 2],
    bb_pieces: [BB; 6],
    board: [Option<Piece>; 64],
    piece_score: Scores,
    sq_score: Scores,
    phase: Phase,
}
impl Position {
    pub fn new(bb_sides: [BB; 2], bb_pieces: [BB; 6], board: Board, phase: Phase) -> Position {
        let (piece_score, sq_score) = Position::calc_score(board, phase);
        Position {
            sq_score,
            piece_score,
            bb_sides,
            bb_pieces,
            board,
            phase,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
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

    pub fn piece_score(&self, side: Side) -> i32 {
        self.piece_score[side.to_usize()]
    }

    pub fn sq_score(&self, side: Side) -> i32 {
        self.sq_score[side.to_usize()]
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

        debug_assert_eq!(
            self.bb_sides[side.to_usize()] & from_bb,
            crate::bitboard::EMPTY
        );

        self.board[from.to_usize()] = None;

        self.piece_score[side.to_usize()] -= piece_type.score() as i32;
        self.sq_score[side.to_usize()] -=
            PSQT[side.to_usize()][piece_type.to_usize()][from.to_usize()].get(self.phase);
    }

    pub fn remove_at(&mut self, sq: Square) -> Option<Piece> {
        let result = self.board[sq.to_usize()];
        debug_assert!(
            result.is_some(),
            "unable to remove at {} because there is nothing there",
            sq
        );
        let pc = result.unwrap();
        self.remove_piece(pc.piece_type(), sq, pc.side());

        result
    }

    pub fn place_piece(&mut self, piece_type: PieceType, to: Square, side: Side) {
        let to_bb = BB::new(to);

        self.bb_pieces[piece_type.to_usize()] |= to_bb;
        self.bb_sides[side.to_usize()] |= to_bb;

        debug_assert_ne!(self.bb_sides[side.to_usize()] & to_bb, bitboard::EMPTY);

        self.board[to.to_usize()] = Some(Piece::new(side, piece_type));

        self.piece_score[side.to_usize()] += piece_type.score() as i32;
        self.sq_score[side.to_usize()] +=
            PSQT[side.to_usize()][piece_type.to_usize()][to.to_usize()].get(self.phase);
    }

    pub fn move_piece(&mut self, piece_type: PieceType, from: Square, to: Square, side: Side) {
        self.remove_piece(piece_type, from, side);
        self.place_piece(piece_type, to, side);
    }

    pub fn in_check(&self, side: Side) -> bool {
        let king_sq = self.king_sq(side);
        is_sq_attacked(self, king_sq, side.opposite())
    }

    pub fn insufficient_material(&self) -> bool {
        // king versus king
        // king and bishop versus king
        // king and knight versus king
        // king and bishop versus king and bishop with the bishops on the same color

        if self.bb_pieces[PieceType::Pawn.to_usize()].not_empty() {
            return false;
        }

        if (self.bb_pieces[PieceType::Rook.to_usize()]
            | self.bb_pieces[PieceType::Queen.to_usize()])
        .not_empty()
        {
            return false;
        }

        let pieces_left_count = self.bb_occupied().count_ones();
        // if there are 3 or less pieces on the board and no rooks, no queens, no pawns,
        // means it is king+bishop vs king or king+knight/king or
        // king vs king
        if pieces_left_count < 4 {
            return true;
        }

        let bishops_bb = self.bb_pieces[PieceType::Bishop.to_usize()];
        if pieces_left_count == 4
            && bishops_bb.count_ones() == 2
            && self.bb_pc(PieceType::Bishop, Side::White).count_ones() == 1
        {
            let first_bishop_is_light = bishops_bb.bitscan().is_light_sq();
            let second_bishop_bb = bishops_bb ^ bishops_bb.lsb();
            let second_bishop_is_light = second_bishop_bb.bitscan().is_light_sq();

            return first_bishop_is_light == second_bishop_is_light;
        }

        false
    }

    pub fn calc_score(board: Board, phase: Phase) -> (Scores, Scores) {
        let mut piece_score = [0; 2];
        let mut sq_score = [0; 2];
        for (sq, piece_result) in board.iter().enumerate() {
            if let Some(piece) = piece_result {
                let side = piece.side();
                let piece_type = piece.piece_type();

                piece_score[side.to_usize()] += piece_type.score() as i32;
                sq_score[side.to_usize()] +=
                    PSQT[side.to_usize()][piece_type.to_usize()][sq as usize].get(phase);
            }
        }

        (piece_score, sq_score)
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
