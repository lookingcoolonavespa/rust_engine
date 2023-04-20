use crate::{
    game::Game,
    move_gen::{attacks, check_legal::LegalCheckPreprocessing, checkers_pinners_pinned},
};

fn count_moves(depth: u32, game: &mut Game) -> u32 {
    if depth == 0 {
        return 1;
    }

    let count: u32 = 0;

    let side = game.state().side_to_move();
    let pseudo_legal_mv_list = game.pseudo_legal_moves(side);
    let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
    let attacked_squares_bb = attacks(game.position(), side.opposite());
    let legal_check_preprocessing =
        LegalCheckPreprocessing::new(checkers, pinners, pinned, attacked_squares_bb);

    for mv in pseudo_legal_mv_list.iter() {
        if !game.is_legal(*mv, &legal_check_preprocessing) {
            continue;
        }
    }

    count
}
