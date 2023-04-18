use std::ops::{Shl, Shr};

use crate::{
    bitboard::{self, BB, KING_MOVES, KNIGHT_JUMPS, PAWN_CAPTURES, PAWN_PUSHES},
    piece_type::PieceType,
    side::Side,
    square::Square,
    state::position::Position,
};

use super::slider::*;

pub fn attacks_for_piece_type(position: &Position, piece_type: &PieceType, side: Side) -> BB {
    let friendly_occupied = position.bb_sides()[side.to_usize()];
    let enemy_occupied = position.bb_sides()[side.opposite().to_usize()];
    let pieces_bb =
        position.bb_pieces()[piece_type.to_usize()] & position.bb_sides()[side.to_usize()];

    match piece_type {
        PieceType::Pawn => {
            let mut attacks = bitboard::EMPTY;

            for from in pieces_bb.iter() {
                attacks |= pawn_attacks(from, side)
            }

            attacks
        }
        PieceType::Knight => {
            let mut attacks = bitboard::EMPTY;

            for from in pieces_bb.iter() {
                attacks |= knight_attacks(from, friendly_occupied)
            }

            attacks
        }
        PieceType::Bishop => {
            let mut attacks = bitboard::EMPTY;

            for from in pieces_bb.iter() {
                attacks |= bishop_attacks(from, friendly_occupied, enemy_occupied)
            }

            attacks
        }
        PieceType::Rook => {
            let mut attacks = bitboard::EMPTY;

            for from in pieces_bb.iter() {
                attacks |= rook_attacks(from, friendly_occupied, enemy_occupied)
            }

            attacks
        }
        PieceType::Queen => {
            let mut attacks = bitboard::EMPTY;

            for from in pieces_bb.iter() {
                attacks |= queen_attacks(from, friendly_occupied, enemy_occupied)
            }

            attacks
        }
        PieceType::King => {
            let mut attacks = bitboard::EMPTY;

            for from in pieces_bb.iter() {
                attacks |= king_attacks(from, enemy_occupied)
            }

            attacks
        }
    }
}

pub fn bishop_attacks(from: Square, friendly_occupied: BB, enemy_occupied: BB) -> BB {
    let occupied = friendly_occupied | enemy_occupied;

    (diagonal_moves_from_sq(from, occupied) | anti_diagonal_moves_from_sq(from, occupied))
        & !friendly_occupied
}

pub fn rook_attacks(from: Square, friendly_occupied: BB, enemy_occupied: BB) -> BB {
    let occupied = friendly_occupied | enemy_occupied;

    (vertical_moves_from_sq(from, occupied) | horizontal_moves_from_sq(from, occupied))
        & !friendly_occupied
}

pub fn queen_attacks(from: Square, friendly_occupied: BB, enemy_occupied: BB) -> BB {
    let occupied = friendly_occupied | enemy_occupied;
    ((vertical_moves_from_sq(from, occupied) | horizontal_moves_from_sq(from, occupied))
        | (diagonal_moves_from_sq(from, occupied) | anti_diagonal_moves_from_sq(from, occupied)))
        & !friendly_occupied
}

pub fn knight_attacks(from: Square, friendly_occupied: BB) -> BB {
    KNIGHT_JUMPS[from.to_usize()] & !friendly_occupied
}

pub fn king_attacks(from: Square, friendly_occupied: BB) -> BB {
    KING_MOVES[from.to_usize()] & !friendly_occupied
}

const PAWN_HOME_RANK: [usize; 2] = [1, 6];
pub fn pawn(
    from: Square,
    friendly_occupied: BB,
    enemy_occupied: BB,
    en_passant: Option<Square>,
    color: Side,
) -> BB {
    let mut pushes =
        PAWN_PUSHES[color.to_usize()][from.to_usize()] & !(friendly_occupied | enemy_occupied);

    if from.rank() == PAWN_HOME_RANK[color.to_usize()] && pushes.not_empty() {
        // if pawn is on home rank and is not blocked
        let double_push_bb = match color {
            Side::White => pushes.shl(8),
            Side::Black => pushes.shr(8),
        };
        pushes |= double_push_bb;
    }
    let en_passant_bb: BB = match en_passant {
        Some(sq) => BB::new(sq),
        None => bitboard::EMPTY,
    };

    (pushes & !(friendly_occupied | enemy_occupied))
        | (pawn_attacks(from, color) & (enemy_occupied | en_passant_bb))
}

fn pawn_attacks(from: Square, color: Side) -> BB {
    PAWN_CAPTURES[color.to_usize()][from.to_usize()]
}

#[cfg(test)]
pub mod queen_tests {
    use super::*;
    use crate::{bitboard, square, util::get_bb_from_array_of_squares};

    #[test]
    pub fn empty_board() {
        let from = square::E4;
        let attacks = queen_attacks(from, BB::new(from), bitboard::EMPTY);
        print!("{}", attacks);

        let expected = (from.rank_mask()
            | from.file_mask()
            | from.diagonal_mask()
            | from.anti_diagonal_mask())
            ^ BB::new(from);

        assert_eq!(attacks, expected);
    }

    #[test]
    pub fn occupied_board_enemy() {
        let from = square::E4;
        let enemy_occupied = get_bb_from_array_of_squares(&[
            square::E3,
            square::E5,
            square::D4,
            square::F4,
            square::F3,
            square::F5,
            square::D3,
            square::D5,
        ]);
        let attacks = queen_attacks(from, BB::new(from), enemy_occupied);
        print!("{}", enemy_occupied);
        print!("{}", attacks);

        let expected = enemy_occupied;
        assert_eq!(attacks, expected);
    }

