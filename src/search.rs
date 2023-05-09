use crate::{
    eval::{eval, CHECKMATE_VAL, DRAW_VAL},
    game::Game,
    move_gen::check_legal::LegalCheckPreprocessing,
    move_list::MoveList,
    mv::{Decode, Move},
    piece_type::PieceType,
    side::Side,
};

use self::tt::{TranspositionTable, TtFlag};

mod tt;

const SEARCH_DEPTH: u8 = 6;
const MAX_DEPTH: u8 = 12;

const MVV_LVA: [[u8; 6]; 6] = [
    [15, 14, 13, 12, 11, 10], // victim P, attacker none, p, n , b, r, q, k
    [25, 24, 23, 22, 21, 20], // victim N, attacker none, p, n , b, r, q, k
    [35, 34, 33, 32, 31, 30], // victim B, attacker none, p, n , b, r, q, k
    [45, 44, 43, 42, 41, 40], // victim R, attacker none, p, n , b, r, q, k
    [55, 54, 53, 52, 51, 50], // victim Q, attacker none, p, n , b, r, q, k
    [0, 0, 0, 0, 0, 0],       // victim K, attacker none, p, n , b, r, q, k
];

const TT_MOVE_SORT_VAL: u8 = 60;

pub struct MoveFinder {
    tt: TranspositionTable,
    game: Game,
}

impl MoveFinder {
    pub fn new(game: Game) -> MoveFinder {
        MoveFinder {
            game,
            tt: TranspositionTable::new(),
        }
    }

    pub fn set_game(self, game: Game) -> MoveFinder {
        MoveFinder { game, tt: self.tt }
    }

    pub fn score_moves(&self, game: &Game, mv_list: &MoveList, tt_mv: Move) -> Vec<u8> {
        let mut scores = vec![0; mv_list.list().len()];

        for (i, mv) in mv_list.list().iter().enumerate() {
            if *mv == tt_mv {
                scores[i] = TT_MOVE_SORT_VAL;
            } else {
                match mv {
                    Move::King(mv) | Move::Rook(mv) | Move::Pawn(mv) | Move::Piece(mv) => {
                        if mv.is_capture() {
                            let (_, to) = mv.decode_into_squares();
                            debug_assert!(
                                game.position().at(to).is_some(),
                                "move is capture but no piece found on {}",
                                to
                            );
                            let attacker = mv.piece_type();
                            let capture = game.position().at(to).unwrap().piece_type();

                            scores[i] = MVV_LVA[capture.to_usize()][attacker.to_usize()];
                        }
                    }
                    Move::EnPassant(_) => {
                        scores[i] = MVV_LVA[PieceType::Pawn.to_usize()][PieceType::Pawn.to_usize()];
                    }
                    Move::Promotion(promote_mv) => {
                        // using mvv lva array to get difference of value between
                        // the promote piece type and the pawn
                        if promote_mv.is_capture() {
                            let (_, to) = promote_mv.decode_into_squares();
                            debug_assert!(
                                game.position().at(to).is_some(),
                                "move is capture but no piece found on {}",
                                to
                            );
                            let capture = game.position().at(to).unwrap().piece_type();
                            scores[i] = MVV_LVA[promote_mv.promote_piece_type().to_usize()]
                                [PieceType::Pawn.to_usize()]
                                + MVV_LVA[capture.to_usize()][PieceType::Pawn.to_usize()]
                        } else {
                            scores[i] = MVV_LVA[promote_mv.promote_piece_type().to_usize()]
                                [PieceType::Pawn.to_usize()]
                        }
                    }
                    Move::DoublePawnPush(_) | Move::Castle(_) => {}
                    Move::Null() => panic!("null move encountered in move list"),
                }
            }
        }

        scores
    }

    pub fn pick_move(
        &self,
        mv_list: &mut MoveList,
        scores: &mut Vec<u8>,
        start_idx: usize,
    ) -> Move {
        // finds the move with the highest score and swaps it with the item at start idx

        let mut best_score = scores[start_idx];
        let mut best_score_idx = start_idx;

        for (i, score) in scores.iter().enumerate() {
            let score = *score;
            if score > best_score {
                best_score = score;
                best_score_idx = i;
            }
        }

        scores.swap(start_idx, best_score_idx);
        mv_list.mut_list().swap(start_idx, best_score_idx);

        debug_assert!(
            mv_list.list().get(best_score_idx).is_some(),
            "there is no move at idx {} in the move list",
            best_score_idx
        );
        *mv_list.list().get(best_score_idx).unwrap()
    }

    pub fn get(&mut self) -> Option<(Move, i32)> {
        self.tt.update_age(&self.game);

        let mut best_move = None;

        let mut alpha = -i32::MAX;
        let beta = i32::MAX;

        let stm = self.game.state().side_to_move();

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut self.game, stm);
        let mut pseudo_legal_mv_list = if legal_check_preprocessing.num_of_checkers() == 0 {
            self.game.pseudo_legal_moves(stm)
        } else {
            self.game
                .pseudo_legal_escape_moves(stm, &legal_check_preprocessing)
        };

