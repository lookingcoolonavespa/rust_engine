use crate::{bitboard::BB, side::Side, square::Square};

use super::{check_legal::LegalCheckPreprocessing, pseudo_legal};

pub fn king_moves(
    from: Square,
    friendly_occupied: BB,
    legal_check_preprocessing: &LegalCheckPreprocessing,
) -> BB {
    pseudo_legal::king_attacks(from, friendly_occupied)
        & !legal_check_preprocessing.controlled_squares_with_king_gone_bb()
}

pub fn knight_moves(from: Square, friendly_occupied: BB, check_ray: BB) -> BB {
    pseudo_legal::knight_attacks(from, friendly_occupied) & check_ray
}

pub fn bishop_moves(from: Square, friendly_occupied: BB, enemy_occupied: BB, check_ray: BB) -> BB {
    // check_ray includes the square of the checker
    pseudo_legal::bishop_attacks(from, friendly_occupied, enemy_occupied) & check_ray
}

pub fn queen_moves(from: Square, friendly_occupied: BB, enemy_occupied: BB, check_ray: BB) -> BB {
    // check_ray includes the square of the checker
    pseudo_legal::queen_attacks(from, friendly_occupied, enemy_occupied) & check_ray
}

pub fn rook_moves(from: Square, friendly_occupied: BB, enemy_occupied: BB, check_ray: BB) -> BB {
    // check_ray includes the square of the checker
    pseudo_legal::rook_attacks(from, friendly_occupied, enemy_occupied) & check_ray
}

pub fn pawn_moves(
    from: Square,
    friendly_occupied: BB,
    enemy_occupied: BB,
    en_passant: Option<Square>,
    side: Side,
    check_ray: BB,
) -> BB {
    // check_ray includes the square of the checker
    pseudo_legal::pawn(from, friendly_occupied, enemy_occupied, en_passant, side) & check_ray
}
