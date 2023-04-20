use crate::{
    bitboard::{squares_between::bb_squares_between, BB},
    mv::{castle::Castle, Decode},
    side::Side,
    square::Square,
    state::position::Position,
};

pub struct LegalCheckPreprocessing {
    checkers: BB,
    pinners: BB,
    pinned: BB,
    attacked_squares_bb: BB,
}

impl LegalCheckPreprocessing {
    pub fn new(
        checkers: BB,
        pinners: BB,
        pinned: BB,
        attacked_squares_bb: BB,
    ) -> LegalCheckPreprocessing {
        LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            attacked_squares_bb,
        }
    }
    pub fn pinners(&self) -> BB {
        self.pinners
    }

    pub fn pinned(&self) -> BB {
        self.pinned
    }

    pub fn checkers(&self) -> BB {
        self.checkers
    }

    pub fn attacked_squares_bb(&self) -> BB {
        self.attacked_squares_bb
    }

    pub fn num_of_checkers(&self) -> u32 {
        self.checkers().count_ones()
    }
}

fn is_pinned_move_legal(from_sq: Square, to_sq: Square, king_sq: Square) -> bool {
    // piece is assumed to be pinned

    let pinned_on_diagonal = from_sq.diagonal_mask().is_set(king_sq);
    if pinned_on_diagonal {
        return from_sq.diagonal_mask().is_set(to_sq);
    }

    let pinned_on_anti_diagonal = from_sq.anti_diagonal_mask().is_set(king_sq);
    if pinned_on_anti_diagonal {
        return from_sq.anti_diagonal_mask().is_set(to_sq);
    }

    let pinned_on_file = from_sq.file_mask().is_set(king_sq);
    if pinned_on_file {
        return from_sq.file_mask().is_set(to_sq);
    }

    let pinned_on_rank = from_sq.rank_mask().is_set(king_sq);
    if pinned_on_rank {
        return from_sq.rank_mask().is_set(to_sq);
    }

    false
}

pub fn is_legal_regular_move(
    position: &Position,
    mv: impl Decode,
    is_king: bool,
    side: Side,
    legal_check_preprocessing: &LegalCheckPreprocessing,
) -> bool {
    let (from_sq, to_sq) = mv.decode_into_squares();
    let attacked_squares_bb = legal_check_preprocessing.attacked_squares_bb();

    let num_of_checkers = legal_check_preprocessing.num_of_checkers();

    match is_king {
        true => return !attacked_squares_bb.is_set(to_sq),
        false => {
            let pinned = legal_check_preprocessing.pinned();
            let is_pinned = pinned.is_set(from_sq);

            if num_of_checkers == 0 && !is_pinned {
                return true;
            }

            if num_of_checkers == 0 && is_pinned {
                return is_pinned_move_legal(from_sq, to_sq, position.king_sq(side));
            }

            if num_of_checkers == 1 {
                let checkers = legal_check_preprocessing.checkers();
                let checker_sq = checkers.bitscan();
                let squares_btwn_checker_and_king =
                    checkers | bb_squares_between(checker_sq, position.king_sq(side));

                return squares_btwn_checker_and_king.is_set(to_sq);
            }

            // num of checkers is 2
            false
        }
    }
}

pub fn is_legal_castle(
    position: &Position,
    castle: Castle,
    side: Side,
    attacked_squares_bb: BB,
    checkers: BB,
) -> bool {
    if checkers.not_empty() {
        return false;
    }
    let occupied = position.bb_occupied();
    let must_clear_squares = castle.must_clear_squares(side);
    if (occupied & must_clear_squares).not_empty() {
        return false;
    }

    (castle.pass_through_squares(side) & attacked_squares_bb).empty()
}

#[cfg(test)]
pub mod test_is_pinned_move_legal {
    use crate::{
        game::Game,
        move_gen::slider::{
            anti_diagonal_moves_from_sq, diagonal_moves_from_sq, horizontal_moves_from_sq,
            vertical_moves_from_sq,
        },
        move_list::MoveList,
        mv::{EncodedMove, Move},
        piece_type::PieceType,
        side::Side,
        square::*,
    };

    use super::*;

