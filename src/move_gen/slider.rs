use std::ops::Mul;

use crate::{bitboard::BB, square::Square};

fn hyp_quint(from: Square, occupied: BB, mask: BB) -> BB {
    let masked_occupied = occupied & mask;
    let piece_bb = BB::new(from);
    let positive_direction = (masked_occupied) - piece_bb.mul(BB(2u64));
    let negative_direction =
        ((masked_occupied).reverse() - piece_bb.reverse().mul(BB(2u64))).reverse();
    (positive_direction ^ negative_direction) & mask
}

pub fn horizontal_moves_from_sq(from: Square, occupied: BB) -> BB {
    hyp_quint(from, occupied, from.rank_mask())
}

pub fn vertical_moves_from_sq(from: Square, occupied: BB) -> BB {
    hyp_quint(from, occupied, from.file_mask())
}

pub fn diagonal_moves_from_sq(from: Square, occupied: BB) -> BB {
    hyp_quint(from, occupied, from.diagonal_mask())
}

pub fn anti_diagonal_moves_from_sq(from: Square, occupied: BB) -> BB {
    hyp_quint(from, occupied, from.anti_diagonal_mask())
}

#[cfg(test)]
pub mod rank_and_file_tests {
    use super::*;
    use crate::{bitboard, square};

    #[test]
    fn empty_rank() {
        let attacks = hyp_quint(square::A1, bitboard::EMPTY, square::A1.rank_mask());
        let expected = bitboard::ROW_1 - BB::new(square::A1);

        assert_eq!(attacks, expected);
    }

    #[test]
    fn rank_occupied_on_left() {
        let attacks = hyp_quint(square::C1, BB::new(square::B1), square::C1.rank_mask());
        println!("{}", attacks);
        let expected = (bitboard::ROW_1 ^ BB::new(square::C1)) ^ BB::new(square::A1);

        assert_eq!(attacks, expected);
    }

    #[test]
    fn rank_occupied_on_right() {
        let attacks = hyp_quint(square::A1, BB::new(square::B1), square::A1.rank_mask());
        println!("{}", attacks);
        let expected = BB::new(square::B1);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn rank_occupied_on_both_sides() {
        let attacks = hyp_quint(square::B1, BB(5u64), square::A1.rank_mask());
        println!("{}", attacks);
        let expected = BB(5u64);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn empty_file() {
        let attacks = hyp_quint(square::A1, bitboard::EMPTY, square::A1.file_mask());
        let expected = bitboard::FILE_A ^ BB::new(square::A1);

        assert_eq!(attacks, expected);
    }

    #[test]
    fn file_occupied_below() {
        let attacks = hyp_quint(square::C5, BB::new(square::C3), square::C5.file_mask());
        println!("{}", attacks);
        let expected = (bitboard::FILE_A << 2)
            ^ (BB::new(square::C1) | BB::new(square::C2) | BB::new(square::C5));

        assert_eq!(attacks, expected);
    }

    #[test]
    fn file_occupied_above() {
        let attacks = hyp_quint(square::C3, BB::new(square::C5), square::C3.file_mask());
        println!("{}", attacks);
        let expected = (bitboard::FILE_A << 2)
            ^ (BB::new(square::C6)
                | BB::new(square::C7)
                | BB::new(square::C8)
                | BB::new(square::C3));

        assert_eq!(attacks, expected);
    }
}

#[cfg(test)]
mod diagonal_and_anti_diagonal_tests {
    use crate::{bitboard, square};

    use super::*;

    #[test]
    fn empty_diagonal() {
        let attacks = diagonal_moves_from_sq(square::H1, bitboard::EMPTY);
        print!("{}", attacks);
        let expected = square::H1.diagonal_mask();
        assert_eq!(attacks, expected);
    }

    #[test]
    fn occupied_diagonal() {
        let attacks = diagonal_moves_from_sq(square::H1, BB::new(square::F3));
        print!("{}", attacks);
        let expected = BB::new(square::G2) | BB::new(square::F3);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn empty_anti_diagonal() {
        let attacks = anti_diagonal_moves_from_sq(square::A1, bitboard::EMPTY);
        print!("{}", attacks);
        let expected = square::A1.anti_diagonal_mask();
        assert_eq!(attacks, expected);
    }

    #[test]
    fn occupied_anti_diagonal() {
        let attacks = anti_diagonal_moves_from_sq(square::A1, BB::new(square::C3));
        print!("{}", attacks);
        let expected = BB::new(square::B2) | BB::new(square::C3);
        assert_eq!(attacks, expected);
    }
}
