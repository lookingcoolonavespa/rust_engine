mod king_heuristics;
mod pawn_heuristics;

use crate::{
    bitboard::{self, BB},
    game::Game,
    move_gen::{check_legal::LegalCheckPreprocessing, controlled_squares},
    score::Score,
};

use self::king_heuristics::king_safety;

pub const CHECKMATE_VAL: i32 = i32::MAX;
pub const DRAW_SCORE: Score = Score(-50, -25, 0);

pub fn eval(
    game: &mut Game,
    legal_check_preprocessing: &LegalCheckPreprocessing,
    levels_searched: u8,
) -> i32 {
    if game.is_draw() {
        return DRAW_SCORE.get(game.position().phase());
    }
    if game.is_checkmate(legal_check_preprocessing) {
        return -(CHECKMATE_VAL - levels_searched as i32);
    }
    if game.is_stalemate(legal_check_preprocessing) {
        return DRAW_SCORE.get(game.position().phase());
    }

    let side = game.state().side_to_move();
    let piece_score: i32 =
        game.position().piece_score(side) - game.position().piece_score(side.opposite());
    let sq_score: i32 = game.position().sq_score(side) - game.position().sq_score(side.opposite());

    let controlled_squares = controlled_squares(game.position(), side);
    let center_control = center_control(controlled_squares) as i32
        - center_control(legal_check_preprocessing.controlled_squares_with_king_gone_bb()) as i32;

    let mobility_bonus = mobility(controlled_squares) as i32
        - mobility(legal_check_preprocessing.controlled_squares_with_king_gone_bb()) as i32;

    let king_safety_bonus = king_safety(
        game.position().king_sq(side),
        game.position()
            .bb_pc(crate::piece_type::PieceType::Pawn, side),
    ) as i32
        - king_safety(
            game.position().king_sq(side.opposite()),
            game.position()
                .bb_pc(crate::piece_type::PieceType::Pawn, side.opposite()),
        ) as i32;

    sq_score + piece_score + center_control + king_safety_bonus
}

const INNER_CENTER_CONTROL_MULTIPLER: u32 = 30;
const OUTER_CENTER_CONTROL_MULTIPLER: u32 = 20;
fn center_control(controlled_squares: BB) -> u32 {
    let inner_center_control_bb = controlled_squares & bitboard::INNER_CENTER;
    let outer_center_control_bb = controlled_squares & bitboard::OUTER_CENTER;

    inner_center_control_bb.count_ones() * INNER_CENTER_CONTROL_MULTIPLER
        + outer_center_control_bb.count_ones() * OUTER_CENTER_CONTROL_MULTIPLER
}

const MOBILITY_MULTIPLIER: u32 = 1;
fn mobility(controlled_squares: BB) -> u32 {
    controlled_squares.count_ones() * MOBILITY_MULTIPLIER
}