        let tt_mv_result = self
            .tt
            .probe_move(self.game.state().zobrist().to_u64(), SEARCH_DEPTH);
        let mut scores = self.score_moves(
            &self.game,
            &pseudo_legal_mv_list,
            tt_mv_result.unwrap_or(Move::Null()),
        );

        for i in 0..pseudo_legal_mv_list.list().len() {
            let mv = self.pick_move(&mut pseudo_legal_mv_list, &mut scores, i);
            if !self.game.is_legal(mv, &legal_check_preprocessing) {
                continue;
            }

            let prev_state = self.game.state().encode();
            let capture = self.game.make_move(mv);

            let zobrist = self.game.state().zobrist().to_u64();
            let eval: i32 = if self.game.is_draw() {
                DRAW_VAL
            } else {
                let tt_val_result = self.tt.probe_val(zobrist, SEARCH_DEPTH, alpha, beta);

                if let Some(tt_val) = tt_val_result {
                    tt_val
                } else {
                    -self.alpha_beta(SEARCH_DEPTH - 1, -beta, -alpha, 1)
                }
            };

            if eval > alpha {
                alpha = eval;
                best_move = Some(mv);
                self.tt
                    .store(zobrist, SEARCH_DEPTH, TtFlag::Exact, eval, Some(mv));
            } else {
                // store lower bound
                self.tt
                    .store(zobrist, SEARCH_DEPTH, TtFlag::Alpha, eval, None);
            }

            self.game.unmake_move(mv, capture, prev_state);
        }

        self.tt.store(
            self.game.state().zobrist().to_u64(),
            SEARCH_DEPTH,
            TtFlag::Exact,
            alpha,
            best_move,
        );
        Some((
            best_move.unwrap(),
            if stm == Side::White { alpha } else { -alpha },
        ))
    }

    fn alpha_beta(&mut self, depth: u8, mut alpha: i32, beta: i32, levels_searched: u8) -> i32 {
        if depth == 0 {
            // need to reverse beta and alpha bc the eval stored is from the eyes of
            // the opposing side
            let tt_val_result =
                self.tt
                    .probe_val(self.game.state().zobrist().to_u64(), depth, -beta, -alpha);
            return if let Some(tt_val) = tt_val_result {
                tt_val
            } else {
                self.quiescence(alpha, beta, levels_searched)
            };
        };

        let stm = self.game.state().side_to_move();

        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut self.game, stm);
        let mut pseudo_legal_mv_list = if legal_check_preprocessing.num_of_checkers() == 0 {
            self.game.pseudo_legal_moves(stm)
        } else {
            self.game
                .pseudo_legal_escape_moves(stm, &legal_check_preprocessing)
        };

        let tt_mv_result = self
            .tt
            .probe_move(self.game.state().zobrist().to_u64(), depth);
        let mut scores = self.score_moves(
            &self.game,
            &pseudo_legal_mv_list,
            tt_mv_result.unwrap_or(Move::Null()),
        );

        let mut legal_moves_available = false;

        for i in 0..pseudo_legal_mv_list.list().len() {
            legal_moves_available = true;

            let mv = self.pick_move(&mut pseudo_legal_mv_list, &mut scores, i);
            if !self.game.is_legal(mv, &legal_check_preprocessing) {
                continue;
            }

            let prev_state = self.game.state().encode();
            let capture = self.game.make_move(mv);

            let zobrist = self.game.state().zobrist().to_u64();
            let eval: i32 = if self.game.is_draw() {
                DRAW_VAL
            } else {
                let tt_val_result = self.tt.probe_val(zobrist, depth, alpha, beta);

                if let Some(tt_val) = tt_val_result {
                    tt_val
                } else {
                    -self.alpha_beta(depth - 1, -beta, -alpha, levels_searched + 1)
                }
            };

            if eval >= beta {
                // store upper bound for position
                self.tt.store(zobrist, depth, TtFlag::Beta, eval, Some(mv));
                self.game.unmake_move(mv, capture, prev_state);
                return beta;
            }

            if eval > alpha {
                // store exact evaluation for position
                self.tt.store(zobrist, depth, TtFlag::Exact, eval, Some(mv));
                alpha = eval;
            } else {
                // store lower bound
                self.tt.store(zobrist, depth, TtFlag::Alpha, eval, None);
            }

            self.game.unmake_move(mv, capture, prev_state);
        }

        if !legal_moves_available && legal_check_preprocessing.in_check() {
            return -CHECKMATE_VAL + levels_searched as i32;
        } else if !legal_moves_available && DRAW_VAL > alpha {
            // is a stalemate
            return DRAW_VAL;
        }

        alpha
    }

    fn quiescence(&mut self, mut alpha: i32, beta: i32, levels_searched: u8) -> i32 {
        let stm = self.game.state().side_to_move();
        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut self.game, stm);

        if levels_searched == MAX_DEPTH {
            return eval(&mut self.game, &legal_check_preprocessing, levels_searched);
        }

        if legal_check_preprocessing.in_check() {
            // need to reverse beta and alpha bc the eval stored is from the eyes of
            // the opposing side
            // go back to alpha_beta to generate escape moves
            let tt_val_result =
                self.tt
                    .probe_val(self.game.state().zobrist().to_u64(), 0, alpha, beta);

            if let Some(tt_val) = tt_val_result {
                return tt_val;
            } else {
                return self.alpha_beta(1, alpha, beta, levels_searched);
            }
        }

        let stand_pat = eval(&mut self.game, &legal_check_preprocessing, levels_searched);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut pseudo_legal_mv_list = self.game.pseudo_legal_loud_moves(stm);

        let tt_mv_result = self.tt.probe_move(self.game.state().zobrist().to_u64(), 0);
        let mut scores = self.score_moves(
            &self.game,
            &pseudo_legal_mv_list,
            tt_mv_result.unwrap_or(Move::Null()),
        );

        for i in 0..pseudo_legal_mv_list.list().len() {
            let mv = self.pick_move(&mut pseudo_legal_mv_list, &mut scores, i);
            if !self.game.is_legal(mv, &legal_check_preprocessing) {
                continue;
            }

            let prev_state = self.game.state().encode();
            let capture = self.game.make_move(mv);

            let eval: i32 = if self.game.is_draw() {
                DRAW_VAL
            } else {
                let tt_val_result =
                    self.tt
                        .probe_val(self.game.state().zobrist().to_u64(), 0, alpha, beta);

                if let Some(tt_val) = tt_val_result {
                    tt_val
                } else {
                    -self.quiescence(-beta, -alpha, levels_searched + 1)
                }
            };

            if eval >= beta {
                self.game.unmake_move(mv, capture, prev_state);
                return beta;
            }

            if eval > alpha {
                alpha = eval;
            }
            self.game.unmake_move(mv, capture, prev_state);
        }

        alpha
    }
}

