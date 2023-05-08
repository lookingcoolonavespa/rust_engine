use crate::{game::Game, move_gen::check_legal::LegalCheckPreprocessing};

pub const CHECKMATE_VAL: i32 = i32::MAX;
pub const DRAW_VAL: i32 = 0;

pub fn eval(
    game: &mut Game,
    legal_check_preprocessing: &LegalCheckPreprocessing,
    levels_searched: u8,
) -> i32 {
    if game.is_draw() {
        return DRAW_VAL;
    }
    if game.is_checkmate(legal_check_preprocessing) {
        return -CHECKMATE_VAL + levels_searched as i32;
    }
    if game.is_stalemate(legal_check_preprocessing) {
        return DRAW_VAL;
    }

    let side = game.state().side_to_move();
    let score: i32 =
        game.position().score(side) as i32 - game.position().score(side.opposite()) as i32;

    score
}
