use crate::{
    bitboard::{self, squares_between::bb_squares_between, BB, KING_MOVES, PAWN_CAPTURES},
    mv::castle,
    piece_type::PieceType,
    side::Side,
    square::Square,
    state::position::Position,
};

mod legal_check_preprocessing;
mod parallel;
mod pawn;
pub mod pseudo_legal;
mod slider;

fn is_sq_attacked(position: &Position, sq: &Square, attack_side: Side) -> bool {
    let occupied = position.bb_occupied();

    if (sq.knight_jumps() & position.bb_pc(PieceType::Knight, attack_side)).not_empty() {
        return true;
    }

    let (diag_attackers, non_diag_attackers) = position.bb_sliders(attack_side);
    let potential_slider_attackers =
        (diag_attackers & sq.bishop_rays()) | (non_diag_attackers & sq.rook_rays());
    for potential_attack_sq in potential_slider_attackers.iter() {
        let blockers = occupied & bb_squares_between(&potential_attack_sq, sq);
        if blockers.empty() {
            return true;
        }
    }

    let possible_pawn_squares = PAWN_CAPTURES[attack_side.opposite().to_usize()][sq.to_usize()];
    if (position.bb_pc(PieceType::Pawn, attack_side) & possible_pawn_squares).not_empty() {
        return true;
    }

    let possible_king_squares = KING_MOVES[sq.to_usize()];
    let enemy_king_square = position.bb_pc(PieceType::King, attack_side);
    if (enemy_king_square & possible_king_squares).not_empty() {
        return true;
    }

    false
}

pub fn attacks(position: &Position, side: Side) -> BB {
    let knight_jumps = parallel::knight_jumps(position.bb_pc(PieceType::Knight, side));
    let pawn_attacks = parallel::pawn_attacks(position.bb_pc(PieceType::Pawn, side), side);
    let (diag_attackers, non_diag_attackers) = position.bb_sliders(side);
    let occupied = position.bb_occupied();
    let diagonal_attacks = parallel::diagonal_attacks(diag_attackers, occupied);
    let file_rank_attacks = parallel::file_rank_attacks(non_diag_attackers, occupied);
    let king_attacks = pseudo_legal::king_attacks(position.king_sq(side), position.bb_side(side));

    (king_attacks | pawn_attacks | knight_jumps | diagonal_attacks | file_rank_attacks)
        ^ position.bb_side(side)
}

pub fn checkers_pinners_pinned(position: &Position, attack_side: Side) -> (BB, BB, BB) {
    // check for checks by knight and pawn first bc they dont care about the position of other
    // pieces
    let defend_side = attack_side.opposite();
    let king_sq = position.king_sq(defend_side);

    let mut checkers = bitboard::EMPTY;
    let mut pinned = bitboard::EMPTY;
    let mut pinners = bitboard::EMPTY;

    let possible_pawn_checkers = PAWN_CAPTURES[defend_side.to_usize()][king_sq.to_usize()];
    checkers |= possible_pawn_checkers & position.bb_pc(PieceType::Pawn, attack_side);

    checkers |= king_sq.knight_jumps() & position.bb_pc(PieceType::Knight, attack_side);

    // deal with slider pieces
    let occupied = position.bb_occupied();
    let (diag_attackers, non_diag_attackers) = position.bb_sliders(attack_side);
    let potential_checkers =
        (diag_attackers & king_sq.bishop_rays()) | (non_diag_attackers & king_sq.rook_rays());
    for sq in potential_checkers.iter() {
        let bb_squares_between = bb_squares_between(&king_sq, &sq);
        let blockers = bb_squares_between & occupied;

        if blockers.empty() {
            checkers |= BB::new(sq);
        } else if blockers.count_ones() == 1
            && (blockers & position.bb_side(defend_side)).not_empty()
        {
            pinned |= blockers;
            pinners |= BB::new(sq);
        }
    }

    (checkers, pinners, pinned)
}

#[cfg(test)]
pub mod test_checkers_pinners {
    use super::*;
    use crate::game::Game;
    use crate::square::*;

    #[test]
    fn pawn_check() {
        let fen = "4k3/8/8/8/3p1p2/4K3/8/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let position = game.position();

        let (checkers, _, _) =
            checkers_pinners_pinned(position, game.state().side_to_move().opposite());

        assert_eq!(checkers.count_ones(), 2);
        let mut bb_iter = checkers.iter();
        assert_eq!(bb_iter.next().unwrap(), D4);
        assert_eq!(bb_iter.next().unwrap(), F4);
    }

    #[test]
    fn single_slider_check() {
        let fen = "4k3/8/4q3/8/8/4K3/8/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let position = game.position();

        let (checkers, _, _) =
            checkers_pinners_pinned(position, game.state().side_to_move().opposite());

        assert_eq!(checkers.count_ones(), 1);
        assert_eq!(checkers.bitscan(), E6);
    }