#[cfg(test)]
pub mod test_basic_tactics {
    use crate::fen::STARTING_POSITION_FEN;
    use crate::mv::EncodedMove;
    use crate::piece_type::PieceType;
    use crate::{square::*, uci};

    use super::*;

    #[test]
    fn pos_1() {
        let fen = "r3rk2/pb4p1/4QbBp/1p1q4/2pP4/2P5/PP3PPP/R3R1K1 w - - 0 21";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let mut mv_evaluator = MoveFinder::new(game);

        let best_move_result = mv_evaluator.get();
        let expected = Move::Piece(EncodedMove::new(E6, E8, PieceType::Queen, true));

        assert!(best_move_result.is_some());
        let (best_move, _) = best_move_result.unwrap();
        assert_eq!(best_move, expected)
    }

    #[test]
    fn pos_2() {
        let fen = "5rk1/ppq3p1/2p3Qp/8/3P4/2P3nP/PP1N2PK/R1B5 b - - 0 28";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let mut mv_evaluator = MoveFinder::new(game);

        let best_move_result = mv_evaluator.get();
        let expected = Move::Piece(EncodedMove::new(G3, F1, PieceType::Knight, false));

        assert!(best_move_result.is_some());
        let (best_move, _) = best_move_result.unwrap();
        assert_eq!(
            best_move, expected,
            "\nbest move: {}; \nexpected: {}",
            best_move, expected
        )
    }

    #[test]
    fn mate_in_4() {
        let fen = "r1bqr2k/ppp3bp/2np2p1/8/2BnPQ2/2N2N2/PPPB1PP1/2KR3R w - - 0 0";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let mut mv_evaluator = MoveFinder::new(game);

        let best_move_result = mv_evaluator.get();
        let expected = Move::Rook(EncodedMove::new(H1, H7, PieceType::Rook, true));

        assert!(best_move_result.is_some());
        let (best_move, eval) = best_move_result.unwrap();
        assert_eq!(
            best_move, expected,
            "\nbest move: {}; eval: {}\nexpected: {}",
            best_move, eval, expected
        );
        assert_eq!(eval, CHECKMATE_VAL - 9);
    }

    #[test]
    fn debug_pos_1() {
        let mut game = Game::from_fen(STARTING_POSITION_FEN).unwrap();
        game = uci::input_position("position startpos moves e2e3 c7c6 c2c3 a7a5 a2a3 e7e6 d2d3 d7d6 g2g3 b7b6 f2f3 b6b5 e3e4 f7f6 b2b3 g7g6 h2h3 b8a6 a3a4 b5a4", game);
        let mut mv_finder = MoveFinder::new(game.clone());

        let best_move_result = mv_finder.get();
        let expected_eval = 0;

        assert!(best_move_result.is_some());
        let (best_move, eval) = best_move_result.unwrap();
        println!(
            "white score: {}, black score: {}",
            game.position().score(Side::White),
            game.position().score(Side::Black)
        );
        assert_eq!(
            expected_eval, eval,
            "\nbest move: {}; eval: {}\nexpected eval: {}",
            best_move, eval, expected_eval
        );
    }
}
