use crate::{
    bitboard,
    fen::STARTING_POSITION_FEN,
    game::Game,
    move_gen::{
        check_legal::LegalCheckPreprocessing,
        pseudo_legal::{self, is_double_pawn_push},
    },
    move_list::MoveList,
    mv::{castle::Castle, Decode, EncodedMove, Move, PromotionMove},
    piece::Piece,
    piece_type::{PieceType, PROMOTE_TYPE_ARR},
    search::MoveFinder,
    side::Side,
    square::{self, Square, ALL},
    uci::{algebra_to_move, input_position, move_to_algebra},
};
use wasm_bindgen::prelude::*;

//A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct WasmInterface {
    game: Game,
    move_finder: MoveFinder,
}

impl WasmInterface {
    fn pseudo_legal_moves_at_sq(&self, from: Square, piece: Piece) -> MoveList {
        let side = piece.side();
        let piece_type = piece.piece_type();

        let friendly_occupied = self.game.position().bb_side(side);
        let enemy_occupied = self.game.position().bb_side(side.opposite());
        let pseudo_legal_mv_list: MoveList = match piece_type {
            PieceType::Pawn => {
                let mut mv_list = MoveList::new();
                let en_passant = {
                    let stm = self.game.state().side_to_move();

                    if stm == side {
                        self.game.state().en_passant()
                    } else {
                        None
                    }
                };

                let moves_bb =
                    pseudo_legal::pawn(from, friendly_occupied, enemy_occupied, en_passant, side);

                let promote_rank_bb = if side == Side::White {
                    bitboard::ROW_8
                } else {
                    bitboard::ROW_1
                };

                for to in moves_bb.iter() {
                    let is_capture = enemy_occupied.is_set(to);

                    if to == en_passant.unwrap_or(square::NULL) {
                        mv_list.push_move(Move::EnPassant(EncodedMove::new(
                            from,
                            to,
                            PieceType::Pawn,
                            true,
                        )));
                    } else if promote_rank_bb.is_set(to) {
                        for promote_type in PROMOTE_TYPE_ARR.iter() {
                            mv_list.push_move(Move::Promotion(PromotionMove::new(
                                from,
                                to,
                                promote_type,
                                is_capture,
                            )))
                        }
                    } else if is_double_pawn_push(from, to, side) {
                        mv_list.push_move(Move::DoublePawnPush(EncodedMove::new(
                            from,
                            to,
                            PieceType::Pawn,
                            false,
                        )));
                    } else {
                        mv_list.push_move(Move::Pawn(EncodedMove::new(
                            from,
                            to,
                            PieceType::Pawn,
                            is_capture,
                        )));
                    }
                }

                mv_list
            }
            PieceType::Knight => {
                let mut mv_list = MoveList::new();
                let moves_bb = pseudo_legal::knight_attacks(from, friendly_occupied);

                mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                    Move::Piece(EncodedMove::new(
                        from,
                        to,
                        PieceType::Knight,
                        enemy_occupied.is_set(to),
                    ))
                });

                mv_list
            }
            PieceType::Bishop => {
                let mut mv_list = MoveList::new();
                let moves_bb =
                    pseudo_legal::bishop_attacks(from, friendly_occupied, enemy_occupied);

                mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                    Move::Piece(EncodedMove::new(
                        from,
                        to,
                        PieceType::Bishop,
                        enemy_occupied.is_set(to),
                    ))
                });

                mv_list
            }
            PieceType::Rook => {
                let mut mv_list = MoveList::new();
                let moves_bb = pseudo_legal::rook_attacks(from, friendly_occupied, enemy_occupied);

                mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                    Move::Rook(EncodedMove::new(
                        from,
                        to,
                        PieceType::Rook,
                        enemy_occupied.is_set(to),
                    ))
                });

                mv_list
            }
            PieceType::Queen => {
                let mut mv_list = MoveList::new();
                let moves_bb = pseudo_legal::queen_attacks(from, friendly_occupied, enemy_occupied);

                mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                    Move::Piece(EncodedMove::new(
                        from,
                        to,
                        PieceType::Queen,
                        enemy_occupied.is_set(to),
                    ))
                });

                mv_list
            }
            PieceType::King => {
                let mut mv_list = MoveList::new();
                let moves_bb = pseudo_legal::king_attacks(from, friendly_occupied);

                mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                    Move::King(EncodedMove::new(
                        from,
                        to,
                        PieceType::King,
                        enemy_occupied.is_set(to),
                    ))
                });

                let castle_rights = self.game.state().castle_rights();
                if castle_rights.can(side, Castle::Queenside) {
                    mv_list.push_move(Move::Castle(Castle::Queenside))
                }
                if castle_rights.can(side, Castle::Kingside) {
                    mv_list.push_move(Move::Castle(Castle::Kingside))
                }

                mv_list
            }
        };

        pseudo_legal_mv_list
    }

    fn is_move_pseudo_legal(&self, mv: Move, side: Side) -> bool {
        match mv {
            Move::King(mv)
            | Move::Rook(mv)
            | Move::Pawn(mv)
            | Move::DoublePawnPush(mv)
            | Move::EnPassant(mv)
            | Move::Piece(mv) => {
                let (from, to) = mv.decode_into_squares();
                let piece = mv.piece_type();

                let friendly_occupied = self.game.position().bb_side(side);
                let enemy_occupied = self.game.position().bb_side(side.opposite());

                match piece {
                    PieceType::Pawn => {
                        let pseudo_legal_moves = pseudo_legal::pawn(
                            from,
                            friendly_occupied,
                            enemy_occupied,
                            self.game.state().en_passant(),
                            side,
                        );

                        pseudo_legal_moves.is_set(to)
                    }
                    PieceType::Knight => {
                        let pseudo_legal_moves =
                            pseudo_legal::knight_attacks(from, friendly_occupied);

                        pseudo_legal_moves.is_set(to)
                    }
                    PieceType::Bishop => {
                        let pseudo_legal_moves =
                            pseudo_legal::bishop_attacks(from, friendly_occupied, enemy_occupied);

                        pseudo_legal_moves.is_set(to)
                    }
                    PieceType::Rook => {
                        let pseudo_legal_moves =
                            pseudo_legal::rook_attacks(from, friendly_occupied, enemy_occupied);

                        pseudo_legal_moves.is_set(to)
                    }
                    PieceType::Queen => {
                        let pseudo_legal_moves =
                            pseudo_legal::queen_attacks(from, friendly_occupied, enemy_occupied);

                        pseudo_legal_moves.is_set(to)
                    }
                    PieceType::King => {
                        let pseudo_legal_moves =
                            pseudo_legal::king_attacks(from, friendly_occupied);

                        pseudo_legal_moves.is_set(to)
                    }
                }
            }
            Move::Promotion(mv) => {
                let (from, to) = mv.decode_into_squares();
                let friendly_occupied = self.game.position().bb_side(side);
                let enemy_occupied = self.game.position().bb_side(side.opposite());
                let pseudo_legal_moves = pseudo_legal::pawn(
                    from,
                    friendly_occupied,
                    enemy_occupied,
                    self.game.state().en_passant(),
                    side,
                );

                pseudo_legal_moves.is_set(to)
            }
            Move::Castle(castle_mv) => {
                if !self.game.state().castle_rights().can(side, castle_mv) {
                    return false;
                }

                let (king_sq, _) = castle_mv.king_squares(side);
                let (rook_sq, _) = castle_mv.rook_squares(side);

                let king_bb = self.game.position().bb_pc(PieceType::King, side);
                assert!(
                    king_bb.is_set(king_sq),
                    "king is not on {}. {} is not a legal move.",
                    king_sq,
                    mv
                );

                let rook_bb = self.game.position().bb_pc(PieceType::Rook, side);
                assert!(
                    rook_bb.is_set(rook_sq),
                    "rook is not on {}. {} is not a legal move.",
                    rook_sq,
                    mv
                );

                true
            }
        }
    }
}

