use core::fmt;

use crate::{
    bitboard, fen,
    move_gen::{
        check_legal::{is_legal_castle, is_legal_regular_move, LegalCheckPreprocessing},
        pseudo_legal,
    },
    move_list::MoveList,
    mv::{castle::Castle, Decode, EncodedMove, Move, PromotionMove},
    piece::Piece,
    piece_type::{PieceType, PIECE_TYPE_MAP, PROMOTE_TYPE_ARR},
    side::Side,
    square,
    state::position::Position,
    state::{castle_rights, State},
};

pub struct Game {
    position: Position,
    state: State,
}

impl Game {
    pub fn from_fen(fen: &str) -> Result<Game, String> {
        let (position, state) = fen::load_fen(&fen)?;

        Ok(Game { position, state })
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn pseudo_legal_moves(&self, side: Side) -> MoveList {
        let friendly_occupied = self.position().bb_sides()[side.to_usize()];
        let enemy_occupied = self.position().bb_sides()[side.opposite().to_usize()];

        let mut mv_list = MoveList::new();
        for (i, piece_bb) in self.position().bb_pieces().iter().enumerate() {
            let piece_type = PIECE_TYPE_MAP[i];
            let piece_bb_iter = (*piece_bb & self.position().bb_sides()[side.to_usize()]).iter();

            for from in piece_bb_iter {
                match piece_type {
                    PieceType::Pawn => {
                        let moves_bb = pseudo_legal::pawn(
                            from,
                            friendly_occupied,
                            enemy_occupied,
                            self.state().en_passant(),
                            side,
                        );

                        let promote_rank_bb = if side == Side::White {
                            bitboard::ROW_8
                        } else {
                            bitboard::ROW_1
                        };

                        for to in moves_bb.iter() {
                            let is_capture = enemy_occupied.is_set(to);

                            if to == self.state().en_passant().unwrap_or(square::NULL) {
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
                            } else {
                                mv_list.push_move(Move::Piece(EncodedMove::new(
                                    from,
                                    to,
                                    PieceType::Pawn,
                                    is_capture,
                                )));
                            }
                        }
                    }
                    PieceType::Knight => {
                        let moves_bb = pseudo_legal::knight_attacks(from, friendly_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            Move::Piece(EncodedMove::new(
                                from,
                                to,
                                PieceType::Knight,
                                enemy_occupied.is_set(to),
                            ))
                        })
                    }
                    PieceType::Bishop => {
                        let moves_bb =
                            pseudo_legal::bishop_attacks(from, friendly_occupied, enemy_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            Move::Piece(EncodedMove::new(
                                from,
                                to,
                                PieceType::Bishop,
                                enemy_occupied.is_set(to),
                            ))
                        })
                    }
                    PieceType::Rook => {
                        let moves_bb =
                            pseudo_legal::rook_attacks(from, friendly_occupied, enemy_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            Move::Piece(EncodedMove::new(
                                from,
                                to,
                                PieceType::Rook,
                                enemy_occupied.is_set(to),
                            ))
                        })
                    }
                    PieceType::Queen => {
                        let moves_bb =
                            pseudo_legal::queen_attacks(from, friendly_occupied, enemy_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            Move::Piece(EncodedMove::new(
                                from,
                                to,
                                PieceType::Queen,
                                enemy_occupied.is_set(to),
                            ))
                        })
                    }
                    PieceType::King => {
                        let moves_bb = pseudo_legal::king_attacks(from, friendly_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            Move::King(EncodedMove::new(
                                from,
                                to,
                                PieceType::King,
                                enemy_occupied.is_set(to),
                            ))
                        });

                        let castle_rights = self.state().castle_rights();
                        if castle_rights.can(side, castle_rights::QUEENSIDE) {
                            mv_list.push_move(Move::Castle(Castle::QueenSide))
                        }
                        if castle_rights.can(side, castle_rights::KINGSIDE) {
                            mv_list.push_move(Move::Castle(Castle::KingSide))
                        }
                    }
                }
            }
        }

        mv_list
    }

    fn make_en_passant_move(&mut self, mv: EncodedMove) -> Option<Piece> {
        let side = self.state.side_to_move();
        let (from, to) = mv.decode_into_squares();

        self.position.move_piece(PieceType::Pawn, from, to, side);
        let en_passant_capture_sq = self
            .state
            .en_passant_capture_sq()
            .expect("made en passant capture when there was no en passant square");
        let capture = self.position.remove_at(en_passant_capture_sq);

        capture
    }

    fn unmake_en_passant_move(&mut self, mv: EncodedMove, captured_side: Side) {
        let side = self.state.side_to_move();
        let (from, to) = mv.decode_into_squares();

        self.position.move_piece(PieceType::Pawn, to, from, side);

        let en_passant_capture_sq = self
            .state
            .en_passant_capture_sq()
            .expect("unwrapped en passant square when there was no en passant square");
        self.position
            .place_piece(PieceType::Pawn, en_passant_capture_sq, captured_side);
    }

    fn is_legal_en_passant_move(&mut self, mv: EncodedMove) -> bool {
        let side = self.state.side_to_move();
        self.make_en_passant_move(mv);
        let check = self.position.in_check(side);
        self.unmake_en_passant_move(mv, side.opposite());

        !check
    }

    pub fn is_legal(
        &mut self,
        mv: Move,
        legal_check_preprocessing: &LegalCheckPreprocessing,
    ) -> bool {
        match mv {
            Move::King(king_mv) => is_legal_regular_move(
                &self.position,
                king_mv,
                true,
                self.state.side_to_move(),
                &legal_check_preprocessing,
            ),
            Move::Piece(piece_mv) => is_legal_regular_move(
                &self.position,
                piece_mv,
                false,
                self.state.side_to_move(),
                &legal_check_preprocessing,
            ),
            Move::Promotion(promotion_mv) => is_legal_regular_move(
                &self.position,
                promotion_mv,
                false,
                self.state.side_to_move(),
                &legal_check_preprocessing,
            ),
            Move::EnPassant(en_passant_mv) => self.is_legal_en_passant_move(en_passant_mv),
            Move::Castle(castle) => is_legal_castle(
                &self.position,
                castle,
                self.state.side_to_move(),
                legal_check_preprocessing.attacked_squares_bb(),
                legal_check_preprocessing.checkers(),
            ),
        }
    }

    fn make_regular_move(&mut self, mv: EncodedMove) -> Option<Piece> {
        let side = self.state.side_to_move();
        let piece_type = mv.piece_type();
        let (from, to) = mv.decode_into_squares();

        let mut capture = None;
        if mv.is_capture() {
            capture = self.position.remove_at(to);
        }
        self.position.move_piece(piece_type, from, to, side);

        capture
    }

    fn make_castle_move(&mut self, castle: Castle) {
        let side = self.state.side_to_move();

        let (king_from, king_to) = castle.king_squares(side);
        let (rook_from, rook_to) = castle.rook_squares(side);

        self.position
            .move_piece(PieceType::King, king_from, king_to, side);
        self.position
            .move_piece(PieceType::Rook, rook_from, rook_to, side);
    }

    fn make_promotion_move(&mut self, mv: PromotionMove) -> Option<Piece> {
        let side = self.state.side_to_move();
        let (from, to) = mv.decode_into_squares();

        let mut capture = None;
        if mv.is_capture() {
            capture = self.position.remove_at(to);
        }
        self.position.remove_piece(PieceType::Pawn, from, side);
        self.position.place_piece(mv.promote_piece_type(), to, side);

        capture
    }

    pub fn adjust_castle_rights_on_rook_move(&mut self, mv: EncodedMove) {
        let side = self.state.side_to_move();
        let (rook_queenside_sq, rook_kingside_sq) = side.rook_start_squares();
        let (from, _) = mv.decode_into_squares();
        if from == rook_kingside_sq {
            self.state.remove_castle_rights(side, Castle::QueenSide);
        } else if from == rook_queenside_sq {
            self.state.remove_castle_rights(side, Castle::KingSide);
        }
    }

    pub fn adjust_castle_rights_on_king_move(&mut self) {
        let side = self.state.side_to_move();
        self.state.remove_castle_rights(side, Castle::KingSide);
        self.state.remove_castle_rights(side, Castle::QueenSide);
    }

    pub fn make_move(&mut self, mv: Move) -> Option<Piece> {
        match mv {
            Move::King(mv) => {
                self.adjust_castle_rights_on_king_move();
                self.make_regular_move(mv)
            }
            Move::Piece(mv) => {
                if mv.piece_type() == PieceType::Rook {
                    self.adjust_castle_rights_on_rook_move(mv)
                };
                self.make_regular_move(mv)
            }
            Move::Castle(castle_mv) => {
                self.adjust_castle_rights_on_king_move();
                self.make_castle_move(castle_mv);
                None
            }
            Move::Promotion(promotion_mv) => self.make_promotion_move(promotion_mv),
            Move::EnPassant(en_passant_mv) => self.make_en_passant_move(en_passant_mv),
        }
    }

    pub fn unmake_regular_move(&mut self, mv: EncodedMove, capture: Option<Piece>) {
        let side = self.state.side_to_move();
        let piece_type = mv.piece_type();
        let (from, to) = mv.decode_into_squares();

        if mv.is_capture() {
            let (capture_side, capture_pc) = capture
                .expect("capture is true, but unmake function was not giving a Piece")
                .decode();
            self.position.place_piece(capture_pc, to, capture_side);
        }
        self.position.move_piece(piece_type, to, from, side);
    }

    fn unmake_castle_move(&mut self, castle: Castle) {
        let side = self.state.side_to_move();

        let (king_from, king_to) = castle.king_squares(side);
        let (rook_from, rook_to) = castle.rook_squares(side);

        self.position
            .move_piece(PieceType::King, king_to, king_from, side);
        self.position
            .move_piece(PieceType::Rook, rook_to, rook_from, side);
    }

    fn unmake_promotion_move(&mut self, mv: PromotionMove, capture: Option<Piece>) {
        let side = self.state.side_to_move();
        let (from, to) = mv.decode_into_squares();

        if mv.is_capture() {
            let (capture_side, capture_pc) = capture
                .expect("capture is true, but unmake function was not giving a Piece")
                .decode();
            self.position.place_piece(capture_pc, to, capture_side);
        }

        self.position.place_piece(PieceType::Pawn, from, side);
        self.position
            .remove_piece(mv.promote_piece_type(), to, side);
    }

    pub fn unmake_move(&mut self, mv: Move, capture: Option<Piece>) {
        match mv {
            Move::King(mv) | Move::Piece(mv) => {
                self.unmake_regular_move(mv, capture);
            }
            Move::Castle(castle_mv) => {
                self.unmake_castle_move(castle_mv);
            }
            Move::Promotion(promotion_mv) => {
                self.unmake_promotion_move(promotion_mv, capture);
            }
            Move::EnPassant(en_passant_mv) => {
                self.unmake_en_passant_move(en_passant_mv, self.state.side_to_move().opposite());
            }
        };
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let position_str = self.position.to_string();
        let state_str = self.state.to_string();

        write!(f, "{}\n{}", position_str, state_str)
    }
}

