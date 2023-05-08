use std::io;

use crate::{
    fen::STARTING_POSITION_FEN,
    game::Game,
    move_gen::pseudo_legal::is_double_pawn_push,
    mv::{castle::Castle, Decode, EncodedMove, Move, PromotionMove},
    perft::count_moves_debug,
    piece_type::{PieceType, PromoteType},
    search::MoveFinder,
    side::Side,
    square::{self, Square},
};

pub fn main() {
    let mut game = Game::from_fen(STARTING_POSITION_FEN)
        .expect("game is not loading the starting position fen correctly");
    let mut mv_finder = MoveFinder::new(game.clone());

    loop {
        let mut input_str = String::new();
        io::stdin()
            .read_line(&mut input_str)
            .expect("failed to read line");

        match input_str.trim() {
            "uci" => {
                input_uci();
            }
            "isready" => {
                input_is_ready();
            }
            "ucinewgame" => {
                game = input_uci_new_game();
                mv_finder = mv_finder.set_game(game.clone());
            }
            input if input.starts_with("position") => {
                game = input_position(&input_str, game);
                mv_finder = mv_finder.set_game(game.clone());
            }
            input if input.starts_with("go perft") => input_perft(&input_str, &mut game),
            input if input.starts_with("go") => input_go(&mut game, &mut mv_finder),
            "quit:" => input_quit(),
            "print" => print(&game),
            _ => {
                println!("Invalid input: {}", input_str);
            }
        };
    }
}

fn input_uci() {
    println!("id name croChess");
    println!("id author alex");
    // options go here
    println!("uciok");
}

fn input_is_ready() {
    println!("readyok");
}

fn input_uci_new_game() -> Game {
    Game::from_fen(STARTING_POSITION_FEN).unwrap()
}

fn input_quit() {
    std::process::exit(0);
}

fn decode_algebra(move_notation: &str) -> (Square, Square, Option<PromoteType>) {
    let mut chars = move_notation.chars();
    let from = chars.next().unwrap() as i32 - 'a' as i32
        + (8 * (chars.next().unwrap() as i32 - '1' as i32));
    let to = chars.next().unwrap() as i32 - 'a' as i32
        + (8 * (chars.next().unwrap() as i32 - '1' as i32));

    let mut promote_pc = None;
    if move_notation.len() == 5 {
        match PromoteType::try_from(move_notation.chars().nth(4).unwrap()) {
            Ok(promote_type) => {
                promote_pc = Some(promote_type);
            }
            Err(err) => {
                println!("{}", err)
            }
        }
    }

    (
        Square::new(from as usize),
        Square::new(to as usize),
        promote_pc,
    )
}

#[cfg(test)]
pub mod test_decode_algebra {
    use super::*;
    use crate::square::*;

    #[test]
    fn e2e4() {
        let (from, to, promote_pc) = decode_algebra("e2e4");
        assert_eq!(from, E2);
        assert_eq!(to, E4);
        assert_eq!(promote_pc, None);
    }

    #[test]
    fn promote_1() {
        let (from, to, promote_pc) = decode_algebra("e2e4q");
        assert_eq!(from, E2);
        assert_eq!(to, E4);
        assert_eq!(promote_pc, Some(PromoteType::Queen));
    }
}

fn algebra_to_move(move_notation: &str, game: &Game) -> Result<Move, String> {
    if move_notation.len() < 4 || move_notation.len() > 5 {
        return Err("invalid move notation".to_string());
    }
    let (from, to, promote_pc) = decode_algebra(move_notation);
    let moving_piece_result = game.position().at(from);
    match moving_piece_result {
        Some(pc) => {
            let piece_type = pc.piece_type();
            let side = pc.side();

            match piece_type {
                PieceType::King => {
                    let side = pc.side();

                    if from.distance(to) == 2 {
                        let (_, queenside_sq) = Castle::Queenside.king_squares(side);
                        let (_, kingside_sq) = Castle::Kingside.king_squares(side);
                        if to == queenside_sq {
                            return Ok(Move::Castle(Castle::Queenside));
                        } else if to == kingside_sq {
                            return Ok(Move::Castle(Castle::Kingside));
                        } else {
                            return Err("king is attemping to move two squares, but it is not a castle move".to_string());
                        }
                    }

                    let is_capture = match game.position().at(to) {
                        Some(_) => true,
                        None => false,
                    };

                    Ok(Move::King(EncodedMove::new(
                        from, to, piece_type, is_capture,
                    )))
                }
                PieceType::Pawn => {
                    let is_capture = match game.position().at(to) {
                        Some(_) => true,
                        None => false,
                    };

                    if to == game.state().en_passant().unwrap_or(square::NULL) {
                        Ok(Move::EnPassant(EncodedMove::new(
                            from,
                            to,
                            PieceType::Pawn,
                            true,
                        )))
                    } else if promote_pc.is_some() {
                        Ok(Move::Promotion(PromotionMove::new(
                            from,
                            to,
                            &promote_pc.unwrap(),
                            is_capture,
                        )))
                    } else if is_double_pawn_push(from, to, side) {
                        Ok(Move::DoublePawnPush(EncodedMove::new(
                            from,
                            to,
                            PieceType::Pawn,
                            false,
                        )))
                    } else {
                        Ok(Move::Pawn(EncodedMove::new(
                            from,
                            to,
                            PieceType::Pawn,
                            is_capture,
                        )))
                    }
                }
                PieceType::Rook => {
                    let is_capture = match game.position().at(to) {
                        Some(_) => true,
                        None => false,
                    };
                    Ok(Move::Rook(EncodedMove::new(
                        from, to, piece_type, is_capture,
                    )))
                }
                _ => {
                    let is_capture = match game.position().at(to) {
                        Some(_) => true,
                        None => false,
                    };
                    Ok(Move::Piece(EncodedMove::new(
                        from, to, piece_type, is_capture,
                    )))
                }
            }
        }
        None => Err(format!("no piece at {from}")),
    }
}

