use crate::{
    bitboard::{self, squares_between::bb_squares_between, BB},
    game::Game,
    mv::{castle::Castle, Decode, EncodedMove},
    side::Side,
    square::Square,
    state::position::Position,
};

use super::{checkers_pinners_pinned, controlled_squares_with_king_gone};

pub struct LegalCheckPreprocessing {
    checkers: BB,
    pinners: BB,
    pinned: BB,
    // the bb representing the squares attacked with the king removed
    // the king is removed to ensure the the danger squares are accurate
    controlled_squares_with_king_gone_bb: BB,
}

impl LegalCheckPreprocessing {
    pub fn new(
        checkers: BB,
        pinners: BB,
        pinned: BB,
        controlled_squares_with_king_gone_bb: BB,
    ) -> LegalCheckPreprocessing {
        LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        }
    }

    pub fn from(game: &mut Game, side: Side) -> LegalCheckPreprocessing {
        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side.opposite());
        LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
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

    pub fn controlled_squares_with_king_gone_bb(&self) -> BB {
        self.controlled_squares_with_king_gone_bb
    }

    pub fn num_of_checkers(&self) -> u32 {
        self.checkers().count_ones()
    }
}

pub fn pin_direction(piece_sq: Square, king_sq: Square) -> BB {
    // piece is assumed to be pinned

    let pinned_on_diagonal = piece_sq.diagonal_mask().is_set(king_sq);
    if pinned_on_diagonal {
        return piece_sq.diagonal_mask();
    }

    let pinned_on_anti_diagonal = piece_sq.anti_diagonal_mask().is_set(king_sq);
    if pinned_on_anti_diagonal {
        return piece_sq.anti_diagonal_mask();
    }

    let pinned_on_file = piece_sq.file_mask().is_set(king_sq);
    if pinned_on_file {
        return piece_sq.file_mask();
    }

    let pinned_on_rank = piece_sq.rank_mask().is_set(king_sq);
    if pinned_on_rank {
        return piece_sq.rank_mask();
    }

    bitboard::EMPTY
}

