use crate::{
    eval::{eval, CHECKMATE_VAL, DRAW_VAL},
    game::Game,
    move_gen::check_legal::LegalCheckPreprocessing,
    mv::Move,
    side::Side,
};

use self::tt::{TranspositionTable, TtFlag};

mod tt;

const SEARCH_DEPTH: u8 = 6;

struct MoveEvaluator {
    tt: TranspositionTable,
    max_depth: u8,
    search_depth: u8,
}

impl MoveEvaluator {
    pub fn new() -> MoveEvaluator {
        MoveEvaluator {
            tt: TranspositionTable::new(),
            max_depth: SEARCH_DEPTH * 2,
            search_depth: SEARCH_DEPTH,
        }
    }

    pub fn get(&mut self, game: &mut Game) -> Option<(Move, i32)> {
        self.tt.update_age(game);

        let mut best_move = None;

        let mut alpha = -i32::MAX;
        let beta = i32::MAX;

        let stm = game.state().side_to_move();

        let legal_check_preprocessing = LegalCheckPreprocessing::from(game, stm);
        let pseudo_legal_mv_list = if legal_check_preprocessing.num_of_checkers() == 0 {
            game.pseudo_legal_moves(stm)
        } else {
            game.pseudo_legal_escape_moves(stm, &legal_check_preprocessing)
        };

        // let tt_move_result = self
        //     .tt
        //     .prove_move(game.state().zobrist().to_u64(), self.search_depth);

        for mv in pseudo_legal_mv_list.iter() {
            let mv = *mv;
            if !game.is_legal(mv, &legal_check_preprocessing) {
                continue;
            }

            let prev_state = game.state().encode();
            let capture = game.make_move(mv);

            let zobrist = game.state().zobrist().to_u64();
            let mut eval: i32 = 0;
            if !game.is_draw() {
                let tt_val_result = self.tt.probe_val(zobrist, self.search_depth, alpha, beta);
                eval = if let Some(tt_val) = tt_val_result {
                    tt_val
                } else {
                    -self.alpha_beta(game, self.search_depth - 1, -beta, -alpha, 1)
                };
            }

            if eval > alpha {
                alpha = eval;
                best_move = Some(mv);
                self.tt
                    .store(zobrist, self.search_depth, TtFlag::Exact, eval, Some(mv));
            } else {
                self.tt
                    .store(zobrist, self.search_depth, TtFlag::Alpha, eval, None);
            }

            game.unmake_move(mv, capture, prev_state);
        }

        self.tt.store(
            game.state().zobrist().to_u64(),
            self.search_depth,
            TtFlag::Exact,
            alpha,
            best_move,
        );
        Some((
            best_move.unwrap(),
            if stm == Side::White { alpha } else { -alpha },
        ))
    }

    fn alpha_beta(
        &self,
        game: &mut Game,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        levels_searched: u8,
    ) -> i32 {
        if depth == 0 {
            return self.quiescence(game, alpha, beta, levels_searched);
        }

        let stm = game.state().side_to_move();

        let legal_check_preprocessing = LegalCheckPreprocessing::from(game, stm);
        let pseudo_legal_mv_list = if legal_check_preprocessing.num_of_checkers() == 0 {
            game.pseudo_legal_moves(stm)
        } else {
            game.pseudo_legal_escape_moves(stm, &legal_check_preprocessing)
        };

        let mut legal_moves_available = false;

        for mv in pseudo_legal_mv_list.iter() {
            legal_moves_available = true;

            let mv = *mv;
            if !game.is_legal(mv, &legal_check_preprocessing) {
                continue;
            }

            let prev_state = game.state().encode();
            let capture = game.make_move(mv);

            let mut eval: i32 = DRAW_VAL;
            if !game.is_draw() {
                eval = -self.alpha_beta(game, depth - 1, -beta, -alpha, levels_searched + 1);
            }

            if eval >= beta {
                game.unmake_move(mv, capture, prev_state);
                return beta;
            }

            if eval > alpha {
                alpha = eval;
            }

            game.unmake_move(mv, capture, prev_state);
        }

        if !legal_moves_available && legal_check_preprocessing.num_of_checkers() > 0 {
            return -CHECKMATE_VAL + levels_searched as i32;
        } else if !legal_moves_available && DRAW_VAL > alpha {
            // is a stalemate
            return DRAW_VAL;
        }

        alpha
    }

    fn quiescence(&self, game: &mut Game, mut alpha: i32, beta: i32, levels_searched: u8) -> i32 {
        let stm = game.state().side_to_move();
        let legal_check_preprocessing = LegalCheckPreprocessing::from(game, stm);

        if levels_searched == self.max_depth {
            return eval(game, &legal_check_preprocessing, levels_searched);
        }

        if legal_check_preprocessing.num_of_checkers() > 0 {
            // go back to alpha_beta to generate escape moves
            return self.alpha_beta(game, 1, alpha, beta, levels_searched);
        }

        let pseudo_legal_mv_list = game.pseudo_legal_loud_moves(stm);

        let stand_pat = eval(game, &legal_check_preprocessing, levels_searched);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        for mv in pseudo_legal_mv_list.iter() {
            let mv = *mv;
            if !game.is_legal(mv, &legal_check_preprocessing) {
                continue;
            }

            let prev_state = game.state().encode();
            let capture = game.make_move(mv);

            let mut eval: i32 = 0;
            if !game.is_draw() {
                eval = -self.quiescence(game, -beta, -alpha, levels_searched + 1);
            }

            if eval >= beta {
                game.unmake_move(mv, capture, prev_state);
                return beta;
            }

            if eval > alpha {
                alpha = eval;
            }

            game.unmake_move(mv, capture, prev_state);
        }

        alpha
    }
}

#[cfg(test)]
pub mod test_basic_tactics {
    use crate::mv::EncodedMove;
    use crate::piece_type::PieceType;
    use crate::square::*;

    use super::*;

    #[test]
    fn pos_1() {
        let fen = "r3rk2/pb4p1/4QbBp/1p1q4/2pP4/2P5/PP3PPP/R3R1K1 w - - 0 21";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mut mv_evaluator = MoveEvaluator::new();

        let best_move_result = mv_evaluator.get(&mut game);
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
        let mut game = result.unwrap();
        let mut mv_evaluator = MoveEvaluator::new();

        let best_move_result = mv_evaluator.get(&mut game);
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
        let mut game = result.unwrap();
        let mut mv_evaluator = MoveEvaluator::new();

        let best_move_result = mv_evaluator.get(&mut game);
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
}