    #[test]
    pub fn occupied_board_friendly() {
        let from = square::E4;
        let friendly_occupied = get_bb_from_array_of_squares(&[
            square::E3,
            square::E5,
            square::D4,
            square::F4,
            square::F3,
            square::F5,
            square::D3,
            square::D5,
        ]);
        let attacks = queen_attacks(from, friendly_occupied, bitboard::EMPTY);
        print!("{}", friendly_occupied);
        print!("{}", attacks);

        let expected = bitboard::EMPTY;
        assert_eq!(attacks, expected);
    }
}

#[cfg(test)]
pub mod knight_tests {
    use super::*;
    use crate::{square::*, util::get_bb_from_array_of_squares};

    #[test]
    pub fn on_a1() {
        let from = A1;
        let expected = get_bb_from_array_of_squares(&[B3, C2]);
        let attacks = knight_attacks(from, BB::new(from));
        println!("{}", attacks);
        assert_eq!(attacks, expected);
    }

    #[test]
    pub fn on_a1_blocked_on_b3() {
        let from = A1;
        let expected = get_bb_from_array_of_squares(&[C2]);
        let attacks = knight_attacks(from, get_bb_from_array_of_squares(&[from, B3]));
        println!("{}", attacks);
        assert_eq!(attacks, expected);
    }
}

#[cfg(test)]
pub mod king_test {
    use super::*;
    use crate::{bitboard, square::*, util::get_bb_from_array_of_squares};

    #[test]
    pub fn on_e4() {
        let from = E4;
        let expected = get_bb_from_array_of_squares(&[E5, E3, D4, D3, D5, F4, F5, F3]);
        let attacks = king_attacks(from, BB::new(from));
        println!("{}", attacks);
        assert_eq!(attacks, expected);
    }

    #[test]
    pub fn on_e4_blocked() {
        let from = E4;
        let expected = bitboard::EMPTY;
        let attacks = king_attacks(
            from,
            get_bb_from_array_of_squares(&[E5, E3, D4, D3, D5, F4, F5, F3]),
        );
        println!("{}", attacks);
        assert_eq!(attacks, expected);
    }
}

#[cfg(test)]
pub mod pawn_test {
    use super::*;

    #[cfg(test)]
    pub mod white {
        use super::*;
        use crate::{bitboard, square::*, util::get_bb_from_array_of_squares};

        #[test]
        pub fn basic_push() {
            let from = E4;
            let expected = BB::new(E5);
            let attacks = pawn(from, bitboard::EMPTY, bitboard::EMPTY, None, Side::White);
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn double_push() {
            let from = E2;
            let expected = get_bb_from_array_of_squares(&[E3, E4]);
            let attacks = pawn(from, bitboard::EMPTY, bitboard::EMPTY, None, Side::White);
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn blocked_single_push() {
            let from = E2;
            let expected = bitboard::EMPTY;
            let attacks = pawn(from, bitboard::EMPTY, BB::new(E3), None, Side::White);
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn blocked_double_push() {
            let from = E2;
            let expected = BB::new(E3);
            let attacks = pawn(from, bitboard::EMPTY, BB::new(E4), None, Side::White);
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn captures() {
            let from = E2;
            let expected = get_bb_from_array_of_squares(&[F3, D3]);
            let attacks = pawn(
                from,
                BB::new(E3),
                get_bb_from_array_of_squares(&[F3, D3]),
                None,
                Side::White,
            );
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn en_passant_captures() {
            let from = E5;
            let expected = get_bb_from_array_of_squares(&[D6]);
            let attacks = pawn(
                from,
                BB::new(E6),
                get_bb_from_array_of_squares(&[D5]),
                Some(D6),
                Side::White,
            );
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }
    }

    #[cfg(test)]
    pub mod black {
        use super::*;
        use crate::{bitboard, square::*, util::get_bb_from_array_of_squares};

        #[test]
        pub fn basic_push() {
            let from = E4;
            let expected = BB::new(E3);
            let attacks = pawn(from, bitboard::EMPTY, bitboard::EMPTY, None, Side::Black);
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn double_push() {
            let from = E7;
            let expected = get_bb_from_array_of_squares(&[E6, E5]);
            let attacks = pawn(from, bitboard::EMPTY, bitboard::EMPTY, None, Side::Black);
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn blocked_single_push() {
            let from = E7;
            let expected = bitboard::EMPTY;
            let attacks = pawn(from, bitboard::EMPTY, BB::new(E6), None, Side::Black);
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn blocked_double_push() {
            let from = E7;
            let expected = BB::new(E6);
            let attacks = pawn(from, bitboard::EMPTY, BB::new(E5), None, Side::Black);
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn captures() {
            let from = E5;
            let expected = get_bb_from_array_of_squares(&[F4, D4]);
            let attacks = pawn(
                from,
                BB::new(E4),
                get_bb_from_array_of_squares(&[F4, D4]),
                None,
                Side::Black,
            );
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }

        #[test]
        pub fn en_passant_captures() {
            let from = E4;
            let expected = get_bb_from_array_of_squares(&[D3]);
            let attacks = pawn(
                from,
                BB::new(E3),
                get_bb_from_array_of_squares(&[D4]),
                Some(D3),
                Side::Black,
            );
            println!("{}", attacks);
            assert_eq!(attacks, expected);
        }
    }
}
