use crate::{
    bitboard::{self, BB},
    side::Side,
};

#[derive(Copy, Clone, PartialEq)]
pub struct AttackTable([[BB; 6]; 2]);

impl AttackTable {
    pub fn new() -> AttackTable {
        AttackTable([
            [
                bitboard::EMPTY,
                bitboard::EMPTY,
                bitboard::EMPTY,
                bitboard::EMPTY,
                bitboard::EMPTY,
                bitboard::EMPTY,
            ],
            [
                bitboard::EMPTY,
                bitboard::EMPTY,
                bitboard::EMPTY,
                bitboard::EMPTY,
                bitboard::EMPTY,
                bitboard::EMPTY,
            ],
        ])
    }
    pub fn get(self, side: &Side) -> [BB; 6] {
        self.0[side.to_usize()]
    }

    pub fn update(mut self, side: &Side, table: [BB; 6]) {
        self.0[side.to_usize()] = table;
    }
}

// fn blocking_on_rank(sq: Square, attack_bb: BB) -> bool {
//     let sq_bb = BB::new(sq);
//     let blocking_left = (sq_bb.shl(1) & attack_bb).not_empty();
//     let blocking_right = (sq_bb.shr(2) & attack_bb).not_empty();
//
//     blocking_left || blocking_right
// }
//
// fn blocking_on_file(sq: Square, attack_bb: BB) -> bool {
//     let sq_bb = BB::new(sq);
//     let blocking_above = (sq_bb.shl(8) & attack_bb).not_empty();
//     let blocking_below = (sq_bb.shr(8) & attack_bb).not_empty();
//
//     blocking_above || blocking_below
// }
//
// fn blocking_on_anti_diagonal(sq: Square, attack_bb: BB) -> bool {
//     let sq_bb = BB::new(sq);
//     let blocking_above = (sq_bb.shl(9) & attack_bb).not_empty();
//     let blocking_below = (sq_bb.shr(9) & attack_bb).not_empty();
//
//     blocking_above || blocking_below
// }
//
// fn blocking_on_diagonal(sq: Square, attack_bb: BB) -> bool {
//     let sq_bb = BB::new(sq);
//     let blocking_above = (sq_bb.shl(7) & attack_bb).not_empty();
//     let blocking_below = (sq_bb.shr(7) & attack_bb).not_empty();
//
//     blocking_above || blocking_below
// }