pub fn is_en_passant_pinned_on_rank(
    position: &Position,
    defend_side: Side,
    pawn_sq: Square,
    en_passant_capture_sq: Square,
    king_sq: Square,
) -> bool {
    if !pawn_sq.rank_mask().is_set(king_sq) {
        return false;
    }

    let attack_side = defend_side.opposite();
    let (_, non_diag_attackers) = position.bb_sliders(attack_side);
    let potential_pinners = non_diag_attackers & king_sq.rook_rays();

    let occupied = position.bb_occupied();

    for sq in potential_pinners.iter() {
        let blockers = bb_squares_between(king_sq, sq) & occupied;

        if blockers.count_ones() == 2
            && blockers.is_set(pawn_sq)
            && blockers.is_set(en_passant_capture_sq)
        {
            return true;
        }
    }

    false
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

pub fn is_legal_king_move(
    mv: impl Decode,
    legal_check_preprocessing: &LegalCheckPreprocessing,
) -> bool {
    let (_, to_sq) = mv.decode_into_squares();
    let attacked_squares_bb = legal_check_preprocessing.controlled_squares_with_king_gone_bb();
    !attacked_squares_bb.is_set(to_sq)
}

pub fn is_legal_regular_move(
    position: &Position,
    from: Square,
    to: Square,
    side: Side,
    legal_check_preprocessing: &LegalCheckPreprocessing,
) -> bool {
    let num_of_checkers = legal_check_preprocessing.num_of_checkers();

    let is_pinned = legal_check_preprocessing.pinned().is_set(from);

    if num_of_checkers == 0 && !is_pinned {
        return true;
    }

    if num_of_checkers == 0 && is_pinned {
        return is_pinned_move_legal(from, to, position.king_sq(side));
    }

    if num_of_checkers == 1 {
        if is_pinned && !is_pinned_move_legal(from, to, position.king_sq(side)) {
            return false;
        }

        let checkers = legal_check_preprocessing.checkers();
        let checker_sq = checkers.bitscan();
        let squares_btwn_checker_and_king =
            checkers | bb_squares_between(checker_sq, position.king_sq(side));

        return squares_btwn_checker_and_king.is_set(to);
    }

    // num of checkers is 2
    false
}

pub fn is_legal_castle(
    position: &Position,
    castle: Castle,
    side: Side,
    attacked_squares_bb: BB,
    checkers: BB,
) -> bool {
    // assumes castle rights are set and king and rook are on home squares
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

pub fn is_legal_en_passant_move(
    position: &Position,
    from: Square,
    to: Square,
    en_passant_capture_sq: Square,
    side: Side,
    legal_check_preprocessing: &LegalCheckPreprocessing,
) -> bool {
    is_legal_regular_move(position, from, to, side, legal_check_preprocessing)
        && !is_en_passant_pinned_on_rank(
            position,
            side,
            from,
            en_passant_capture_sq,
            position.king_sq(side),
        )
}

#[cfg(test)]
pub mod test_is_pinned_move_legal {
    use core::panic;

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
                _ => {
                    panic!()
                }
            }
        }
        for illegal_mv in illegal_move_list.iter() {
            match illegal_mv {
                Move::Piece(mv) => {
                    let (from_sq, to_sq) = mv.decode_into_squares();
                    assert_eq!(is_pinned_move_legal(from_sq, to_sq, king_sq), false);
                }
                _ => {
                    panic!();
                }
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
        move_gen::{
            checkers_pinners_pinned, controlled_squares, controlled_squares_with_king_gone,
        },
        mv::EncodedMove,
        piece_type::PieceType,
    };

    use super::*;
    use crate::square::*;

    #[test]
    fn can_capture_checker() {
        let fen = "4k3/8/8/7b/8/8/4Q3/3K4 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let game = result.unwrap();
        let position = game.position();
        let side = Side::White;

        let (checkers, pinners, pinned) = checkers_pinners_pinned(position, side.opposite());
        let attacked_squares_bb = controlled_squares(position, side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb: attacked_squares_bb,
        };

        let legal_mv = EncodedMove::new(E2, H5, PieceType::Queen, true);
        let illegal_mv = EncodedMove::new(E2, D3, PieceType::Queen, false);

        let (legal_from, legal_to) = legal_mv.decode_into_squares();
        let (illegal_from, illegal_to) = illegal_mv.decode_into_squares();
        assert_eq!(
            is_legal_regular_move(
                position,
                legal_from,
                legal_to,
                side,
                &legal_check_preprocessing
            ),
            true
        );
        assert_eq!(
            is_legal_regular_move(
                position,
                illegal_from,
                illegal_to,
                side,
                &legal_check_preprocessing
            ),
            false
        );
    }

    #[test]
    fn can_block_check() {
        let fen = "4k3/8/8/7b/8/8/3Q4/3K4 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let game = result.unwrap();
        let position = game.position();
        let side = Side::White;

        let (checkers, pinners, pinned) = checkers_pinners_pinned(position, side.opposite());
        let attacked_squares_bb = controlled_squares(position, side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb: attacked_squares_bb,
        };

        let legal_mv = EncodedMove::new(D2, E2, PieceType::Queen, false);
        let illegal_mv = EncodedMove::new(D2, D3, PieceType::Queen, false);

        let (legal_from, legal_to) = legal_mv.decode_into_squares();
        let (illegal_from, illegal_to) = illegal_mv.decode_into_squares();
        assert_eq!(
            is_legal_regular_move(
                position,
                legal_from,
                legal_to,
                side,
                &legal_check_preprocessing
            ),
            true
        );
        assert_eq!(
            is_legal_regular_move(
                position,
                illegal_from,
                illegal_to,
                side,
                &legal_check_preprocessing
            ),
            false
        );
    }

    #[test]
    fn cant_block_when_there_are_two_checkers() {
        let fen = "4k3/8/8/7b/q7/8/3Q4/3K4 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let mut game = result.unwrap();
        let position = game.mut_position();
        let side = Side::White;

        let (checkers, pinners, pinned) = checkers_pinners_pinned(position, side.opposite());
        let attacked_squares_bb = controlled_squares_with_king_gone(position, side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb: attacked_squares_bb,
        };

        let legal_mv = EncodedMove::new(D1, E1, PieceType::King, false);
        let illegal_mv = EncodedMove::new(D2, E2, PieceType::Queen, false);

        let (illegal_from, illegal_to) = illegal_mv.decode_into_squares();
        assert_eq!(
            is_legal_king_move(legal_mv, &legal_check_preprocessing),
            true
        );
        assert_eq!(
            is_legal_regular_move(
                position,
                illegal_from,
                illegal_to,
                side,
                &legal_check_preprocessing
            ),
            false
        );
    }
}

#[cfg(test)]
pub mod test_is_legal_king_mv {
    use crate::{
        game::Game,
        move_gen::{checkers_pinners_pinned, controlled_squares_with_king_gone},
        mv::EncodedMove,
        square::*,
    };

    use super::*;

    #[test]
    fn king_cant_move_in_direction_of_attack_ray() {
        let fen = "3k4/8/8/7q/8/8/4K3/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let mut game = result.unwrap();
        let from = E2;
        let to = D1;
        let mv = EncodedMove::new(from, to, crate::piece_type::PieceType::King, false);
        let attack_side = game.state().side_to_move().opposite();
        let position = game.mut_position();
        let attacked_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(position, attack_side);
        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), attack_side);
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb: attacked_squares_with_king_gone_bb,
        };

        assert!(!is_legal_king_move(mv, &legal_check_preprocessing));
    }

    #[test]
    fn king_cant_capture_defended_piece_1() {
        let fen = "4k3/8/8/8/8/6p1/5p2/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let mut game = result.unwrap();
        let from = E2;
        let to = F2;
        let mv = EncodedMove::new(from, to, crate::piece_type::PieceType::King, false);
        let attack_side = game.state().side_to_move().opposite();
        let position = game.mut_position();
        let attacked_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(position, attack_side);
        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), attack_side);
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb: attacked_squares_with_king_gone_bb,
        };

        assert!(!is_legal_king_move(mv, &legal_check_preprocessing));
    }

    #[test]
    fn king_cant_capture_defended_piece_2() {
        let fen = "n1n5/PPP5/3k4/3b4/3K4/8/5p1p/5N2 b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let mut game = result.unwrap();
        let from = D4;
        let to = D5;
        let mv = EncodedMove::new(from, to, crate::piece_type::PieceType::King, false);
        let attack_side = game.state().side_to_move().opposite();
        let position = game.mut_position();
        let attacked_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(position, attack_side);
        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), attack_side);
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb: attacked_squares_with_king_gone_bb,
        };

        assert!(!is_legal_king_move(mv, &legal_check_preprocessing));
    }

    #[test]
    fn test_1() {
        let fen = "rn1qkbnr/p1pppppp/b7/1p6/5P2/P7/1PPPP1PP/RNBQKBNR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let mut game = result.unwrap();
        let from = E1;
        let to = F2;
        let mv = EncodedMove::new(from, to, crate::piece_type::PieceType::King, false);
        let attack_side = game.state().side_to_move().opposite();
        let position = game.mut_position();
        let attacked_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(position, attack_side);
        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), attack_side);
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb: attacked_squares_with_king_gone_bb,
        };

        assert!(is_legal_king_move(mv, &legal_check_preprocessing));
    }

    #[test]
    fn test_2() {
        let fen = "rnbqk1nr/pppppp1p/7b/6p1/P3P3/8/1PPP1PPP/RNBQKBNR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());

        let mut game = result.unwrap();
        let from = E1;
        let to = E2;
        let mv = EncodedMove::new(from, to, crate::piece_type::PieceType::King, false);
        let attack_side = game.state().side_to_move().opposite();
        let position = game.mut_position();
        let attacked_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(position, attack_side);
        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), attack_side);
        let legal_check_preprocessing = LegalCheckPreprocessing {
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb: attacked_squares_with_king_gone_bb,
        };

        println!("{}", attacked_squares_with_king_gone_bb);

        assert!(is_legal_king_move(mv, &legal_check_preprocessing));
    }
}

