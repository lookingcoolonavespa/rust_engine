use crate::{
    game::Game,
    move_gen::{
        attacks_with_king_gone, check_legal::LegalCheckPreprocessing, checkers_pinners_pinned,
    },
};

fn count_moves_debug(depth: u32, game: &mut Game) -> u32 {
    if depth == 0 {
        return 1;
    }

    let mut count: u32 = 0;

    let side = game.state().side_to_move();
    let pseudo_legal_mv_list = game.pseudo_legal_moves(side);
    let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
    let attacked_squares_with_king_gone_bb =
        attacks_with_king_gone(game.mut_position(), side.opposite());
    let legal_check_preprocessing = LegalCheckPreprocessing::new(
        checkers,
        pinners,
        pinned,
        attacked_squares_with_king_gone_bb,
    );

    for mv in pseudo_legal_mv_list.iter() {
        if !game.is_legal(*mv, &legal_check_preprocessing) {
            continue;
        }

        let (capture, prev_state) = game.make_move(*mv);
        let sub_nodes = count_moves(depth - 1, game);
        println!("{} {}", mv, sub_nodes);
        count += sub_nodes;
        game.unmake_move(*mv, capture, prev_state)
    }

    count
}

fn count_moves(depth: u32, game: &mut Game) -> u32 {
    if depth == 0 {
        return 1;
    }

    let mut count: u32 = 0;

    let side = game.state().side_to_move();
    let pseudo_legal_mv_list = game.pseudo_legal_moves(side);
    let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
    let attacked_squares_with_king_gone_bb =
        attacks_with_king_gone(game.mut_position(), side.opposite());
    let legal_check_preprocessing = LegalCheckPreprocessing::new(
        checkers,
        pinners,
        pinned,
        attacked_squares_with_king_gone_bb,
    );

    for mv in pseudo_legal_mv_list.iter() {
        if !game.is_legal(*mv, &legal_check_preprocessing) {
            continue;
        }

        let (capture, prev_state) = game.make_move(*mv);
        count += count_moves(depth - 1, game);
        game.unmake_move(*mv, capture, prev_state)
    }

    count
}

#[cfg(test)]
pub mod perft {
    use super::*;
    use crate::fen::STARTING_POSITION_FEN;

    #[test]
    fn start_pos_one_ply() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves(1, &mut game), 20)
    }

    #[test]
    fn start_pos_two_ply() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves(2, &mut game), 400)
    }

    #[test]
    fn start_pos_three_ply() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves(3, &mut game), 8902)
    }

    #[test]
    fn start_pos_four_ply() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves(4, &mut game), 197_281)
    }

    #[test]
    fn start_pos_five_ply() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(5, &mut game), 4_865_609)
    }
}
