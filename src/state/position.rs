use core::fmt;

use crate::attack_table::AttackTable;
use crate::bitboard::BB;
use crate::move_gen::pseudo_legal;
use crate::mv::castle::{castle_must_clear_squares, castle_pass_through_squares, Castle};
use crate::piece_type::PieceType;
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

    pub fn occupied(self) -> BB {
        self.bb_sides[WHITE.to_usize()] | (self.bb_sides[BLACK.to_usize()])
    }

    pub fn bb_pieces(&self) -> &[BB; 6] {
        &self.bb_pieces
    }

    pub fn bb_sides(&self) -> &[BB; 2] {
        &self.bb_sides
    }

    fn is_sq_attacked(self, sq: &Square, attack_side: &Side) -> bool {
        let sq_bb = BB::new(sq);
        for (i, _) in self.bb_pieces().iter().enumerate() {
            let piece_type = PieceType::try_from(i).unwrap();
            let attacks = pseudo_legal::attacks_for_piece_type(&self, &piece_type, attack_side);
            if (attacks & sq_bb).not_empty() {
                return true;
            }
        }

        false
    }

    pub fn can_castle(&self, castle: &Castle, side: &Side) -> bool {
        let occupied = self.bb_sides[0] | self.bb_sides[1];
        let must_clear_squares = castle_must_clear_squares(side, castle);
        if (occupied & must_clear_squares).not_empty() {
            return false;
        }
        for sq in castle_pass_through_squares(side, castle).iter() {
            if self.is_sq_attacked(sq, &side.other_side()) {
                return false;
            }
        }

        true
    }

    pub fn at(self, sq: Square) -> Option<(PieceType, Side)> {
        let sq_bb = BB::new(&sq);
        for (i, bb) in self.bb_pieces.iter().enumerate() {
            if (*bb & sq_bb).not_empty() {
                let piece_type = PieceType::try_from(i).unwrap();
                let side = if (sq_bb & self.bb_sides[side::WHITE.to_usize()]).not_empty() {
                    side::WHITE
                } else {
                    side::BLACK
                };
                return Some((piece_type, side));
            }
        }

        None
    }
}

#[cfg(test)]
pub mod test {
    use crate::fen::STARTING_POSITION_FEN;
    use crate::game::Game;
    use crate::mv::castle::Castle;
    use crate::{side, square::*};

    #[test]
    pub fn is_sq_attacked_1() {
        let fen = "8/8/3P2N1/1P6/pRK2p1B/P4qpk/2p4r/b6b w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert_eq!(game.position().is_sq_attacked(&F7, &side::BLACK), false);
        assert_eq!(game.position().is_sq_attacked(&G4, &side::BLACK), true);
        assert_eq!(game.position().is_sq_attacked(&A4, &side::WHITE), true);
        assert_eq!(game.position().is_sq_attacked(&B8, &side::BLACK), false);
    }

    #[test]
    pub fn cant_castle_if_pieces_occupy_squares_1() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::WHITE),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::WHITE),
            false
        );
    }

    #[test]
    pub fn cant_castle_if_pieces_occupy_squares_2() {
        let fen = "rn2k1nr/pppppppp/8/8/8/8/PPPPPPPP/RN2K1NR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::BLACK),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::BLACK),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::WHITE),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::WHITE),
            false
        );
    }

    #[test]
    pub fn cant_castle_if_pieces_occupy_squares_3() {
        let fen = "r1b1kb1r/pppppppp/8/8/8/8/PPPPPPPP/R1B1KB1R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::BLACK),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::BLACK),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::WHITE),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::WHITE),
            false
        );
    }

    #[test]
    pub fn cant_castle_if_pieces_occupy_squares_4() {
        let fen = "r2qk2r/pppppppp/8/8/8/8/PPPPPPPP/R2QK2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::BLACK),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::BLACK),
            true
        );
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::WHITE),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::WHITE),
            true
        );
    }

    #[test]
    pub fn can_castle_if_squares_are_empty() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::WHITE),
            true
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::WHITE),
            true
        );
    }

    #[test]
    pub fn cant_castle_pass_through_sq_is_attacked() {
        let fen = "r3k2r/2P3P1/8/8/8/2n3n1/8/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::WHITE),
            false
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::WHITE),
            false
        );
    }

    #[test]
    pub fn can_castle_pass_through_sq_isnt_attacked() {
        let fen = "r3k2r/P5b1/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        assert_eq!(
            game.position().can_castle(&Castle::QueenSide, &side::WHITE),
            true
        );
        assert_eq!(
            game.position().can_castle(&Castle::KingSide, &side::WHITE),
            true
        );
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