#[cfg(test)]
pub mod test_algebra_to_move {
    use super::*;
    use crate::square::*;

    #[test]
    fn double_pawn_push() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let mv_result = algebra_to_move("e2e4", &game);
        assert!(mv_result.is_ok());
        assert_eq!(
            mv_result.unwrap(),
            Move::DoublePawnPush(EncodedMove::new(E2, E4, PieceType::Pawn, false))
        )
    }

    #[test]
    fn en_passant() {
        let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let mv_result = algebra_to_move("e5d6", &game);
        assert!(mv_result.is_ok());
        assert_eq!(
            mv_result.unwrap(),
            Move::EnPassant(EncodedMove::new(E5, D6, PieceType::Pawn, true))
        )
    }

    #[test]
    fn promotion() {
        let fen = "rnbqkbn1/ppp1pppP/8/3p4/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let mv_result = algebra_to_move("h7h8q", &game);
        assert!(mv_result.is_ok());
        assert_eq!(
            mv_result.unwrap(),
            Move::Promotion(PromotionMove::new(H7, H8, &PromoteType::Queen, false))
        )
    }

    #[test]
    fn castle_kingside_w() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let mv_result = algebra_to_move("e1g1", &game);
        assert!(mv_result.is_ok());
        assert_eq!(mv_result.unwrap(), Move::Castle(Castle::Kingside))
    }

    #[test]
    fn castle_queenside_w() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let mv_result = algebra_to_move("e1c1", &game);
        assert!(mv_result.is_ok());
        assert_eq!(mv_result.unwrap(), Move::Castle(Castle::Queenside))
    }

    #[test]
    fn rook_capture() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let mv_result = algebra_to_move("h1h8", &game);
        assert!(mv_result.is_ok());
        assert_eq!(
            mv_result.unwrap(),
            Move::Rook(EncodedMove::new(H1, H8, PieceType::Rook, true))
        )
    }

    #[test]
    fn piece_mv() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K1NR w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let mv_result = algebra_to_move("g1f3", &game);
        assert!(mv_result.is_ok());
        assert_eq!(
            mv_result.unwrap(),
            Move::Piece(EncodedMove::new(G1, F3, PieceType::Knight, false))
        )
    }
}

pub fn input_position(input: &str, mut game: Game) -> Game {
    let mut input = input[9..].to_owned() + " ";
    if input.contains("startpos") {
        input = input[9..].to_owned();
        let result = Game::from_fen(STARTING_POSITION_FEN);
        game = result.unwrap();
    } else if input.contains("fen") {
        input = input[4..].to_owned();
        let result = Game::from_fen(input.trim());
        game = result.expect("invalid fen");
    }

    if input.contains("moves") {
        input = input[input.find("moves").unwrap() + 6..].to_owned();
        // make each of the moves
        let moves = input.trim().split(' ');
        for move_notation in moves {
            let mv_result = algebra_to_move(move_notation, &game);
            match mv_result {
                Ok(mv) => {
                    game.make_move(mv);
                }
                Err(err) => {
                    println!("{}", err);
                }
            }
        }
    }

    game
}

fn input_perft(input: &str, game: &mut Game) {
    let depth_result = input[8..].to_owned().trim().parse();
    if let Ok(num) = depth_result {
        count_moves_debug(num, game);
    } else {
        println!("wasn't given a valid number for perft depth'")
    }
}

#[cfg(test)]
pub mod test_input_position {
    use crate::side::Side;
    use crate::square::*;

    use super::*;

    #[test]
    fn startpos() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K1NR w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let game = input_position("position startpos", game);
        let position = game.position();
        let expected = unindent::unindent(
            "
                  ABCDEFGH
                8|rnbqkbnr|8
                7|pppppppp|7
                6|........|6
                5|........|5
                4|........|4
                3|........|3
                2|PPPPPPPP|2
                1|RNBQKBNR|1
                  ABCDEFGH
                ",
        );