#[cfg(test)]
pub mod test_fen {
    use super::*;
    use crate::square::*;
    use crate::state::castle_rights;
    use unindent;

    #[test]
    fn empty_fen() {
        let fen = "";
        assert_eq!(true, Game::from_fen(fen).is_err());
    }
    #[test]
    fn invalid_fen_board() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNRz w KQkq - 0 0";
        assert_eq!(true, Game::from_fen(fen).is_err());
    }
    #[test]
    fn invalid_color() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR z KQkq - 0 0";
        assert_eq!(true, Game::from_fen(fen).is_err());
    }
    #[test]
    fn invalid_castle_rights() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KaQkq f10 0 0";
        assert_eq!(true, Game::from_fen(fen).is_err());
    }
    #[test]
    fn invalid_en_passant_sq() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f10 0 0";
        assert_eq!(true, Game::from_fen(fen).is_err());
    }
    #[test]
    fn invalid_halfmoves() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f1 a 0";
        assert_eq!(true, Game::from_fen(fen).is_err());
    }
    #[test]
    fn invalid_fullmoves() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq f1 0 a";
        assert_eq!(true, Game::from_fen(fen).is_err());
    }

    #[test]
    fn parse_parse_with_starting_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
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

                println!("{}", game.position.to_string());
                assert_eq!(game.position.to_string(), expected);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_default_state() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(
                    game.state().castle_rights().to_string(),
                    (castle_rights::WHITE | castle_rights::BLACK).to_string()
                );
                assert_eq!(game.state().halfmoves(), 0);
                assert_eq!(game.state().fullmoves(), 0);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_parse_with_random_fen() {
        let fen = "8/8/7p/3KNN1k/2p4p/8/3P2p1/8 w - - 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                let expected = unindent::unindent(
                    "
              ABCDEFGH
            8|........|8
            7|........|7
            6|.......p|6
            5|...KNN.k|5
            4|..p....p|4
            3|........|3
            2|...P..p.|2
            1|........|1
              ABCDEFGH
        ",
                );

                println!("{}", game.position.to_string());
                assert_eq!(game.position.to_string(), expected);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_stm_1() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().side_to_move(), Side::White);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_stm_2() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().side_to_move(), Side::Black);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_ep_square_1() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().en_passant(), None);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_ep_square_2() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq c3 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().en_passant(), Some(C3));
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_half_move_clock_1() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq c3 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().halfmoves(), 0);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_half_move_clock_2() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq c3 23 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().halfmoves(), 23);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_full_move_number_1() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq c3 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().fullmoves(), 0);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_full_move_number_2() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq c3 0 45";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().fullmoves(), 45);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_castling_rights_1() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b - c3 0 45";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(game.state().castle_rights(), castle_rights::NONE);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_castling_rights_2() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b Kq c3 0 45";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                let mut expected =
                    castle_rights::NONE.set(castle_rights::KINGSIDE & castle_rights::WHITE);
                expected = expected.set(castle_rights::QUEENSIDE & castle_rights::BLACK);
                assert_eq!(game.state().castle_rights(), expected);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }

    #[test]
    fn parse_with_castling_rights_3() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq c3 0 45";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                assert_eq!(
                    game.state().castle_rights(),
                    (castle_rights::WHITE | castle_rights::BLACK)
                );
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }
}