#[cfg(test)]
pub mod test_is_legal_castle {
    use super::*;
    use crate::bitboard;
    use crate::fen::STARTING_POSITION_FEN;
    use crate::game::Game;
    use crate::move_gen::controlled_squares;
    use crate::mv::castle::Castle;

    #[test]
    fn cant_castle_if_pieces_occupy_squares_1() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let attacked_squares_bb = controlled_squares(game.position(), Side::Black);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
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
        let attacked_squares_bb_b = controlled_squares(game.position(), Side::Black);
        let attacked_squares_bb_w = controlled_squares(game.position(), Side::White);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
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
        let attacked_squares_bb_b = controlled_squares(game.position(), Side::Black);
        let attacked_squares_bb_w = controlled_squares(game.position(), Side::White);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
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
        let attacked_squares_bb_b = controlled_squares(game.position(), Side::Black);
        let attacked_squares_bb_w = controlled_squares(game.position(), Side::White);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
                Side::Black,
                attacked_squares_bb_w,
                bitboard::EMPTY
            ),
            true
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::White,
                attacked_squares_bb_b,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
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
        let attacked_squares_bb = controlled_squares(game.position(), Side::Black);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            true
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
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
        let attacked_squares_bb = controlled_squares(game.position(), Side::Black);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            false
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
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
        let attacked_squares_bb = controlled_squares(game.position(), Side::Black);
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Queenside,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            true
        );
        assert_eq!(
            is_legal_castle(
                game.position(),
                Castle::Kingside,
                Side::White,
                attacked_squares_bb,
                bitboard::EMPTY
            ),
            true
        );
    }
}