        assert_eq!(position.to_string(), expected);
    }

    #[test]
    fn fen() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K1NR w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let game = input_position(&format!("position fen {}", fen), game);

        let position = game.position();
        let expected = unindent::unindent(
            "
                  ABCDEFGH
                8|r...k..r|8
                7|........|7
                6|........|6
                5|........|5
                4|........|4
                3|........|3
                2|........|2
                1|R...K.NR|1
                  ABCDEFGH
                ",
        );

        assert_eq!(position.to_string(), expected);
    }

    #[test]
    fn moves_1() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K1NR w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let game = input_position("position startpos moves e2e4 e7e5 g1f3 b8c6", game);

        let position = game.position();
        println!("{}", position.to_string());
        let expected = unindent::unindent(
            "
                  ABCDEFGH
                8|r.bqkbnr|8
                7|pppp.ppp|7
                6|..n.....|6
                5|....p...|5
                4|....P...|4
                3|.....N..|3
                2|PPPP.PPP|2
                1|RNBQKB.R|1
                  ABCDEFGH
                ",
        );

        assert_eq!(position.to_string(), expected);
    }

    #[test]
    fn moves_2() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let game = input_position("position startpos moves a2a3 b7b5", game);

        let position = game.position();
        println!("{}", position.to_string());
        let expected = unindent::unindent(
            "
                  ABCDEFGH
                8|rnbqkbnr|8
                7|p.pppppp|7
                6|........|6
                5|.p......|5
                4|........|4
                3|P.......|3
                2|.PPPPPPP|2
                1|RNBQKBNR|1
                  ABCDEFGH
                ",
        );

        assert_eq!(position.to_string(), expected);
    }

    #[test]
    fn moves_iterative() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let game = input_position("position startpos moves a2a3", game);
        let game = input_position("position startpos moves a2a3 b7b5", game);

        let position = game.position();
        println!("{}", position.to_string());
        let expected = unindent::unindent(
            "
                  ABCDEFGH
                8|rnbqkbnr|8
                7|p.pppppp|7
                6|........|6
                5|.p......|5
                4|........|4
                3|P.......|3
                2|.PPPPPPP|2
                1|RNBQKBNR|1
                  ABCDEFGH
                ",
        );

        assert_eq!(position.to_string(), expected);
    }
}

fn move_to_algebra(mv: Move, side: Side) -> String {
    match mv {
        Move::King(mv)
        | Move::Rook(mv)
        | Move::Pawn(mv)
        | Move::DoublePawnPush(mv)
        | Move::Piece(mv)
        | Move::EnPassant(mv) => {
            let (from, to) = mv.decode_into_squares();
            format!(
                "{}{}",
                from.to_string().to_lowercase(),
                to.to_string().to_lowercase()
            )
        }
        Move::Castle(castle_mv) => {
            let (from, to) = castle_mv.king_squares(side);
            format!(
                "{}{}",
                from.to_string().to_lowercase(),
                to.to_string().to_lowercase()
            )
        }
        Move::Promotion(promotion_mv) => {
            let (from, to) = promotion_mv.decode_into_squares();
            let promote_type_char = promotion_mv.promote_piece_type().to_char();
            format!(
                "{}{}{}",
                from.to_string().to_lowercase(),
                to.to_string().to_lowercase(),
                promote_type_char.to_string().to_lowercase()
            )
        }
    }
}

#[cfg(test)]
pub mod test_move_to_algebra {
    use super::*;
    use crate::square::*;

    #[test]
    fn regular_move() {
        let mv = Move::Piece(EncodedMove::new(A4, A5, PieceType::Pawn, false));
        assert_eq!(move_to_algebra(mv, Side::White), "a4a5")
    }

    #[test]
    fn promotion_move() {
        let mv = Move::Promotion(PromotionMove::new(A7, A8, &PromoteType::Queen, false));
        assert_eq!(move_to_algebra(mv, Side::White), "a7a8q")
    }

    #[test]
    fn castle_move_1() {
        let mv = Move::Castle(Castle::Kingside);
        assert_eq!(move_to_algebra(mv, Side::White), "e1g1")
    }

    #[test]
    fn castle_move_2() {
        let mv = Move::Castle(Castle::Queenside);
        assert_eq!(move_to_algebra(mv, Side::White), "e1c1")
    }

    #[test]
    fn castle_move_3() {
        let mv = Move::Castle(Castle::Kingside);
        assert_eq!(move_to_algebra(mv, Side::Black), "e8g8")
    }

    #[test]
    fn castle_move_4() {
        let mv = Move::Castle(Castle::Queenside);
        assert_eq!(move_to_algebra(mv, Side::Black), "e8c8")
    }
}

fn input_go(game: &mut Game, mv_finder: &mut MoveFinder) {
    // search for best move
    let (best_move, _) = mv_finder.get().unwrap();
    let algebra = move_to_algebra(best_move, game.state().side_to_move());
    println!("bestmove {}", algebra);
}

fn print(game: &Game) {
    println!("{}", game.position());
}
