use std::io;

use crate::{
    fen::STARTING_POSITION_FEN,
    game::Game,
    move_gen::pseudo_legal::{is_double_pawn_push, is_pseudo_legal},
    mv::{EncodedMove, Move, PromotionMove},
    piece_type::{PieceType, PromoteType},
    square::{self, Square},
};

fn main() {
    loop {
        let mut input_str = String::new();
        io::stdin()
            .read_line(&mut input_str)
            .expect("failed to read line");

        match input_str.trim() {
            "com/crochess/engine0x88/uci" => {
                input_uci();
            }
            "isready" => {
                input_is_ready();
            }
            "ucinewgame" => {
                input_uci_new_game();
            }
            // input if input.starts_with("position") => input_position(input),
            // input if input.starts_with("go") => input_go(),
            // "quit:" => input_quit(),
            // "print" => print(),
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
    Game::from_fen("STARTING_POSITION_FEN").unwrap()
}

fn input_quit() {
    std::process::exit(0);
}

fn decode_algebra(move_notation: &str) -> (Square, Square, Option<PromoteType>) {
    let from = move_notation.chars().nth(0).unwrap() as i32 - 'a' as i32
        + (8 * (move_notation.chars().nth(1).unwrap() as i32 - '1' as i32));
    let to = move_notation.chars().nth(2).unwrap() as i32 - 'a' as i32
        + (8 * (move_notation.chars().nth(3).unwrap() as i32 - '1' as i32));

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

fn algebra_to_move(move_notation: &str, game: Game) -> Result<Move, &str> {
    let (from, to, promote_pc) = decode_algebra(move_notation);
    let moving_piece_result = game.position().at(from);
    match moving_piece_result {
        Some(pc) => {
            let piece_type = pc.piece_type();
            let side = pc.side();

            let friendly_occupied = game.position().bb_side(side);
            let enemy_occupied = game.position().bb_side(side.opposite());
            let en_passant = game.state().en_passant();

            // handle castling

            match piece_type {
                PieceType::King => {
                    let side = pc.side();

                    if from.distance(to) == 2 {
                        mv = if color == Color::White {
                            (Castle::W_Q.value << 14) | mv
                        } else {
                            (Castle::B_q.value << 14) | mv
                        };
                    } else if from - to == -2 {
                        mv = if color == Color::White {
                            (Castle::W_K.value << 14) | mv
                        } else {
                            (Castle::B_k.value << 14) | mv
                        };
                    }
                }
                PieceType::Pawn => {
                    let is_capture = match game.position().at(to) {
                        Some(pc) => true,
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
                        Ok(Move::Pawn(EncodedMove::new(
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
                        Some(pc) => true,
                        None => false,
                    };
                    Ok(Move::Rook(EncodedMove::new(
                        from, to, piece_type, is_capture,
                    )))
                }
                _ => {
                    let is_capture = match game.position().at(to) {
                        Some(pc) => true,
                        None => false,
                    };
                    Ok(Move::Piece(EncodedMove::new(
                        from, to, piece_type, is_capture,
                    )))
                }
            }
        }
        None => Err(&format!("no piece at {from}")),
    }
}

// fn input_position(input: &str, game: Game) -> Game {
//     let mut input = input[9..].to_owned() + " ";
//     if input.contains("startpos ") {
//         input = input[9..].to_owned();
//         let result = Game::from_fen(STARTING_POSITION_FEN);
//         game = result.unwrap();
//     } else if input.contains("fen") {
//         input = input[4..].to_owned();
//         let result = Game::from_fen(&input);
//         game = result.expect("invalid fen");
//     }
//
//     if input.contains("moves") {
//         input = input[input.find("moves").unwrap() + 6..].to_owned();
//         // make each of the moves
//         let moves = input.split(' ');
//         for move_notation in moves {
//             let mv = algebra_to_move(move_notation, game);
//             game.make_move(mv);
//         }
//     }
//
//     game
// }

// fn move_to_algebra(mv: i32) -> String {
//     let from = Square::lookup((mv >> 7) & 127);
//     let to = Square::lookup(mv & 127);
//     let promote = mv >> 18;
//
//     let mut algebra = format!(
//         "{}{}",
//         from.to_string().to_lowercase(),
//         to.to_string().to_lowercase()
//     );
//     if promote != 0 {
//         let mut abbr_map = HashMap::new();
//         abbr_map.insert(5, 'q');
//         abbr_map.insert(4, 'r');
//         abbr_map.insert(2, 'n');
//         abbr_map.insert(3, 'b');
//
//         algebra.push(abbr_map.get(&promote).unwrap());
//     }
//
//     algebra
// }

// fn input_go() {
//     // search for best move
//     let best_move = MoveEval::get_best_move(5);
//     let algebra = move_to_algebra(best_move);
//     println!("bestmove {}", algebra);
// }

fn print(game: Game) {
    println!("{}", game.position().to_string());
}
