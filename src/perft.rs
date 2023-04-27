use crate::{
    game::Game,
    move_gen::{
        check_legal::LegalCheckPreprocessing, checkers_pinners_pinned,
        controlled_squares_with_king_gone,
    },
};

pub fn count_moves_debug(depth: u32, game: &mut Game) -> u32 {
    if depth == 0 {
        return 1;
    }

    let mut count: u32 = 0;

    let side = game.state().side_to_move();
    let pseudo_legal_mv_list = game.pseudo_legal_moves(side);
    let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
    let controlled_squares_with_king_gone_bb =
        controlled_squares_with_king_gone(game.mut_position(), side.opposite());
    let legal_check_preprocessing = LegalCheckPreprocessing::new(
        checkers,
        pinners,
        pinned,
        controlled_squares_with_king_gone_bb,
    );

    let prev_state = game.state().encode();

    for mv in pseudo_legal_mv_list.iter() {
        if !game.is_legal(*mv, &legal_check_preprocessing) {
            continue;
        }

        let capture = game.make_move(*mv);
        let sub_nodes = count_moves(depth - 1, game);
        println!("{}: {}", mv, sub_nodes);
        count += sub_nodes;
        game.unmake_move(*mv, capture, prev_state)
    }

    println!("\ntotal nodes: {}\n", count);

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
        controlled_squares_with_king_gone(game.mut_position(), side.opposite());
    let legal_check_preprocessing = LegalCheckPreprocessing::new(
        checkers,
        pinners,
        pinned,
        attacked_squares_with_king_gone_bb,
    );
    let prev_state = game.state().encode();

    for mv in pseudo_legal_mv_list.iter() {
        if !game.is_legal(*mv, &legal_check_preprocessing) {
            continue;
        }

        let capture = game.make_move(*mv);
        count += count_moves(depth - 1, game);
        game.unmake_move(*mv, capture, prev_state);
    }

    count
}

#[cfg(test)]
fn read_test_suite() {
    // reads perft results from file and run tests against those results

    use std::{fs, path::PathBuf};
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/test/perftsuite.epd");

    let contents = fs::read_to_string(d).expect("should have been able to read from file");
    let lines = contents.split_terminator("\n");
    for test in lines.into_iter() {
        let mut results = test.split(";").into_iter();
        let fen = results
            .next()
            .expect("something went wrong reading a line from the perft test suite");

        let result = Game::from_fen(fen);
        assert!(result.is_ok(), "{} is invalid fen", fen);
        let mut game = result.unwrap();

        for (i, ply_result) in results.enumerate() {
            let depth = i + 1;
            let expected_nodes = ply_result[3..].trim().to_owned().parse::<u32>();
            assert!(
                expected_nodes.is_ok(),
                "{} is not a number",
                ply_result[3..].trim().to_owned()
            );
            assert_eq!(
                count_moves_debug(depth as u32, &mut game),
                expected_nodes.unwrap(),
                "fen: {} depth: {}",
                fen,
                depth
            );
        }
    }
}

#[cfg(test)]
pub mod test_suite {
    use super::*;

    #[ignore = "run when movegen changes"]
    #[test]
    fn run() {
        read_test_suite();
    }
}

#[cfg(test)]
pub mod perft_start_pos {
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

    #[ignore]
    #[test]
    fn start_pos_five_ply() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(5, &mut game), 4_865_609)
    }

    #[ignore = "only need to run when movegen logic changes"]
    #[test]
    fn start_pos_six_ply() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(6, &mut game), 119_060_324)
    }
}

#[cfg(test)]
pub mod perft_fen_1 {
    use super::*;

    const FEN: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    #[test]
    fn one_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(1, &mut game), 48)
    }

    #[test]
    fn two_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(2, &mut game), 2039)
    }

    #[test]
    fn three_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(3, &mut game), 97862)
    }

    #[ignore]
    #[test]
    fn four_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(4, &mut game), 4_085_603)
    }

    #[ignore]
    #[test]
    fn five_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(5, &mut game), 193_690_690)
    }
}

#[cfg(test)]
pub mod perft_fen_2 {
    use super::*;

    const FEN: &str = "4k3/8/8/8/8/8/8/4K2R w K - 0 1";

    #[test]
    fn one_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(1, &mut game), 15)
    }

    #[test]
    fn two_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(2, &mut game), 66)
    }

    #[test]
    fn three_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(3, &mut game), 1197)
    }

    #[test]
    fn four_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(4, &mut game), 7059)
    }

    #[test]
    fn five_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(5, &mut game), 133_987)
    }

    #[ignore]
    #[test]
    fn six_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(6, &mut game), 764_643)
    }
}

#[cfg(test)]
pub mod perft_promotion {
    use crate::move_gen::check_legal::is_legal_king_move;
    use crate::mv::{EncodedMove, Move};
    use crate::square::*;
    use crate::uci::input_position;

    use super::*;

    const FEN: &str = "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1";

    #[test]
    fn promotion_pos_one_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(1, &mut game), 24)
    }

    #[test]
    fn promotion_pos_two_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(2, &mut game), 496)
    }

    #[test]
    fn promotion_pos_three_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(3, &mut game), 9483)
    }

    #[test]
    fn promotion_pos_debug_1() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();

        let mv_1 = Move::King(EncodedMove::new(
            D7,
            E7,
            crate::piece_type::PieceType::King,
            false,
        ));
        game.make_move(mv_1);
        let mv_2 = Move::King(EncodedMove::new(
            E2,
            E3,
            crate::piece_type::PieceType::King,
            false,
        ));
        game.make_move(mv_2);

        let mv_list = game.pseudo_legal_moves(game.state().side_to_move());

        for mv in mv_list.iter() {
            assert!(!matches!(mv, Move::Castle(_)))
        }
    }

    #[test]
    fn promotion_pos_four_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(4, &mut game), 182_838)
    }

    #[ignore]
    #[test]
    fn promotion_pos_five_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(5, &mut game), 3_605_103)
    }

    #[ignore]
    #[test]
    fn promotion_pos_six_ply() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(count_moves_debug(6, &mut game), 71_179_139)
    }

    #[test]
    fn promotion_pos_debug_2() {
        let result = Game::from_fen(FEN);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        game = input_position(
            "position fen n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1 moves g2h1b e2d3 h1d5 d3d4 d7d6",
            game,
        );

        let side = game.state().side_to_move();

        let illegal_mv = EncodedMove::new(D4, D5, crate::piece_type::PieceType::King, true);

        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing::new(
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        );
        println!("{}", controlled_squares_with_king_gone_bb);

        assert!(!is_legal_king_move(illegal_mv, &legal_check_preprocessing));
    }
}