#[cfg(test)]
pub mod test_is_legal_en_passant {
    use super::*;
    use crate::square::*;

    #[test]
    fn legal() {
        let fen = "4k3/8/8/4Pp2/8/8/8/4K3 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();
        let from = E5;
        let to = F6;

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        assert!(is_legal_en_passant_move(
            game.position(),
            from,
            to,
            game.state().en_passant_capture_sq().unwrap(),
            side,
            &legal_check_preprocessing
        ));
    }

    #[test]
    fn pinned_on_file() {
        let fen = "4k3/4r3/8/4Pp2/8/8/8/4K3 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();
        let from = E5;
        let to = F6;

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        assert!(!is_legal_en_passant_move(
            game.position(),
            from,
            to,
            game.state().en_passant_capture_sq().unwrap(),
            side,
            &legal_check_preprocessing
        ));
    }

    #[test]
    fn pinned_on_file_2() {
        let fen = "4k3/5r2/8/4Pp2/8/8/5K2/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();
        let from = E5;
        let to = F6;

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        assert!(is_legal_en_passant_move(
            game.position(),
            from,
            to,
            game.state().en_passant_capture_sq().unwrap(),
            side,
            &legal_check_preprocessing
        ));
    }

    #[test]
    fn pinned_on_diagonal_1() {
        let fen = "4k3/2b5/8/4Pp2/5K2/8/8/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();
        let from = E5;
        let to = F6;

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        assert!(!is_legal_en_passant_move(
            game.position(),
            from,
            to,
            game.state().en_passant_capture_sq().unwrap(),
            side,
            &legal_check_preprocessing
        ));
    }

    #[test]
    fn pinned_on_diagonal_2() {
        let fen = "4k3/3b4/8/4Pp2/6K1/8/8/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();
        let from = E5;
        let to = F6;

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        assert!(!is_legal_en_passant_move(
            game.position(),
            from,
            to,
            game.state().en_passant_capture_sq().unwrap(),
            side,
            &legal_check_preprocessing
        ));
    }

    #[test]
    fn pinned_on_rank_1() {
        let fen = "4k3/8/8/1r2PpK1/8/8/8/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();
        let from = E5;
        let to = F6;

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        assert!(!is_legal_en_passant_move(
            game.position(),
            from,
            to,
            game.state().en_passant_capture_sq().unwrap(),
            side,
            &legal_check_preprocessing
        ));
    }

    #[test]
    fn pinned_on_rank_2() {
        let fen = "4k3/8/8/3KPpr1/8/8/8/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();
        let from = E5;
        let to = F6;

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        assert!(!is_legal_en_passant_move(
            game.position(),
            from,
            to,
            game.state().en_passant_capture_sq().unwrap(),
            side,
            &legal_check_preprocessing
        ));
    }
}