#[wasm_bindgen]
impl WasmInterface {
    pub fn active_side(&self) -> String {
        self.game.state().side_to_move().to_string()
    }

    pub fn from_history(history: &str) -> WasmInterface {
        let position_str = if history != "" {
            format!("position startpos moves {}", history)
        } else {
            "position startpos".to_string()
        };
        let game = input_position(
            &position_str,
            Game::from_fen(STARTING_POSITION_FEN).unwrap(),
        );

        WasmInterface {
            game: game.clone(),
            move_finder: MoveFinder::new(game),
        }
    }

    pub fn validate_move(&mut self, move_notation: &str, is_white: bool) -> bool {
        let side = if is_white == true {
            Side::White
        } else {
            Side::Black
        };
        let mv_result = algebra_to_move(move_notation, &self.game);
        assert!(mv_result.is_ok(), "{} is not a valid move", move_notation);
        let mv = mv_result.unwrap();

        let mut game = self.game.clone();
        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        self.is_move_pseudo_legal(mv, side) && game.is_legal(mv, &legal_check_preprocessing)
    }

    pub fn legal_moves_at_sq(&mut self, from: u32) -> Vec<u32> {
        let from = ALL[from as usize];

        let piece_result = self.game.position().at(from);
        assert!(piece_result.is_some(), "no piece found at {}", from);
        let piece = piece_result.unwrap();
        let pseudo_legal_mv_list = self.pseudo_legal_moves_at_sq(from, piece);

        let mut game = self.game.clone();
        let side = piece.side();
        let mut legal_moves: Vec<u32> =
            Vec::with_capacity(pseudo_legal_mv_list.list().len() as usize);
        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);
        for mv in pseudo_legal_mv_list.list() {
            if game.is_legal(*mv, &legal_check_preprocessing) {
                match mv {
                    Move::King(mv)
                    | Move::Rook(mv)
                    | Move::Pawn(mv)
                    | Move::DoublePawnPush(mv)
                    | Move::Piece(mv)
                    | Move::EnPassant(mv) => {
                        let (_, to) = mv.decode_into_squares();
                        legal_moves.push(to.to_u32());
                    }
                    Move::Castle(castle_mv) => {
                        let (_, to) = castle_mv.king_squares(side);
                        legal_moves.push(to.to_u32());
                    }
                    Move::Promotion(mv) => {
                        let (_, to) = mv.decode_into_squares();
                        legal_moves.push(to.to_u32());
                    }
                };
            }
        }

