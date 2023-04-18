use crate::{
    bitboard::BB,
    mv::castle::{castle_must_clear_squares, castle_pass_through_squares, Castle},
    side::Side,
    state::position::Position,
};

use super::pseudo_legal;

struct LegalCheckPreprocessing {
    checkers: BB,
    pinners: BB,
    pinned: BB,
    attacked_squares_bb: BB,
}

impl LegalCheckPreprocessing {
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

    pub fn king_safe_squares(&self, position: &Position, king_color: Side) -> BB {
        let king_mvs_bb =
            pseudo_legal::king_attacks(position.king_sq(king_color), position.bb_side(king_color));

        king_mvs_bb & !self.attacked_squares_bb
    }

    pub fn can_castle(&self, position: &Position, castle: Castle, side: Side) -> bool {
        let occupied = position.bb_occupied();
        let must_clear_squares = castle_must_clear_squares(side, castle);
        if (occupied & must_clear_squares).not_empty() {
            return false;
        }

        (castle_pass_through_squares(side, castle) & self.attacked_squares_bb).empty()
    }
}