#[cfg(test)]
pub mod test_is_legal_en_passant {
    use super::*;
    use crate::square::*;

    #[test]
    fn legal() {
        let fen = "4k3/8/8/4Pp2/8/8/8/4K3 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E5;
        let to = F6;
        let mv = EncodedMove::new(from, to, PieceType::Pawn, true);

        assert_eq!(game.is_legal_en_passant_move(mv), true);
    }

    #[test]
    fn pinned_on_file() {
        let fen = "4k3/4r3/8/4Pp2/8/8/8/4K3 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E5;
        let to = F6;
        let mv = EncodedMove::new(from, to, PieceType::Pawn, true);

        assert_eq!(game.is_legal_en_passant_move(mv), false);
    }

    #[test]
    fn pinned_on_file_2() {
        let fen = "4k3/5r2/8/4Pp2/8/8/5K2/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E5;
        let to = F6;
        let mv = EncodedMove::new(from, to, PieceType::Pawn, true);

        assert_eq!(game.is_legal_en_passant_move(mv), true);
    }

    #[test]
    fn pinned_on_diagonal_1() {
        let fen = "4k3/2b5/8/4Pp2/5K2/8/8/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E5;
        let to = F6;
        let mv = EncodedMove::new(from, to, PieceType::Pawn, true);

        assert_eq!(game.is_legal_en_passant_move(mv), false);
    }