        legal_moves
    }

    fn engine_move(&mut self) -> String {
        let (best_move, _) = self.move_finder.get().unwrap();

        move_to_algebra(best_move, self.game.state().side_to_move())
    }
}

#[cfg(test)]
mod test {
    use super::WasmInterface;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_from_history() {
        let history = "";
        let interface = WasmInterface::from_history(history);
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

        assert_eq!(interface.game.position().to_string(), expected);
    }

    #[wasm_bindgen_test]
    fn test_from_history_2() {
        let history = "e2e4 e7e5";
        let interface = WasmInterface::from_history(history);
        let expected = unindent::unindent(
            "
                  ABCDEFGH
                8|rnbqkbnr|8
                7|pppp.ppp|7
                6|........|6
                5|....p...|5
                4|....P...|4
                3|........|3
                2|PPPP.PPP|2
                1|RNBQKBNR|1
                  ABCDEFGH
                ",
        );

        assert_eq!(interface.game.position().to_string(), expected);
    }

    #[wasm_bindgen_test]
    fn test_validate_move() {
        let history = "e2e4 e7e5";
        let mv = "d2d5";
        let is_white = true;

        let mut interface = WasmInterface::from_history(history);
        assert!(!interface.validate_move(mv, is_white))
    }

    #[wasm_bindgen_test]
    fn test_validate_move_2() {
        let history = "e2e4 a7a6 e4e5 d7d5";
        let mv = "e5d6";
        let is_white = true;

        let mut interface = WasmInterface::from_history(history);
        assert!(interface.validate_move(mv, is_white))
    }
}