    #[test]
    fn double_slider_check() {
        let fen = "4k3/8/4q3/6b1/8/4K3/8/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let position = game.position();

        let (checkers, _, _) =
            checkers_pinners_pinned(position, game.state().side_to_move().opposite());

        assert_eq!(checkers.count_ones(), 2);
        let mut bb_iter = checkers.iter();
        assert_eq!(bb_iter.next().unwrap(), G5);
        assert_eq!(bb_iter.next().unwrap(), E6);
    }

    #[test]
    fn pinner_pinned_1() {
        let fen = "4k3/8/8/6b1/5R2/4K3/8/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let position = game.position();

        let (_, pinners, pinned) =
            checkers_pinners_pinned(position, game.state().side_to_move().opposite());

        assert_eq!(pinners.bitscan(), G5);
        assert_eq!(pinners.count_ones(), 1);

        assert_eq!(pinned.count_ones(), 1);
        assert_eq!(pinned.bitscan(), F4);
    }
}

// #[cfg(test)]
// pub mod test_can_castle {
//     use super::*;
//     use crate::fen::STARTING_POSITION_FEN;
//     use crate::game::Game;
//     use crate::mv::castle::Castle;
//     use crate::{side, square::*};
//
//     #[test]
//     fn is_sq_attacked_1() {
//         let fen = "8/8/3P2N1/1P6/pRK2p1B/P4qpk/2p4r/b6b w - - 0 1";
//         let result = Game::from_fen(fen);
//         assert!(result.is_ok());
//         let game = result.unwrap();
//         let position = game.position();
//
//         assert_eq!(is_sq_attacked(position, &F7, side::BLACK), false);
//         assert_eq!(is_sq_attacked(position, &G4, side::BLACK), true);
//         assert_eq!(is_sq_attacked(position, &A4, Side::White), true);
//         assert_eq!(is_sq_attacked(position, &B8, side::BLACK), false);
//     }
//
//     #[test]
//     fn cant_castle_if_pieces_occupy_squares_1() {
//         let fen = STARTING_POSITION_FEN;
//         let result = Game::from_fen(fen);
//         assert!(result.is_ok());
//         let game = result.unwrap();
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::WHITE),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::WHITE),
//             false
//         );
//     }
//
//     #[test]
//     fn cant_castle_if_pieces_occupy_squares_2() {
//         let fen = "rn2k1nr/pppppppp/8/8/8/8/PPPPPPPP/RN2K1NR w KQkq - 0 1";
//         let result = Game::from_fen(fen);
//         assert!(result.is_ok());
//         let game = result.unwrap();
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::BLACK),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::BLACK),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::WHITE),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::WHITE),
//             false
//         );
//     }
//
//     #[test]
//     fn cant_castle_if_pieces_occupy_squares_3() {
//         let fen = "r1b1kb1r/pppppppp/8/8/8/8/PPPPPPPP/R1B1KB1R w KQkq - 0 1";
//         let result = Game::from_fen(fen);
//         assert!(result.is_ok());
//         let game = result.unwrap();
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::BLACK),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::BLACK),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::WHITE),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::WHITE),
//             false
//         );
//     }
//
//     #[test]
//     fn cant_castle_if_pieces_occupy_squares_4() {
//         let fen = "r2qk2r/pppppppp/8/8/8/8/PPPPPPPP/R2QK2R w KQkq - 0 1";
//         let result = Game::from_fen(fen);
//         assert!(result.is_ok());
//         let game = result.unwrap();
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::BLACK),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::BLACK),
//             true
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::WHITE),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::WHITE),
//             true
//         );
//     }
//
//     #[test]
//     fn can_castle_if_squares_are_empty() {
//         let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
//         let result = Game::from_fen(fen);
//         assert!(result.is_ok());
//         let game = result.unwrap();
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::WHITE),
//             true
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::WHITE),
//             true
//         );
//     }
//
//     #[test]
//     fn cant_castle_pass_through_sq_is_attacked() {
//         let fen = "r3k2r/2P3P1/8/8/8/2n3n1/8/R3K2R w KQkq - 0 1";
//         let result = Game::from_fen(fen);
//         assert!(result.is_ok());
//         let game = result.unwrap();
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::WHITE),
//             false
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::WHITE),
//             false
//         );
//     }
//
//     #[test]
//     fn can_castle_pass_through_sq_isnt_attacked() {
//         let fen = "r3k2r/P5b1/8/8/8/8/8/R3K2R w KQkq - 0 1";
//         let result = Game::from_fen(fen);
//         assert!(result.is_ok());
//         let game = result.unwrap();
//         assert_eq!(
//             can_castle(game.position(), Castle::QueenSide, side::WHITE),
//             true
//         );
//         assert_eq!(
//             can_castle(game.position(), Castle::KingSide, side::WHITE),
//             true
//         );
//     }
// }