    #[test]
    fn pinned_on_diagonal_2() {
        let fen = "4k3/3b4/8/4Pp2/6K1/8/8/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E5;
        let to = F6;
        let mv = EncodedMove::new(from, to, PieceType::Pawn, true);

        assert_eq!(game.is_legal_en_passant_move(mv), false);
    }

    #[test]
    fn pinned_on_rank_1() {
        let fen = "4k3/8/8/1r2PpK1/8/8/8/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E5;
        let to = F6;
        let mv = EncodedMove::new(from, to, PieceType::Pawn, true);

        assert_eq!(game.is_legal_en_passant_move(mv), false);
    }

    #[test]
    fn pinned_on_rank_2() {
        let fen = "4k3/8/8/3KPpr1/8/8/8/8 w - f6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E5;
        let to = F6;
        let mv = EncodedMove::new(from, to, PieceType::Pawn, true);

        assert_eq!(game.is_legal_en_passant_move(mv), false);
    }
}
#[cfg(test)]
pub mod test_make_move {
    use super::*;
    use crate::{piece_type::PromoteType, square::*};

    #[test]
    fn regular_move() {
        let fen = "5k2/8/8/8/8/8/4Q3/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E2;
        let to = F2;
        let mv = Move::Piece(EncodedMove::new(from, to, PieceType::Queen, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(!game.position.bb_pc(PieceType::Queen, side).is_set(from));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
    }

    #[test]
    fn capture_move() {
        let fen = "4k3/3q4/8/8/B7/8/8/4K3 b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = D7;
        let to = A4;
        let mv = Move::Piece(EncodedMove::new(from, to, PieceType::Queen, true));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(!game.position.bb_pc(PieceType::Queen, side).is_set(from));
        assert!(!game
            .position
            .bb_pc(PieceType::Bishop, side.opposite())
            .is_set(to));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
        assert!(!game.position.bb_side(side.opposite()).is_set(to));
    }

    #[test]
    fn promotion_move_1() {
        let fen = "4k3/7P/8/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = H7;
        let to = H8;
        let mv = Move::Promotion(PromotionMove::new(from, to, &PromoteType::Queen, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(!game.position.bb_pc(PieceType::Pawn, side).is_set(from));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
    }

    #[test]
    fn promotion_move_with_capture() {
        let fen = "4k1b1/7P/8/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = H7;
        let to = G8;
        let mv = Move::Promotion(PromotionMove::new(from, to, &PromoteType::Queen, true));
        let side = game.state.side_to_move();
        let capture_piece_type = PieceType::Bishop;
        game.make_move(mv);

        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(!game.position.bb_pc(PieceType::Pawn, side).is_set(from));
        assert!(!game
            .position
            .bb_pc(capture_piece_type, side.opposite())
            .is_set(from));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
        assert!(!game.position.bb_side(side.opposite()).is_set(to));
    }

    #[test]
    fn en_passant_move() {
        let fen = "4k3/8/8/5Pp1/8/8/8/4K3 w - g6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = F5;
        let to = G6;
        let moving_piece_type = PieceType::Pawn;
        let capture_piece_type = PieceType::Pawn;
        let mv = Move::EnPassant(EncodedMove::new(from, to, moving_piece_type, true));
        let side = game.state.side_to_move();
        game.make_move(mv);

        let en_passant_capture_sq = game.state().en_passant_capture_sq().unwrap();
        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(moving_piece_type, side).is_set(to));
        assert!(!game.position.bb_pc(moving_piece_type, side).is_set(from));
        assert!(!game
            .position
            .bb_pc(capture_piece_type, side.opposite())
            .is_set(en_passant_capture_sq));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
        assert!(!game
            .position
            .bb_side(side.opposite())
            .is_set(en_passant_capture_sq));
    }

    #[test]
    fn castle_move_w_kingside() {
        let fen = "4k3/8/8/8/8/8/8/4K2R w K - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::KingSide);
        let side = game.state.side_to_move();
        game.make_move(mv);

        let king_after = F1;
        let king_before = E1;
        let rook_after = G1;
        let rook_before = H1;

        assert!(game.position.at(king_after).is_some());
        assert!(game.position.at(rook_before).is_none());
        assert!(game.position.at(rook_after).is_some());
        assert!(game.position.at(king_before).is_none());
        assert!(game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(rook_after));
        assert!(game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(king_after));
        assert!(!game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_before));
        assert!(!game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_before));
        assert!(game.position.bb_side(side).is_set(rook_after));
        assert!(game.position.bb_side(side).is_set(king_after));
        assert!(!game.position.bb_side(side).is_set(king_before));
        assert!(!game.position.bb_side(side).is_set(rook_before));
    }

    #[test]
    fn castle_move_w_queenside() {
        let fen = "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::QueenSide);
        let side = game.state.side_to_move();
        game.make_move(mv);

        let king_after = C1;
        let king_before = E1;
        let rook_after = D1;
        let rook_before = A1;

        assert!(game.position.at(rook_after).is_some());
        assert!(game.position.at(rook_before).is_none());
        assert!(game.position.at(king_after).is_some());
        assert!(game.position.at(king_before).is_none());
        assert!(game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_after));
        assert!(game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_after));
        assert!(!game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_before));
        assert!(!game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_before));
        assert!(game.position.bb_side(side).is_set(rook_after));
        assert!(game.position.bb_side(side).is_set(king_after));
        assert!(!game.position.bb_side(side).is_set(king_before));
        assert!(!game.position.bb_side(side).is_set(rook_before));
    }

    #[test]
    fn castle_move_b_kingside() {
        let fen = "r3k2r/8/8/8/8/8/8/4K3 b k - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::KingSide);
        let side = game.state.side_to_move();
        game.make_move(mv);

        let king_after = F8;
        let king_before = E8;
        let rook_after = G8;
        let rook_before = H8;

        assert!(game.position.at(king_after).is_some());
        assert!(game.position.at(rook_before).is_none());
        assert!(game.position.at(rook_after).is_some());
        assert!(game.position.at(king_before).is_none());
        assert!(game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(rook_after));
        assert!(game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(king_after));
        assert!(!game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_before));
        assert!(!game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_before));
        assert!(game.position.bb_side(side).is_set(rook_after));
        assert!(game.position.bb_side(side).is_set(king_after));
        assert!(!game.position.bb_side(side).is_set(king_before));
        assert!(!game.position.bb_side(side).is_set(rook_before));
    }

    #[test]
    fn castle_move_b_queenside() {
        let fen = "r3k2r/8/8/8/8/8/8/4K3 b q - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::QueenSide);
        let side = game.state.side_to_move();
        game.make_move(mv);

        let king_after = C8;
        let king_before = E8;
        let rook_after = D8;
        let rook_before = A8;

        assert!(game.position.at(rook_after).is_some());
        assert!(game.position.at(rook_before).is_none());
        assert!(game.position.at(king_after).is_some());
        assert!(game.position.at(king_before).is_none());
        assert!(game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_after));
        assert!(game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_after));
        assert!(!game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_before));
        assert!(!game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_before));
        assert!(game.position.bb_side(side).is_set(rook_after));
        assert!(game.position.bb_side(side).is_set(king_after));
        assert!(!game.position.bb_side(side).is_set(king_before));
        assert!(!game.position.bb_side(side).is_set(rook_before));
    }
}

