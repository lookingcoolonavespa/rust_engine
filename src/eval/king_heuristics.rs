use crate::{
    bitboard::{BB, KING_MOVES},
    side::Side,
    square::Square,
};

const PAWNS_IN_FRONT_OF_KING_MULTIPLIER: u32 = 10;
pub fn king_safety(king_sq: Square, pawns_bb: BB) -> u32 {
    let king_vicinity = KING_MOVES[king_sq.to_usize()];
    (pawns_bb & king_vicinity).count_ones() * PAWNS_IN_FRONT_OF_KING_MULTIPLIER
}
