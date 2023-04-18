use crate::{bitboard::BB, mv::Decode};
use core::fmt;

use crate::{
    bitboard, fen,
    move_gen::pseudo_legal,
    move_list::MoveList,
    mv::{EncodedMove, Move, PromotionMove, KING_SIDE_CASTLE, QUEEN_SIDE_CASTLE},
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

    pub fn pseudo_legal_moves(&self, side: Side) -> () {
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
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Pawn,
                                )));
                            } else if promote_rank_bb.is_set(to) {
                                for promote_type in PROMOTE_TYPE_ARR.iter() {
                                    mv_list.push_move(if is_capture {
                                        Move::PromotionCapture(PromotionMove::new(
                                            from.to_u8(),
                                            to.to_u8(),
                                            promote_type,
                                        ))
                                    } else {
                                        Move::Promotion(PromotionMove::new(
                                            from.to_u8(),
                                            to.to_u8(),
                                            promote_type,
                                        ))
                                    })
                                }
                            } else if is_capture {
                                mv_list.push_move(Move::Capture(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    piece_type,
                                )));
                            } else {
                                mv_list.push_move(Move::Regular(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    piece_type,
                                )));
                            }
                        }
                    }
                    PieceType::Knight => {
                        let moves_bb = pseudo_legal::knight_attacks(from, friendly_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            if enemy_occupied.is_set(to) {
                                Move::Capture(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Knight,
                                ))
                            } else {
                                Move::Regular(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Knight,
                                ))
                            }
                        })
                    }
                    PieceType::Bishop => {
                        let moves_bb =
                            pseudo_legal::bishop_attacks(from, friendly_occupied, enemy_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            if enemy_occupied.is_set(to) {
                                Move::Capture(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Bishop,
                                ))
                            } else {
                                Move::Regular(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Bishop,
                                ))
                            }
                        })
                    }
                    PieceType::Rook => {
                        let moves_bb =
                            pseudo_legal::rook_attacks(from, friendly_occupied, enemy_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            if enemy_occupied.is_set(to) {
                                Move::Capture(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Rook,
                                ))
                            } else {
                                Move::Regular(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Rook,
                                ))
                            }
                        })
                    }
                    PieceType::Queen => {
                        let moves_bb =
                            pseudo_legal::queen_attacks(from, friendly_occupied, enemy_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            if enemy_occupied.is_set(to) {
                                Move::Capture(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Queen,
                                ))
                            } else {
                                Move::Regular(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::Queen,
                                ))
                            }
                        })
                    }
                    PieceType::King => {
                        let moves_bb = pseudo_legal::king_attacks(from, friendly_occupied);

                        mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                            if enemy_occupied.is_set(to) {
                                Move::Capture(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::King,
                                ))
                            } else {
                                Move::Regular(EncodedMove::new(
                                    from.to_u8(),
                                    to.to_u8(),
                                    PieceType::King,
                                ))
                            }
                        });

                        let castle_rights = self.state().castle_rights();
                        if castle_rights.can(side, castle_rights::QUEENSIDE) {
                            mv_list.push_move(Move::Castle(QUEEN_SIDE_CASTLE))
                        }
                        if castle_rights.can(side, castle_rights::KINGSIDE) {
                            mv_list.push_move(Move::Castle(KING_SIDE_CASTLE))
                        }
                    }
                }
            }
        }
    }

    // fn is_legal_move(&self, mv: EncodedMove, checkers_pinners_pinned: (BB, BB, BB)) {
    //     let (checkers, pinners, pinned) = checkers_pinners_pinned;
    //     let (from_bb, to_bb) = mv.decode_into_bb();
    //     let piece_type = mv.piece_type();
    //
    //     let num_of_checkers = checkers.count_ones();
    //     if num_of_checkers > 1 {
    //         match piece_type {
    //             PieceType::King => return (king),
    //             _ => {
    //                 return false;
    //             }
    //         }
    //     } else if num_of_checkers == 1 {
    //         match piece_type {
    //             PieceType::King => {}
    //             _ => {}
    //         }
    //     }
    //
    //     true
    // }

    // pub fn is_legal(
    //     &self,
    //     mv: Move,
    //     piece_type: PieceType,
    //     checkers_pinners_pinned: (BB, BB, BB),
    // ) -> bool {
    //     let (checkers, pinners, pinned) = checkers_pinners_pinned;
    //
    //     match mv {
    //         Move::Regular(mv) | Move::Capture(mv) => {}
    //         Move::Promotion(mv) | Move::PromotionCapture(mv) => true,
    //         Move::EnPassant(en_passant_mv) => true,
    //         Move::Castle(castle_mv) => {
    //             if checkers.not_empty() {
    //                 return false;
    //             }
    //
    //             can_castle(
    //                 &self.position,
    //                 castle_mv.decode(),
    //                 self.state.side_to_move(),
    //             )
    //         }
    //     }
    // }
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