#[cfg(test)]
pub mod test_unmake_move {
    use super::*;
    use crate::{piece_type::PromoteType, square::*};

    #[test]
    fn regular_move() {
        let fen = "5k2/8/8/8/8/8/4Q3/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E2;
        let to = F2;
        let mv = Move::Piece(EncodedMove::new(from, to, PieceType::Queen, false));
        let side = game.state.side_to_move();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_none());
        assert!(!game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(game.position.bb_pc(PieceType::Queen, side).is_set(from));
        assert!(!game.position.bb_side(side).is_set(to));
        assert!(game.position.bb_side(side).is_set(from));
    }

    #[test]
    fn capture_move() {
        let fen = "4k3/3q4/8/8/B7/8/8/4K3 b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = D7;
        let to = A4;
        let mv = Move::Piece(EncodedMove::new(from, to, PieceType::Queen, true));
        let side = game.state.side_to_move();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_none());
        assert!(!game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(game.position.bb_pc(PieceType::Queen, side).is_set(from));
        assert!(game
            .position
            .bb_pc(PieceType::Bishop, side.opposite())
            .is_set(to));
        assert!(!game.position.bb_side(side).is_set(to));
        assert!(game.position.bb_side(side).is_set(from));
        assert!(game.position.bb_side(side.opposite()).is_set(to));
    }