    #[test]
    fn can_capture_pinner() {
        let fen = "4k3/8/8/7b/8/8/4Q3/3K4 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let game = result.unwrap();
        let side = Side::White;
        let from = E2;

        assert_eq!(
            is_pinned_move_legal(from, H5, game.position().king_sq(side)),
            true
        );
    }
    #[test]
    fn pinned_on_file() {
        let fen = "4k3/4r3/8/8/8/8/4Q3/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let game = result.unwrap();
        let occupied = game.position().bb_occupied();
        let side = Side::White;
        let friendly_occupied = game.position().bb_side(side);
        let from = E2;

        let legal_moves_bb = vertical_moves_from_sq(from, occupied) & !friendly_occupied;
        let illegal_moves_bb = horizontal_moves_from_sq(from, occupied) & !friendly_occupied;

        let mut legal_move_list = MoveList::new();
        let mut illegal_move_list = MoveList::new();

        let enemy_occupied = game.position().bb_side(side.opposite());
        let cb = |from: Square, to: Square| -> Move {
            if enemy_occupied.is_set(to) {
                Move::Piece(EncodedMove::new(from, to, PieceType::Queen, true))
            } else {
                Move::Piece(EncodedMove::new(from, to, PieceType::Queen, false))
            }
        };
        legal_move_list.insert_moves(from, legal_moves_bb, cb);
        illegal_move_list.insert_moves(from, illegal_moves_bb, cb);

        let king_sq = game.position().king_sq(side);
        for legal_mv in legal_move_list.iter() {
            match legal_mv {
                Move::Piece(mv) => {
                    let (from_sq, to_sq) = mv.decode_into_squares();
                    assert_eq!(is_pinned_move_legal(from_sq, to_sq, king_sq), true);
                }
                _ => {}
            }
        }
        for illegal_mv in illegal_move_list.iter() {
            match illegal_mv {
                Move::Piece(mv) => {
                    let (from_sq, to_sq) = mv.decode_into_squares();
                    assert_eq!(is_pinned_move_legal(from_sq, to_sq, king_sq), false);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn pinned_on_diagonal() {
        let fen = "4k3/8/8/7b/8/8/4Q3/3K4 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let game = result.unwrap();
        let occupied = game.position().bb_occupied();
        let side = Side::White;
        let friendly_occupied = game.position().bb_side(side);
        let from = E2;

        let legal_moves_bb = anti_diagonal_moves_from_sq(from, occupied) & !friendly_occupied;
        let illegal_moves_bb = (vertical_moves_from_sq(from, occupied)
            | diagonal_moves_from_sq(from, occupied)
            | horizontal_moves_from_sq(from, occupied))
            & !friendly_occupied;

        let mut legal_move_list = MoveList::new();
        let mut illegal_move_list = MoveList::new();

        let enemy_occupied = game.position().bb_side(side.opposite());
        let cb = |from: Square, to: Square| -> Move {
            if enemy_occupied.is_set(to) {
                Move::Piece(EncodedMove::new(from, to, PieceType::Queen, true))
            } else {
                Move::Piece(EncodedMove::new(from, to, PieceType::Queen, false))
            }
        };
        legal_move_list.insert_moves(from, legal_moves_bb, cb);
        illegal_move_list.insert_moves(from, illegal_moves_bb, cb);

        let king_sq = game.position().king_sq(side);
        for legal_mv in legal_move_list.iter() {
            match legal_mv {
                Move::Piece(mv) => {
                    let (from_sq, to_sq) = mv.decode_into_squares();
                    assert_eq!(is_pinned_move_legal(from_sq, to_sq, king_sq), true);
                }
                _ => {}
            }
        }
        for illegal_mv in illegal_move_list.iter() {
            match illegal_mv {
                Move::Piece(mv) => {
                    let (from_sq, to_sq) = mv.decode_into_squares();
                    assert_eq!(is_pinned_move_legal(from_sq, to_sq, king_sq), false);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
pub mod test_is_legal_regular_or_capture_move {
    use crate::{
        game::Game,
        move_gen::{attacks, checkers_pinners_pinned},
        mv::EncodedMove,
        piece_type::PieceType,
    };

    use super::*;
    use crate::square::*;

    #[test]
    fn no_1() {
        let fen = "4k3/8/8/7b/8/8/4Q3/3K4 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let game = result.unwrap();
        let position = game.position();
        let side = Side::White;

        let (checkers, pinners, pinned) = checkers_pinners_pinned(position, side.opposite());
        let attacked_squares_bb = attacks(position, side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            attacked_squares_bb,
        };

        let legal_mv = EncodedMove::new(E2, H5, PieceType::Queen, true);
        let illegal_mv = EncodedMove::new(E2, D3, PieceType::Queen, false);
        assert_eq!(
            is_legal_regular_move(position, legal_mv, false, side, &legal_check_preprocessing),
            true
        );
        assert_eq!(
            is_legal_regular_move(
                position,
                illegal_mv,
                false,
                side,
                &legal_check_preprocessing
            ),
            false
        );
    }

    #[test]
    fn no_2() {
        let fen = "4k3/8/8/7b/8/8/3Q4/3K4 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let game = result.unwrap();
        let position = game.position();
        let side = Side::White;

        let (checkers, pinners, pinned) = checkers_pinners_pinned(position, side.opposite());
        let attacked_squares_bb = attacks(position, side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            attacked_squares_bb,
        };

        let legal_mv = EncodedMove::new(D2, E2, PieceType::Queen, false);
        let illegal_mv = EncodedMove::new(D2, D3, PieceType::Queen, false);
        assert_eq!(
            is_legal_regular_move(position, legal_mv, false, side, &legal_check_preprocessing),
            true
        );
        assert_eq!(
            is_legal_regular_move(
                position,
                illegal_mv,
                false,
                side,
                &legal_check_preprocessing
            ),
            false
        );
    }

    #[test]
    fn no_3() {
        let fen = "4k3/8/8/7b/q7/8/3Q4/3K4 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let game = result.unwrap();
        let position = game.position();
        let side = Side::White;

        let (checkers, pinners, pinned) = checkers_pinners_pinned(position, side.opposite());
        let attacked_squares_bb = attacks(position, side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            attacked_squares_bb,
        };

        let legal_mv = EncodedMove::new(D1, E1, PieceType::King, false);
        let illegal_mv = EncodedMove::new(D2, E2, PieceType::Queen, false);
        assert_eq!(
            is_legal_regular_move(position, legal_mv, true, side, &legal_check_preprocessing),
            true
        );
        assert_eq!(
            is_legal_regular_move(
                position,
                illegal_mv,
                false,
                side,
                &legal_check_preprocessing
            ),
            false
        );
    }
}

#[cfg(test)]
pub mod test_is_legal_castle {
    use super::*;
    use crate::bitboard;
    use crate::fen::STARTING_POSITION_FEN;
    use crate::game::Game;
    use crate::move_gen::attacks;
    use crate::mv::castle::Castle;

    #[test]
    fn cant_castle_if_pieces_occupy_squares_1() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let attacked_squares_bb = attacks(game.position(), Side::Black);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            false
        );
    }

    #[test]
    fn cant_castle_if_pieces_occupy_squares_2() {
        let fen = "rn2k1nr/pppppppp/8/8/8/8/PPPPPPPP/RN2K1NR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let attacked_squares_bb_b = attacks(game.position(), Side::Black);
        let attacked_squares_bb_w = attacks(game.position(), Side::White);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            false
        );
    }

    #[test]
    fn cant_castle_if_pieces_occupy_squares_3() {
        let fen = "r1b1kb1r/pppppppp/8/8/8/8/PPPPPPPP/R1B1KB1R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let attacked_squares_bb_b = attacks(game.position(), Side::Black);
        let attacked_squares_bb_w = attacks(game.position(), Side::White);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            false
        );
    }

    #[test]
    fn cant_castle_if_pieces_occupy_squares_4() {
        let fen = "r2qk2r/pppppppp/8/8/8/8/PPPPPPPP/R2QK2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let attacked_squares_bb_b = attacks(game.position(), Side::Black);
        let attacked_squares_bb_w = attacks(game.position(), Side::White);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            true
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            true
        );
    }

    #[test]
    fn can_castle_if_squares_are_empty() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let attacked_squares_bb = attacks(game.position(), Side::Black);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            true
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            true
        );
    }

    #[test]
    fn cant_castle_pass_through_sq_is_attacked() {
        let fen = "r3k2r/2P3P1/8/8/8/2n3n1/8/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let attacked_squares_bb = attacks(game.position(), Side::Black);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            false
        );
    }

    #[test]
    fn can_castle_pass_through_sq_isnt_attacked() {
        let fen = "r3k2r/P5b1/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let attacked_squares_bb = attacks(game.position(), Side::Black);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::QueenSide,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            true
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::KingSide,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            true
        );
    }
}
