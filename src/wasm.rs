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

#[wasm_bindgen]
struct ChessInterface {
    game: Game,
    move_finder: MoveFinder,
}

impl ChessInterface {
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
}

#[wasm_bindgen]
impl ChessInterface {
    pub fn from_history(history: &str) -> ChessInterface {
        let game = input_position(
            &format!("position startpos {}", history),
            Game::from_fen(STARTING_POSITION_FEN).unwrap(),
        );

        ChessInterface {
            game: game.clone(),
            move_finder: MoveFinder::new(game),
        }
    }

    pub fn validate_mv(&mut self, move_notation: &str, side: bool) -> bool {
        let side = if side == true {
            Side::White
        } else {
            Side::Black
        };
        let mv_result = algebra_to_move(move_notation, &self.game);
        assert!(mv_result.is_ok(), "{} is not a valid move", move_notation);
        let mv = mv_result.unwrap();

        let mut game = self.game.clone();
        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        game.is_legal(mv, &legal_check_preprocessing)
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