    #[test]
    fn promotion_move_1() {
        let fen = "4k3/7P/8/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = H7;
        let to = H8;
        let mv = Move::Promotion(PromotionMove::new(from, to, &PromoteType::Queen, false));
        let side = game.state.side_to_move();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_none());
        assert!(!game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(game.position.bb_pc(PieceType::Pawn, side).is_set(from));
        assert!(!game.position.bb_side(side).is_set(to));
        assert!(game.position.bb_side(side).is_set(from));
    }

    #[test]
    fn promotion_move_with_capture() {
        let fen = "4k1b1/7P/8/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = H7;
        let to = G8;
        let mv = Move::Promotion(PromotionMove::new(from, to, &PromoteType::Queen, true));
        let side = game.state.side_to_move();
        let capture_piece_type = PieceType::Bishop;
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_none());
        assert!(!game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(game.position.bb_pc(PieceType::Pawn, side).is_set(from));
        assert!(game
            .position
            .bb_pc(capture_piece_type, side.opposite())
            .is_set(to));
        assert!(!game.position.bb_side(side).is_set(to));
        assert!(game.position.bb_side(side).is_set(from));
        assert!(game.position.bb_side(side.opposite()).is_set(to));
    }

    #[test]
    fn en_passant_move() {
        let fen = "4k3/8/8/5Pp1/8/8/8/4K3 w - g6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = F5;
        let to = G6;
        let moving_piece_type = PieceType::Pawn;
        let capture_piece_type = PieceType::Pawn;
        let mv = Move::EnPassant(EncodedMove::new(from, to, moving_piece_type, true));
        let side = game.state.side_to_move();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        let en_passant_capture_sq = game.state().en_passant_capture_sq().unwrap();
        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_none());
        assert!(!game.position.bb_pc(moving_piece_type, side).is_set(to));
        assert!(game.position.bb_pc(moving_piece_type, side).is_set(from));
        assert!(game
            .position
            .bb_pc(capture_piece_type, side.opposite())
            .is_set(en_passant_capture_sq));
        assert!(!game.position.bb_side(side).is_set(to));
        assert!(game.position.bb_side(side).is_set(from));
        assert!(game
            .position
            .bb_side(side.opposite())
            .is_set(en_passant_capture_sq));
    }

    #[test]
    fn castle_move_w_kingside() {
        let fen = "4k3/8/8/8/8/8/8/4K2R w K - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::KingSide);
        let side = game.state.side_to_move();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        let king_after = F1;
        let king_before = E1;
        let rook_after = G1;
        let rook_before = H1;

        assert!(game.position.at(king_before).is_some());
        assert!(game.position.at(king_after).is_none());
        assert!(game.position.at(rook_before).is_some());
        assert!(game.position.at(rook_after).is_none());
        assert!(!game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(rook_after));
        assert!(!game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(king_after));
        assert!(game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_before));
        assert!(game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_before));
        assert!(!game.position.bb_side(side).is_set(rook_after));
        assert!(!game.position.bb_side(side).is_set(king_after));
        assert!(game.position.bb_side(side).is_set(king_before));
        assert!(game.position.bb_side(side).is_set(rook_before));
    }

    #[test]
    fn castle_move_w_queenside() {
        let fen = "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::QueenSide);
        let side = game.state.side_to_move();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        let king_after = C1;
        let king_before = E1;
        let rook_after = D1;
        let rook_before = A1;

        assert!(game.position.at(king_before).is_some());
        assert!(game.position.at(king_after).is_none());
        assert!(game.position.at(rook_before).is_some());
        assert!(game.position.at(rook_after).is_none());
        assert!(!game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(rook_after));
        assert!(!game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(king_after));
        assert!(game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_before));
        assert!(game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_before));
        assert!(!game.position.bb_side(side).is_set(rook_after));
        assert!(!game.position.bb_side(side).is_set(king_after));
        assert!(game.position.bb_side(side).is_set(king_before));
        assert!(game.position.bb_side(side).is_set(rook_before));
    }

    #[test]
    fn castle_move_b_kingside() {
        let fen = "r3k2r/8/8/8/8/8/8/4K3 b k - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::KingSide);
        let side = game.state.side_to_move();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        let king_after = F8;
        let king_before = E8;
        let rook_after = G8;
        let rook_before = H8;

        assert!(game.position.at(king_before).is_some());
        assert!(game.position.at(king_after).is_none());
        assert!(game.position.at(rook_before).is_some());
        assert!(game.position.at(rook_after).is_none());
        assert!(!game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(rook_after));
        assert!(!game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(king_after));
        assert!(game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_before));
        assert!(game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_before));
        assert!(!game.position.bb_side(side).is_set(rook_after));
        assert!(!game.position.bb_side(side).is_set(king_after));
        assert!(game.position.bb_side(side).is_set(king_before));
        assert!(game.position.bb_side(side).is_set(rook_before));
    }

    #[test]
    fn castle_move_b_queenside() {
        let fen = "r3k2r/8/8/8/8/8/8/4K3 b q - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::QueenSide);
        let side = game.state.side_to_move();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture);

        let king_after = C8;
        let king_before = E8;
        let rook_after = D8;
        let rook_before = A8;

        assert!(game.position.at(king_before).is_some());
        assert!(game.position.at(king_after).is_none());
        assert!(game.position.at(rook_before).is_some());
        assert!(game.position.at(rook_after).is_none());
        assert!(!game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(rook_after));
        assert!(!game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(king_after));
        assert!(game
            .position
            .bb_pc(PieceType::King, side)
            .is_set(king_before));
        assert!(game
            .position
            .bb_pc(PieceType::Rook, side)
            .is_set(rook_before));
        assert!(!game.position.bb_side(side).is_set(rook_after));
        assert!(!game.position.bb_side(side).is_set(king_after));
        assert!(game.position.bb_side(side).is_set(king_before));
        assert!(game.position.bb_side(side).is_set(rook_before));
    }
}
