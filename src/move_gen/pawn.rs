#[cfg(test)]
use crate::{
    bitboard::{self, BB},
    square,
};

#[cfg(test)]
fn get_w_pawn_pushes() -> [BB; 64] {
    square::ALL.map(|sq| {
        let sq_bb = BB::new(&sq);
        if sq.rank() == 0 || sq.rank() == 7 {
            bitboard::EMPTY
        } else {
            sq_bb << 8
        }
    })
}

#[cfg(test)]
fn get_w_pawn_captures() -> [BB; 64] {
    square::ALL.map(|sq| {
        let sq_bb = BB::new(&sq);
        let next_rank_mask = sq.rank_mask() << 8;
        if sq.rank() == 0 || sq.rank() == 7 {
            bitboard::EMPTY
        } else {
            (sq_bb << 9 | sq_bb << 7) & next_rank_mask
        }
    })
}

#[cfg(test)]
fn get_b_pawn_captures() -> [BB; 64] {
    square::ALL.map(|sq| {
        let sq_bb = BB::new(&sq);
        let next_rank_mask = sq.rank_mask() >> 8;
        if sq.rank() == 0 || sq.rank() == 7 {
            bitboard::EMPTY
        } else {
            (sq_bb >> 9 | sq_bb >> 7) & next_rank_mask
        }
    })
}

#[cfg(test)]
fn get_b_pawn_pushes() -> [BB; 64] {
    square::ALL.map(|sq| {
        let sq_bb = BB::new(&sq);
        if sq.rank() == 0 || sq.rank() == 7 {
            bitboard::EMPTY
        } else {
            sq_bb >> 8
        }
    })
}

#[cfg(test)]
pub mod w_pawn_pushes {
    use super::*;
    use crate::{square::*, util::get_bb_from_array_of_squares};

    #[test]
    pub fn on_second_rank() {
        let w_pawn_pushes = get_w_pawn_pushes();
        let from = E2;
        let pushes = w_pawn_pushes[from.to_usize()];
        let expected = get_bb_from_array_of_squares(&[E3]);
        assert_eq!(pushes, expected);
    }

    #[test]
    pub fn on_third_rank() {
        let w_pawn_pushes = get_w_pawn_pushes();
        let from = E3;
        let pushes = w_pawn_pushes[from.to_usize()];
        let expected = get_bb_from_array_of_squares(&[E4]);
        assert_eq!(pushes, expected);
    }

    #[ignore = "only needed to grab the values"]
    #[test]
    pub fn print() {
        let w_pawn_pushes = get_w_pawn_pushes();
        println!("{:#?}", w_pawn_pushes);
        assert_eq!(true, false)
    }
}

#[cfg(test)]
pub mod w_pawn_captures {
    use super::*;
    use crate::{square::*, util::get_bb_from_array_of_squares};

    #[test]
    pub fn on_file_a() {
        let w_pawn_captures = get_w_pawn_captures();
        let from = A2;
        let captures = w_pawn_captures[from.to_usize()];
        let expected = get_bb_from_array_of_squares(&[B3]);
        assert_eq!(captures, expected);
    }

    #[test]
    pub fn in_middle() {
        let w_pawn_captures = get_w_pawn_captures();
        let from = E3;
        let captures = w_pawn_captures[from.to_usize()];
        let expected = get_bb_from_array_of_squares(&[F4, D4]);
        assert_eq!(captures, expected);
    }

    #[ignore = "only needed to grab the values"]
    #[test]
    pub fn print() {
        let w_pawn_captures = get_w_pawn_captures();
        println!("{:#?}", w_pawn_captures);
        assert_eq!(true, false)
    }
}

#[cfg(test)]
pub mod b_pawn_pushes {
    use super::*;
    use crate::{square::*, util::get_bb_from_array_of_squares};

    #[test]
    pub fn on_seventh_rank() {
        let b_pawn_pushes = get_b_pawn_pushes();
        let from = E7;
        let pushes = b_pawn_pushes[from.to_usize()];
        let expected = get_bb_from_array_of_squares(&[E6]);
        assert_eq!(pushes, expected);
    }

    #[test]
    pub fn on_sixth_rank() {
        let b_pawn_pushes = get_b_pawn_pushes();
        let from = E6;
        let pushes = b_pawn_pushes[from.to_usize()];
        let expected = get_bb_from_array_of_squares(&[E5]);
        assert_eq!(pushes, expected);
    }

    #[ignore = "only needed to grab the values"]
    #[test]
    pub fn print() {
        let b_pawn_pushes = get_b_pawn_pushes();
        println!("{:#?}", b_pawn_pushes);
        assert_eq!(true, false)
    }
}

#[cfg(test)]
pub mod b_pawn_captures {
    use super::*;
    use crate::{square::*, util::get_bb_from_array_of_squares};

    #[test]
    pub fn on_file_a() {
        let b_pawn_captures = get_b_pawn_captures();
        let from = A2;
        let captures = b_pawn_captures[from.to_usize()];
        let expected = get_bb_from_array_of_squares(&[B1]);
        assert_eq!(captures, expected);
    }

    #[test]
    pub fn in_middle() {
        let b_pawn_captures = get_b_pawn_captures();
        let from = E3;
        let captures = b_pawn_captures[from.to_usize()];
        let expected = get_bb_from_array_of_squares(&[F2, D2]);
        assert_eq!(captures, expected);
    }

    #[ignore = "only needed to grab the values"]
    #[test]
    pub fn print() {
        let b_pawn_captures = get_b_pawn_captures();
        println!("{:#?}", b_pawn_captures);
        assert_eq!(true, false)
    }
}
