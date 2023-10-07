use core::fmt;

use crate::{
    bitboard::squares_between::bb_squares_between,
    fen,
    move_gen::{
        check_legal::{
            is_legal_castle, is_legal_en_passant_move, is_legal_king_move, is_legal_regular_move,
            LegalCheckPreprocessing,
        },
        escape_check,
    },
    move_list::MoveList,
    mv::{castle::Castle, Decode, EncodedMove, Move, PromotionMove},
    piece::Piece,
    piece_type::{PieceType, PIECE_TYPE_MAP},
    side::Side,
    square::Square,
    state::position::Position,
    state::{EncodedState, State},
};

#[derive(Clone)]
pub struct Game {
    position: Position,
    state: State,
}

impl Game {
    pub fn from_fen(fen: &str) -> Result<Game, String> {
        let (position, state) = fen::load_fen(fen)?;

        Ok(Game { position, state })
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn mut_position(&mut self) -> &mut Position {
        &mut self.position
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn pseudo_legal_moves(&self, side: Side) -> MoveList {
        let friendly_occupied = self.position().bb_side(side);
        let enemy_occupied = self.position().bb_side(side.opposite());

        let mut mv_list = MoveList::new();
        for (i, piece_bb) in self.position().bb_pieces().iter().enumerate() {
            let piece_type = PIECE_TYPE_MAP[i];
            let piece_bb_iter = (*piece_bb & self.position().bb_side(side)).iter();

            for from in piece_bb_iter {
                let moves_bb = piece_type.pseudo_legal_moves_bb(
                    from,
                    friendly_occupied,
                    enemy_occupied,
                    self.state(),
                    side,
                    self.state().en_passant(),
                );
                piece_type.push_bb_to_move_list(
                    &mut mv_list,
                    moves_bb,
                    from,
                    side,
                    enemy_occupied,
                    self.state(),
                    self.state().en_passant(),
                )
            }
        }

        let castle_rights = self.state().castle_rights();
        if castle_rights.can(side, Castle::Queenside) {
            mv_list.push_move(Move::Castle(Castle::Queenside))
        }
        if castle_rights.can(side, Castle::Kingside) {
            mv_list.push_move(Move::Castle(Castle::Kingside))
        }

        mv_list
    }

    pub fn pseudo_legal_escape_moves(
        &self,
        side: Side,
        legal_check_preprocessing: &LegalCheckPreprocessing,
    ) -> MoveList {
        let num_of_checkers = legal_check_preprocessing.num_of_checkers();
        debug_assert!(
            num_of_checkers > 0,
            "running escape_moves when there are no checks"
        );

        let friendly_occupied = self.position().bb_side(side);
        let enemy_occupied = self.position().bb_side(side.opposite());

        let mut mv_list = MoveList::new();

        if num_of_checkers > 1 {
            let from = self.position.king_sq(side);
            let moves_bb = escape_check::king(from, friendly_occupied, legal_check_preprocessing);

            mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                let encoded_mv =
                    EncodedMove::new(from, to, PieceType::King, enemy_occupied.is_set(to));
                Move::King(encoded_mv)
            });
        } else {
            for (i, piece_bb) in self.position().bb_pieces().iter().enumerate() {
                let piece_type = PIECE_TYPE_MAP[i];
                let piece_bb_iter = (*piece_bb & self.position().bb_side(side)).iter();

                let checker_sq = legal_check_preprocessing.checkers().bitscan();
                let check_ray = bb_squares_between(self.position.king_sq(side), checker_sq)
                    | legal_check_preprocessing.checkers();

                for from in piece_bb_iter {
                    let moves_bb = piece_type.pseudo_legal_escape_moves_bb(
                        from,
                        friendly_occupied,
                        enemy_occupied,
                        side,
                        self.state().en_passant(),
                        legal_check_preprocessing,
                        check_ray,
                    );
                    piece_type.push_bb_to_move_list(
                        &mut mv_list,
                        moves_bb,
                        from,
                        side,
                        enemy_occupied,
                        self.state(),
                        self.state().en_passant(),
                    );
                }
            }
        }

        mv_list
    }

    pub fn pseudo_legal_loud_moves(&self, side: Side) -> MoveList {
        let friendly_occupied = self.position().bb_side(side);
        let enemy_occupied = self.position().bb_side(side.opposite());

        let mut mv_list = MoveList::new();
        for (i, piece_bb) in self.position().bb_pieces().iter().enumerate() {
            let piece_type = PIECE_TYPE_MAP[i];
            let piece_bb_iter = (*piece_bb & self.position().bb_sides()[side.to_usize()]).iter();

            for from in piece_bb_iter {
                let moves_bb = piece_type.pseudo_legal_loud_moves_bb(
                    from,
                    friendly_occupied,
                    enemy_occupied,
                    self.state(),
                    side,
                    self.state().en_passant(),
                );
                piece_type.push_bb_to_move_list(
                    &mut mv_list,
                    moves_bb,
                    from,
                    side,
                    enemy_occupied,
                    self.state(),
                    self.state().en_passant(),
                );
            }
        }

        mv_list
    }

    fn make_en_passant_move(&mut self, mv: EncodedMove, side: Side) -> Option<Piece> {
        let (from, to) = mv.decode_into_squares();

        let en_passant_capture_sq = self
            .state
            .en_passant_capture_sq()
            .expect("made en passant capture when there was no en passant square");
        let capture = self.position.remove_at(en_passant_capture_sq);
        debug_assert!(
            capture.is_some(),
            "\nmove is en passant but no piece found on capture square\nmove: {}\n{}",
            mv,
            self.position()
        );
        self.state.mut_zobrist().hash_piece(
            side.opposite(),
            PieceType::Pawn,
            en_passant_capture_sq,
        );

        self.position.move_piece(PieceType::Pawn, from, to, side);

        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Pawn, from);
        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Pawn, to);

        capture
    }

    fn unmake_en_passant_move(&mut self, mv: EncodedMove, side: Side) {
        let (from, to) = mv.decode_into_squares();

        self.position.move_piece(PieceType::Pawn, to, from, side);
        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Pawn, from);
        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Pawn, to);

        let en_passant_capture_sq = self
            .state
            .en_passant_capture_sq()
            .expect("unwrapped en passant square when there was no en passant square");
        self.position
            .place_piece(PieceType::Pawn, en_passant_capture_sq, side.opposite());
        self.state.mut_zobrist().hash_piece(
            side.opposite(),
            PieceType::Pawn,
            en_passant_capture_sq,
        );
    }

    pub fn is_legal(
        &mut self,
        mv: Move,
        legal_check_preprocessing: &LegalCheckPreprocessing,
    ) -> bool {
        // assumes move is pseudo legal
        match mv {
            Move::King(king_mv) => is_legal_king_move(king_mv, &legal_check_preprocessing),
            Move::Piece(piece_mv)
            | Move::Rook(piece_mv)
            | Move::Pawn(piece_mv)
            | Move::DoublePawnPush(piece_mv) => {
                let (from, to) = piece_mv.decode_into_squares();
                is_legal_regular_move(
                    &self.position,
                    from,
                    to,
                    self.state.side_to_move(),
                    legal_check_preprocessing,
                )
            }
            Move::Promotion(promotion_mv) => {
                let (from, to) = promotion_mv.decode_into_squares();
                is_legal_regular_move(
                    &self.position,
                    from,
                    to,
                    self.state.side_to_move(),
                    legal_check_preprocessing,
                )
            }
            Move::EnPassant(en_passant_mv) => {
                debug_assert!(self.state.en_passant_capture_sq().is_some());

                let (from, to) = en_passant_mv.decode_into_squares();
                let side = self.state.side_to_move();
                is_legal_en_passant_move(
                    &self.position,
                    from,
                    to,
                    self.state.en_passant_capture_sq().unwrap(),
                    side,
                    legal_check_preprocessing,
                )
            }
            Move::Castle(castle) => is_legal_castle(
                &self.position,
                castle,
                self.state.side_to_move(),
                legal_check_preprocessing.controlled_squares_with_king_gone_bb(),
                legal_check_preprocessing.checkers(),
            ),
        }
    }

    fn make_regular_move(&mut self, mv: EncodedMove, side: Side) -> Option<Piece> {
        let piece_type = mv.piece_type();
        let (from, to) = mv.decode_into_squares();

        let mut capture = None;
        if mv.is_capture() {
            capture = self.position.remove_at(to);
            debug_assert!(
                capture.is_some(),
                "\nmove is capture but no piece found on capture square\nmove: {}\n{}",
                mv,
                self.position()
            );
            let capture_pc = capture.unwrap();
            self.state
                .mut_zobrist()
                .hash_piece(capture_pc.side(), capture_pc.piece_type(), to);
        }
        self.position.move_piece(piece_type, from, to, side);
        self.state.mut_zobrist().hash_piece(side, piece_type, from);
        self.state.mut_zobrist().hash_piece(side, piece_type, to);

        capture
    }

    fn make_castle_move(&mut self, castle: Castle, side: Side) {
        let (king_from, king_to) = castle.king_squares(side);
        let (rook_from, rook_to) = castle.rook_squares(side);

        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::King, king_from);
        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::King, king_to);

        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Rook, rook_from);
        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Rook, rook_to);

        self.position
            .move_piece(PieceType::King, king_from, king_to, side);
        self.position
            .move_piece(PieceType::Rook, rook_from, rook_to, side);
    }

    fn make_promotion_move(&mut self, mv: PromotionMove, side: Side) -> Option<Piece> {
        let (from, to) = mv.decode_into_squares();

        let mut capture = None;
        if mv.is_capture() {
            capture = self.position.remove_at(to);
            let capture_pc = capture.expect("captured a piece, but could not unwrap the result");
            self.state
                .mut_zobrist()
                .hash_piece(capture_pc.side(), capture_pc.piece_type(), to);
        }

        self.position.remove_piece(PieceType::Pawn, from, side);
        self.position.place_piece(mv.promote_piece_type(), to, side);

        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Pawn, from);
        self.state
            .mut_zobrist()
            .hash_piece(side, mv.promote_piece_type(), to);

        capture
    }

    pub fn adjust_castle_rights_on_capture(&mut self, mv: impl Decode, capture: Option<Piece>) {
        let pc = capture.expect("adjusting castle rights on capture but no piece was given");
        if pc.piece_type() == PieceType::Rook {
            self.adjust_castle_rights_on_capturing_rook(mv, pc.side());
        }
    }

    pub fn adjust_castle_rights_on_capturing_rook(&mut self, mv: impl Decode, side: Side) {
        let (rook_queenside_sq, rook_kingside_sq) = side.rook_start_squares();
        let (_, to) = mv.decode_into_squares();

        if to == rook_kingside_sq {
            self.state.remove_castle_rights(side, Castle::Kingside);
        } else if to == rook_queenside_sq {
            self.state.remove_castle_rights(side, Castle::Queenside);
        }
    }

    pub fn adjust_castle_rights_on_rook_move(&mut self, mv: impl Decode, side: Side) {
        let (rook_queenside_sq, rook_kingside_sq) = side.rook_start_squares();
        let (from, _) = mv.decode_into_squares();
        if from == rook_kingside_sq {
            self.state.remove_castle_rights(side, Castle::Kingside);
        } else if from == rook_queenside_sq {
            self.state.remove_castle_rights(side, Castle::Queenside);
        }
    }

    pub fn adjust_castle_rights_on_king_move(&mut self, side: Side) {
        self.state.remove_castle_rights_for_color(side);
    }

    pub fn set_en_passant(&mut self, mv: EncodedMove) {
        // assumes move is a double pawn push
        let (_, to) = mv.decode_into_squares();
        let side = self.state.side_to_move();
        let new_en_passant_sq = if side == Side::White {
            to.rank_down()
        } else {
            to.rank_up()
        };
        self.state.set_en_passant(new_en_passant_sq);
    }

    pub fn make_null_move(&mut self) {
        self.state.update_side_to_move();
        self.state.remove_en_passant();
    }

    pub fn unmake_null_move(&mut self, en_passant_option: Option<Square>) {
        self.state.revert_side_to_move();
        if let Some(en_passant_sq) = en_passant_option {
            self.state.set_en_passant(en_passant_sq);
        }
    }

    pub fn make_move(&mut self, mv: Move) -> Option<Piece> {
        // assumes move is legal
        let side = self.state.side_to_move();

        let capture = match mv {
            Move::King(mv) => {
                let capture = self.make_regular_move(mv, side);
                if mv.is_capture() {
                    self.state.reset_halfmoves();
                    self.adjust_castle_rights_on_capture(mv, capture);
                } else {
                    self.state.increase_halfmoves();
                }
                self.adjust_castle_rights_on_king_move(side);

                capture
            }
            Move::Rook(mv) => {
                let capture = self.make_regular_move(mv, side);
                if mv.is_capture() {
                    self.state.reset_halfmoves();
                    self.adjust_castle_rights_on_capture(mv, capture);
                } else {
                    self.state.increase_halfmoves();
                }
                self.adjust_castle_rights_on_rook_move(mv, side);

                capture
            }
            Move::DoublePawnPush(mv) => {
                let capture = self.make_regular_move(mv, side);
                self.set_en_passant(mv);
                self.state.reset_halfmoves();

                capture
            }
            Move::Pawn(mv) => {
                let capture = self.make_regular_move(mv, side);
                if mv.is_capture() {
                    self.adjust_castle_rights_on_capture(mv, capture);
                }
                self.state.reset_halfmoves();

                capture
            }
            Move::Piece(mv) => {
                let capture = self.make_regular_move(mv, side);
                if mv.is_capture() {
                    self.state.reset_halfmoves();
                    self.adjust_castle_rights_on_capture(mv, capture);
                } else {
                    self.state.increase_halfmoves();
                }

                capture
            }
            Move::Castle(castle_mv) => {
                self.make_castle_move(castle_mv, side);
                self.state.increase_halfmoves();
                self.adjust_castle_rights_on_king_move(side);

                None
            }
            Move::Promotion(promotion_mv) => {
                self.state.reset_halfmoves();
                let capture = self.make_promotion_move(promotion_mv, side);
                if promotion_mv.is_capture() {
                    self.adjust_castle_rights_on_capture(promotion_mv, capture);
                }

                capture
            }
            Move::EnPassant(en_passant_mv) => {
                let capture = self.make_en_passant_move(en_passant_mv, side);
                self.state.reset_halfmoves();

                capture
            }
        };

        if !matches!(mv, Move::DoublePawnPush(_)) {
            self.state.remove_en_passant();
        }

        if self.state.side_to_move() == Side::Black {
            self.state.increase_fullmoves();
        };
        self.state.update_side_to_move();
        self.state
            .push_to_zobrist_table(self.state.zobrist().to_u64());

        capture
    }

    pub fn unmake_regular_move(&mut self, mv: EncodedMove, capture: Option<Piece>, side: Side) {
        let piece_type = mv.piece_type();
        let (from, to) = mv.decode_into_squares();

        self.position.move_piece(piece_type, to, from, side);
        self.state.mut_zobrist().hash_piece(side, piece_type, from);
        self.state.mut_zobrist().hash_piece(side, piece_type, to);

        debug_assert!(self.position.at(from).is_some());

        if mv.is_capture() {
            let (capture_side, capture_pc) = capture
                .expect("capture is true, but unmake function was not given a piece")
                .decode();
            self.position.place_piece(capture_pc, to, capture_side);
            self.state
                .mut_zobrist()
                .hash_piece(capture_side, capture_pc, to);
        }
    }

    fn unmake_castle_move(&mut self, castle: Castle, side: Side) {
        let (king_from, king_to) = castle.king_squares(side);
        let (rook_from, rook_to) = castle.rook_squares(side);
        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::King, king_from);
        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::King, king_to);

        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Rook, rook_from);
        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Rook, rook_to);

        self.position
            .move_piece(PieceType::King, king_to, king_from, side);
        self.position
            .move_piece(PieceType::Rook, rook_to, rook_from, side);
    }

    fn unmake_promotion_move(&mut self, mv: PromotionMove, capture: Option<Piece>, side: Side) {
        let (from, to) = mv.decode_into_squares();

        self.position.place_piece(PieceType::Pawn, from, side);
        self.position
            .remove_piece(mv.promote_piece_type(), to, side);

        self.state
            .mut_zobrist()
            .hash_piece(side, PieceType::Pawn, from);
        self.state
            .mut_zobrist()
            .hash_piece(side, mv.promote_piece_type(), to);

        if mv.is_capture() {
            let (capture_side, capture_pc) = capture
                .expect("capture is true, but unmake function was not giving a Piece")
                .decode();
            self.position.place_piece(capture_pc, to, capture_side);

            self.state
                .mut_zobrist()
                .hash_piece(capture_side, capture_pc, to);
        }
    }

    pub fn unmake_move(&mut self, mv: Move, capture: Option<Piece>, prev_state: EncodedState) {
        debug_assert!(
            self.state
                .zobrist_table()
                .get(&self.state.zobrist().to_u64())
                .is_some(),
            "failed unmaking move: {}; no zobrist found\n{}",
            mv,
            self.position()
        );
        self.state
            .rollback_zobrist_table(self.state.zobrist().to_u64());
        self.state.decode_from(prev_state);
        let side = self.state.side_to_move();

        match mv {
            Move::King(mv)
            | Move::Piece(mv)
            | Move::Pawn(mv)
            | Move::Rook(mv)
            | Move::DoublePawnPush(mv) => {
                self.unmake_regular_move(mv, capture, side);
            }
            Move::Castle(castle_mv) => {
                self.unmake_castle_move(castle_mv, side);
            }
            Move::Promotion(promotion_mv) => {
                self.unmake_promotion_move(promotion_mv, capture, side);
            }
            Move::EnPassant(en_passant_mv) => {
                self.unmake_en_passant_move(en_passant_mv, side);
            }
        };
    }

    pub fn is_draw(&self) -> bool {
        let last_zobrist = self.state.zobrist();
        self.state.is_draw_by_repetition(*last_zobrist)
            || self.state.is_draw_by_halfmoves()
            || self.position.insufficient_material()
    }

    pub fn is_checkmate(&mut self, legal_check_preprocessing: &LegalCheckPreprocessing) -> bool {
        if legal_check_preprocessing.num_of_checkers() == 0 {
            return false;
        }

        let side = self.state.side_to_move();

        let escape_moves = self.pseudo_legal_escape_moves(side, legal_check_preprocessing);
        let escape_moves_iter = escape_moves
            .list()
            .iter()
            .filter(|mv| self.is_legal(**mv, legal_check_preprocessing));

        escape_moves_iter.count() == 0
    }

    pub fn is_stalemate(&mut self, legal_check_preprocessing: &LegalCheckPreprocessing) -> bool {
        if legal_check_preprocessing.num_of_checkers() > 0 {
            return false;
        }

        let side = self.state.side_to_move();

        let friendly_occupied = self.position().bb_side(side);
        let enemy_occupied = self.position().bb_side(side.opposite());

        let pinned_pieces_bb = legal_check_preprocessing.pinned();
        let king_sq = self.position.king_sq(side);

        for (i, piece_bb) in self.position().bb_pieces().iter().enumerate() {
            let piece_type = PIECE_TYPE_MAP[i];
            let piece_bb_iter = (*piece_bb & self.position().bb_side(side)).iter();

            for from in piece_bb_iter {
                if piece_type.has_legal_moves(
                    &self,
                    from,
                    friendly_occupied,
                    enemy_occupied,
                    side,
                    pinned_pieces_bb,
                    king_sq,
                    legal_check_preprocessing,
                ) {
                    return false;
                }
            }
        }

        return true;
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
    fn phase() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";
        let result = Game::from_fen(fen);
        match result {
            Ok(game) => {
                let expected = "Opening";
                println!("{}", game.position.phase().to_string());
                assert_eq!(game.position.phase().to_string(), expected);
            }
            Err(e) => {
                println!("{}", &e);
                panic!()
            }
        }
    }
    #[test]
    fn parse_with_starting_fen() {
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
    fn parse_with_random_fen() {
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
                assert_eq!(
                    game.state().castle_rights().to_string(),
                    (castle_rights::NONE).to_string()
                );
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
                let mut expected = castle_rights::NONE.set(Side::White, Castle::Kingside);
                expected = expected.set(Side::Black, Castle::Queenside);
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
pub mod test_pseudo_legal {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_1() {
        let fen = "4k3/7P/8/3Pp3/8/8/P7/R3K2R w KQ e6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let mv_list = game.pseudo_legal_moves(Side::White);
        let mut mv_counter: HashMap<&str, u32> = HashMap::from([
            ("king", 0),
            ("rook", 0),
            ("pawn", 0),
            ("double pawn push", 0),
            ("piece", 0),
            ("castle", 0),
            ("promotion", 0),
            ("en passant", 0),
            ("null", 0),
        ]);

        for mv in mv_list.list().iter() {
            let key_to_update = match mv {
                Move::King(_) => "king",
                Move::Rook(_) => "rook",
                Move::Pawn(_) => "pawn",
                Move::DoublePawnPush(_) => "double pawn push",
                Move::Piece(_) => "piece",
                Move::Castle(_) => "castle",
                Move::Promotion(_) => "promotion",
                Move::EnPassant(_) => "en passant",
            };

            if let Some(x) = mv_counter.get_mut(key_to_update) {
                *x += 1;
            }
        }

        assert_eq!(
            *mv_counter.get("promotion").unwrap(),
            4,
            "count of promotion moves is not correct"
        );
        assert_eq!(
            *mv_counter.get("king").unwrap(),
            5,
            "count of king moves is not correct"
        );
        assert_eq!(
            *mv_counter.get("castle").unwrap(),
            2,
            "count of castle moves is not correct"
        );
        assert_eq!(
            *mv_counter.get("en passant").unwrap(),
            1,
            "count of en passant moves is not correct"
        );
        assert_eq!(
            *mv_counter.get("double pawn push").unwrap(),
            1,
            "count of double pawn pushes is not correct"
        );
        assert_eq!(
            *mv_counter.get("rook").unwrap(),
            10,
            "count of rook moves is not correct, count of double pawn pushes is not correct"
        );
        assert_eq!(
            *mv_counter.get("piece").unwrap(),
            0,
            "count of piece moves is not correct"
        );
        assert_eq!(
            *mv_counter.get("pawn").unwrap(),
            2,
            "count of single pawn pushes is not correct"
        );
    }
}

#[cfg(test)]
pub mod test_is_legal {
    use crate::move_gen::{checkers_pinners_pinned, controlled_squares_with_king_gone};

    use super::*;

    #[test]
    fn have_to_deal_with_check() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPpP/R4K1R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();

        let side = game.state().side_to_move();
        let mv_list = game.pseudo_legal_moves(side);
        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side);
        let legal_check_preprocessing = LegalCheckPreprocessing::new(
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        );

        assert_eq!(legal_check_preprocessing.num_of_checkers(), 1);

        let prev_state = game.state.encode();

        for mv in mv_list.list().iter() {
            if game.is_legal(*mv, &legal_check_preprocessing) {
                let capture = game.make_move(*mv);
                assert!(
                    !game.position().in_check(side),
                    "king is still in check after {}",
                    mv
                );
                game.unmake_move(*mv, capture, prev_state);
            }
        }
    }
}

#[cfg(test)]
pub mod test_make_move {
    use super::*;
    use crate::{
        fen::STARTING_POSITION_FEN,
        piece_type::PromoteType,
        square::{self, *},
    };

    #[test]
    fn state_change_1() {
        let fen = "5k2/8/8/8/8/8/4Q3/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E2;
        let to = F2;
        let mv = Move::Piece(EncodedMove::new(from, to, PieceType::Queen, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert_ne!(side, game.state.side_to_move());
        assert_eq!(game.state.halfmoves(), 1);
        assert_eq!(game.state.fullmoves(), 1);
    }

    #[test]
    fn state_change_2() {
        let fen = "4k3/4p3/8/8/8/8/4P3/4K3 b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E7;
        let to = E5;
        let mv = Move::DoublePawnPush(EncodedMove::new(from, to, PieceType::Pawn, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert_ne!(side, game.state.side_to_move());
        assert_eq!(game.state.halfmoves(), 0);
        assert_eq!(game.state.fullmoves(), 2);
    }

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
    fn removes_en_passant_1() {
        let fen = "5k2/8/8/8/8/8/4Q3/4K3 w - e6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(game.state.en_passant().unwrap_or(square::NULL), E6);
        let from = E2;
        let to = F2;
        let mv = Move::Piece(EncodedMove::new(from, to, PieceType::Queen, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(game.state.en_passant().is_none());
        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(!game.position.bb_pc(PieceType::Queen, side).is_set(from));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
    }
    #[test]
    fn double_pawn_push_w() {
        let fen = "4k3/4p3/8/8/8/8/4P3/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E2;
        let to = E4;
        let mv = Move::DoublePawnPush(EncodedMove::new(from, to, PieceType::Pawn, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(PieceType::Pawn, side).is_set(to));
        assert!(!game.position.bb_pc(PieceType::Pawn, side).is_set(from));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
        assert_eq!(game.state.en_passant().unwrap_or(square::NULL), E3);
    }

    #[test]
    fn double_pawn_push_b() {
        let fen = "4k3/4p3/8/8/8/8/4P3/4K3 b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E7;
        let to = E5;
        let mv = Move::DoublePawnPush(EncodedMove::new(from, to, PieceType::Pawn, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(PieceType::Pawn, side).is_set(to));
        assert!(!game.position.bb_pc(PieceType::Pawn, side).is_set(from));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
        assert_eq!(game.state.en_passant().unwrap_or(square::NULL), E6);
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
        let score = game.position.piece_score(side.opposite());

        let capture = game.make_move(mv);

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
        assert_eq!(
            game.position.piece_score(side.opposite()),
            score
                - capture
                    .expect("capture made but no piece given")
                    .piece_type()
                    .score() as i32
        )
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
        let score = game.position.piece_score(side);
        game.make_move(mv);

        assert!(game.position.at(to).is_some());
        assert!(game.position.at(from).is_none());
        assert!(game.position.bb_pc(PieceType::Queen, side).is_set(to));
        assert!(!game.position.bb_pc(PieceType::Pawn, side).is_set(from));
        assert!(game.position.bb_side(side).is_set(to));
        assert!(!game.position.bb_side(side).is_set(from));
        assert_eq!(
            game.position.piece_score(side),
            score - PieceType::Pawn.score() as i32 + PieceType::Queen.score() as i32
        )
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
        let en_passant_capture_sq = game.state().en_passant_capture_sq().unwrap();
        game.make_move(mv);

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
    fn en_passant_iterative() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();

        game.make_move(Move::DoublePawnPush(EncodedMove::new(
            A2,
            A4,
            PieceType::Pawn,
            false,
        )));
        game.make_move(Move::Pawn(EncodedMove::new(A7, A6, PieceType::Pawn, false)));
        game.make_move(Move::Pawn(EncodedMove::new(A4, A5, PieceType::Pawn, false)));
        game.make_move(Move::DoublePawnPush(EncodedMove::new(
            B7,
            B5,
            PieceType::Pawn,
            false,
        )));
        let from = A5;
        let to = B6;
        let moving_piece_type = PieceType::Pawn;
        let capture_piece_type = PieceType::Pawn;
        let mv = Move::EnPassant(EncodedMove::new(from, to, moving_piece_type, true));
        let side = game.state.side_to_move();
        let en_passant_capture_sq = game.state().en_passant_capture_sq().unwrap();
        game.make_move(mv);

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
        let mv = Move::Castle(Castle::Kingside);
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
        let mv = Move::Castle(Castle::Queenside);
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
        let mv = Move::Castle(Castle::Kingside);
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
        let mv = Move::Castle(Castle::Queenside);
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

    #[test]
    fn king_mv_changes_castle_rights_1() {
        let fen = "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::King(EncodedMove::new(D7, E7, PieceType::King, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(!game.state.castle_rights().can(side, Castle::Kingside));
        assert!(!game.state.castle_rights().can(side, Castle::Queenside));
    }

    #[test]
    fn king_mv_changes_castle_rights_2() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::King(EncodedMove::new(E8, E7, PieceType::King, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(!game.state.castle_rights().can(side, Castle::Kingside));
        assert!(!game.state.castle_rights().can(side, Castle::Queenside));
    }

    #[test]
    fn rook_mv_changes_castle_rights_1() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Rook(EncodedMove::new(A8, A7, PieceType::Rook, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(game.state.castle_rights().can(side, Castle::Kingside));
        assert!(!game.state.castle_rights().can(side, Castle::Queenside));
        assert!(game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Kingside));
        assert!(game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Queenside));
    }

    #[test]
    fn rook_mv_changes_castle_rights_2() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Rook(EncodedMove::new(H8, H7, PieceType::Rook, false));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(!game.state.castle_rights().can(side, Castle::Kingside));
        assert!(game.state.castle_rights().can(side, Castle::Queenside));
        assert!(game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Kingside));
        assert!(game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Queenside));
    }

    #[test]
    fn rook_capture_changes_castle_rights_1() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPpP/R3K2R b KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Pawn(EncodedMove::new(G2, H1, PieceType::Pawn, true));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(!game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Kingside));
        assert!(game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Queenside));
        assert!(game.state.castle_rights().can(side, Castle::Kingside));
        assert!(game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Queenside));
    }

    #[test]
    fn rook_capture_changes_castle_rights_2() {
        let fen = "r3k2r/p1ppqpb1/bn2pnN1/3P4/1p2P3/2N2Q2/PPPBBPpP/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Piece(EncodedMove::new(G6, H8, PieceType::Knight, true));
        let side = game.state.side_to_move();
        game.make_move(mv);

        assert!(!game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Kingside));
        assert!(game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Queenside));
        assert!(game.state.castle_rights().can(side, Castle::Kingside));
        assert!(game
            .state
            .castle_rights()
            .can(side.opposite(), Castle::Queenside));
    }

    #[test]
    fn null_mv_1() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let en_passant_option = game.state().en_passant();

        let initial_zobrist = game.state().zobrist().to_u64();
        game.make_null_move();

        assert_eq!(game.state().side_to_move(), Side::Black);

        game.unmake_null_move(en_passant_option);

        assert_eq!(game.state().side_to_move(), Side::White);
        assert_eq!(initial_zobrist, game.state().zobrist().to_u64());
    }

    #[test]
    fn null_mv_2() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        game.make_move(Move::DoublePawnPush(EncodedMove::new(
            E2,
            E4,
            PieceType::Pawn,
            false,
        )));

        let initial_zobrist = game.state().zobrist().clone();
        let en_passant_option = game.state().en_passant();

        let side = game.state().side_to_move();
        game.make_null_move();
        assert_eq!(game.state().side_to_move(), Side::White);
        assert!(game.state().en_passant().is_none());

        let mut expected_zobrist = initial_zobrist.clone();
        expected_zobrist.hash_side(game.state().side_to_move());
        expected_zobrist.hash_en_passant(en_passant_option);

        assert_eq!(game.state().zobrist(), &expected_zobrist);

        let pseudo_legal_mv_list = game.pseudo_legal_moves(game.state().side_to_move());
        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);
        let prev_state = game.state().encode();

        for mv in pseudo_legal_mv_list.list().iter() {
            if !game.is_legal(*mv, &legal_check_preprocessing) {
                continue;
            }

            let capture = game.make_move(*mv);

            game.unmake_move(*mv, capture, prev_state)
        }

        assert_eq!(
            game.state().zobrist().to_u64(),
            expected_zobrist.to_u64(),
            "zobrist after legal moves are made and unmade does not match expected_zobrist"
        );
        expected_zobrist.hash_side(game.state().side_to_move());
        println!("{expected_zobrist} {}", game.state().side_to_move());
        expected_zobrist.hash_en_passant(en_passant_option);

        game.unmake_null_move(en_passant_option);

        assert_eq!(game.state().side_to_move(), Side::Black);
        assert_eq!(game.state().en_passant(), Some(E3));
        assert_eq!(
            game.state().zobrist().to_u64(),
            initial_zobrist.to_u64(),
            "zobrist after null move is unmade does not match initial zobrist"
        );
    }
}

#[cfg(test)]
pub mod test_zobrist {
    use super::*;
    use crate::{
        fen::STARTING_POSITION_FEN, piece_type::PromoteType, square::*, state::zobrist::Zobrist,
    };

    const STARTING_ZOBRIST: Zobrist = Zobrist(1208123176030986407);

    #[test]
    fn zobrist_double_pawn_push() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(game.state().zobrist(), &STARTING_ZOBRIST);

        let from = E2;
        let to = E4;
        let mv = Move::DoublePawnPush(EncodedMove::new(from, to, PieceType::Pawn, false));
        let side = game.state().side_to_move();
        game.make_move(mv);

        let mut expected = STARTING_ZOBRIST.clone();
        expected.hash_side(side.opposite());
        expected.hash_en_passant(game.state().en_passant());
        expected.hash_piece(side, PieceType::Pawn, from);
        expected.hash_piece(side, PieceType::Pawn, to);

        assert_eq!(&expected, game.state().zobrist());
    }

    #[test]
    fn zobrist_castle_kingside_w() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let start_zobrist = game.state().zobrist().clone();

        let mv = Move::Castle(Castle::Kingside);
        let side = game.state().side_to_move();
        game.make_move(mv);

        let mut expected = start_zobrist.clone();
        expected.hash_side(game.state().side_to_move());
        expected.hash_piece(side, PieceType::Rook, H1);
        expected.hash_piece(side, PieceType::Rook, F1);

        expected.hash_piece(side, PieceType::King, E1);
        expected.hash_piece(side, PieceType::King, G1);

        expected.hash_castle_rights_single(side, Castle::Kingside);
        expected.hash_castle_rights_single(side, Castle::Queenside);

        assert_eq!(&expected, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&game.state.zobrist().to_u64())
                .expect("zobrist was not found in zobrist table"),
            &1u8
        );
    }

    #[test]
    fn zobrist_castle_queenside_b() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let start_zobrist = game.state().zobrist().clone();

        let mv = Move::Castle(Castle::Queenside);
        let side = game.state().side_to_move();
        game.make_move(mv);

        let mut expected = start_zobrist.clone();
        expected.hash_side(game.state().side_to_move());
        expected.hash_piece(side, PieceType::Rook, A8);
        expected.hash_piece(side, PieceType::Rook, D8);

        expected.hash_piece(side, PieceType::King, E8);
        expected.hash_piece(side, PieceType::King, C8);

        expected.hash_castle_rights_single(side, Castle::Kingside);
        expected.hash_castle_rights_single(side, Castle::Queenside);

        assert_eq!(&expected, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&game.state.zobrist().to_u64())
                .expect("zobrist was not found in zobrist table"),
            &1u8
        )
    }

    #[test]
    fn zobrist_promotion_move() {
        let fen = "r3k3/pppppppP/8/8/4P3/8/PPPP1PP1/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let start_zobrist = game.state().zobrist().clone();

        let from = H7;
        let to = H8;
        let promote_type = PromoteType::Knight;
        let mv = Move::Promotion(PromotionMove::new(from, to, &promote_type, false));
        let side = game.state().side_to_move();
        game.make_move(mv);

        let mut expected = start_zobrist;
        expected.hash_side(side.opposite());
        expected.hash_piece(side, PieceType::Pawn, from);
        expected.hash_piece(side, PieceType::Knight, to);

        assert_eq!(&expected, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&game.state.zobrist().to_u64())
                .expect("zobrist was not found in zobrist table"),
            &1u8
        )
    }

    #[test]
    fn zobrist_en_passant() {
        let fen = "r3k3/ppppp1pP/8/8/4Pp2/8/PPPP1PP1/R3K2R b KQkq e3 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let start_zobrist = game.state().zobrist().clone();

        let from = F4;
        let to = E3;
        let mv = Move::EnPassant(EncodedMove::new(from, to, PieceType::Pawn, true));
        let side = game.state().side_to_move();
        let en_passant = game.state().en_passant();
        let en_passant_capture_sq = game.state().en_passant_capture_sq();
        game.make_move(mv);

        let mut expected = start_zobrist;
        expected.hash_side(game.state().side_to_move());
        expected.hash_en_passant(en_passant);
        expected.hash_piece(side, PieceType::Pawn, from);
        expected.hash_piece(side, PieceType::Pawn, to);
        expected.hash_piece(
            side.opposite(),
            PieceType::Pawn,
            en_passant_capture_sq.unwrap(),
        );

        assert_eq!(&expected, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&game.state.zobrist().to_u64())
                .expect("zobrist was not found in zobrist table"),
            &1u8
        )
    }

    #[test]
    fn zobrist_unmake_double_pawn_push() {
        let fen = STARTING_POSITION_FEN;
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        assert_eq!(game.state().zobrist(), &STARTING_ZOBRIST);

        let from = E2;
        let to = E4;
        let mv = Move::DoublePawnPush(EncodedMove::new(from, to, PieceType::Pawn, false));
        let prev_state = game.state().encode();
        let capture = game.make_move(mv);
        let zobrist = game.state.zobrist().to_u64();
        game.unmake_move(mv, capture, prev_state);

        let expected = STARTING_ZOBRIST.clone();

        assert_eq!(&expected, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&zobrist)
                .expect("zobrist was not found in zobrist table"),
            &0u8
        )
    }

    #[test]
    fn zobrist_unmake_promote() {
        let fen = "4k3/7P/8/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let start_zobrist = game.state.zobrist().clone();

        let from = H7;
        let to = H8;
        let mv = Move::Promotion(PromotionMove::new(from, to, &PromoteType::Queen, false));
        let prev_state = game.state().encode();
        let capture = game.make_move(mv);
        let zobrist = game.state.zobrist().to_u64();
        game.unmake_move(mv, capture, prev_state);

        assert_eq!(&start_zobrist, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&zobrist)
                .expect("zobrist was not found in zobrist table"),
            &0u8
        )
    }

    #[test]
    fn zobrist_unmake_en_passant() {
        let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let start_zobrist = game.state.zobrist().clone();

        let from = E5;
        let to = D6;
        let mv = Move::EnPassant(EncodedMove::new(from, to, PieceType::Pawn, true));
        let prev_state = game.state().encode();
        let capture = game.make_move(mv);
        let zobrist = game.state.zobrist().to_u64();
        game.unmake_move(mv, capture, prev_state);

        assert_eq!(&start_zobrist, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&zobrist)
                .expect("zobrist was not found in zobrist table"),
            &0u8
        )
    }

    #[test]
    fn zobrist_unmake_castle_kingside_w() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let start_zobrist = game.state().zobrist().clone();

        let mv = Move::Castle(Castle::Kingside);
        let prev_state = game.state().encode();
        game.make_move(mv);
        let zobrist = game.state.zobrist().to_u64();
        game.unmake_move(mv, None, prev_state);

        let expected = start_zobrist.clone();

        assert_eq!(&expected, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&zobrist)
                .expect("zobrist was not found in zobrist table"),
            &0u8
        )
    }

    #[test]
    fn zobrist_unmake_castle_queenside_b() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let start_zobrist = game.state().zobrist().clone();

        let mv = Move::Castle(Castle::Queenside);
        let prev_state = game.state().encode();
        game.make_move(mv);
        let zobrist = game.state.zobrist().to_u64();
        game.unmake_move(mv, None, prev_state);

        let expected = start_zobrist.clone();

        assert_eq!(&expected, game.state().zobrist());
        assert_eq!(
            game.state
                .zobrist_table()
                .get(&zobrist)
                .expect("zobrist was not found in zobrist table"),
            &0u8
        )
    }

    #[test]
    fn unmake_capture() {
        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E4;
        let to = D5;

        let mv = Move::Pawn(EncodedMove::new(from, to, PieceType::Pawn, true));
        let prev_state = game.state().encode();

        let zobrist = game.state.zobrist().clone();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        assert_eq!(&zobrist, game.state().zobrist());
    }
}

#[cfg(test)]
pub mod test_unmake_move {
    use super::*;
    use crate::{piece_type::PromoteType, square::*};

    #[test]
    fn state_revert_1() {
        let fen = "5k2/8/8/8/8/8/4Q3/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E2;
        let to = F2;
        let mv = Move::Piece(EncodedMove::new(from, to, PieceType::Queen, false));
        let side = game.state.side_to_move();
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        assert_eq!(side, game.state.side_to_move());
        assert_eq!(game.state.halfmoves(), 0);
        assert_eq!(game.state.fullmoves(), 1);
    }

    #[test]
    fn state_revert_2() {
        let fen = "4k3/4p3/8/8/8/8/4P3/4K3 b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = E7;
        let to = E5;
        let mv = Move::DoublePawnPush(EncodedMove::new(from, to, PieceType::Pawn, false));
        let side = game.state.side_to_move();
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        assert_eq!(side, game.state.side_to_move());
        assert_eq!(game.state.halfmoves(), 0);
        assert_eq!(game.state.fullmoves(), 1);
    }
    #[test]
    fn state_revert_3() {
        let fen = "4k3/8/8/5Pp1/8/8/8/4K3 w - g6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let from = F5;
        let to = G6;
        let moving_piece_type = PieceType::Pawn;
        let mv = Move::EnPassant(EncodedMove::new(from, to, moving_piece_type, true));
        let en_passant_sq = game.state.en_passant();
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        assert_eq!(en_passant_sq, game.state.en_passant());
    }

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
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_none());
        assert!(!game.position.bb_pc(PieceType::Queen, side).is_set(to));
        println!("{}", game.position().bb_pc(PieceType::Queen, side));
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
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_some());
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
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

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
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_some());
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
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        let en_passant_capture_sq = game.state().en_passant_capture_sq().unwrap();
        assert!(game.position.at(from).is_some());
        assert!(game.position.at(to).is_none());
        assert!(game.position.at(en_passant_capture_sq).is_some());
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
        let mv = Move::Castle(Castle::Kingside);
        let side = game.state.side_to_move();
        let castle_rights = game.state.castle_rights();
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

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
        assert_eq!(castle_rights, game.state.castle_rights());
    }

    #[test]
    fn castle_move_w_queenside() {
        let fen = "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::Queenside);
        let side = game.state.side_to_move();
        let castle_rights = game.state.castle_rights();
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

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
        assert_eq!(castle_rights, game.state.castle_rights());
    }

    #[test]
    fn castle_move_b_kingside() {
        let fen = "r3k2r/8/8/8/8/8/8/4K3 b k - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv = Move::Castle(Castle::Kingside);
        let side = game.state.side_to_move();
        let castle_rights = game.state.castle_rights();
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        let king_after = F8;
        let king_before = E8;
        let rook_after = G8;
        let rook_before = H8;

        assert_eq!(castle_rights, game.state.castle_rights());
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
        let mv = Move::Castle(Castle::Queenside);
        let castle_rights = game.state.castle_rights();
        let side = game.state.side_to_move();
        let prev_state = game.state.encode();
        let capture = game.make_move(mv);
        game.unmake_move(mv, capture, prev_state);

        let king_after = C8;
        let king_before = E8;
        let rook_after = D8;
        let rook_before = A8;

        assert_eq!(castle_rights, game.state.castle_rights());
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

#[cfg(test)]
pub mod test_is_draw {
    use super::*;
    use crate::square::*;

    #[test]
    fn is_draw_by_repetition_1() {
        let fen = "4k3/8/8/8/2B5/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv_1 = Move::Piece(EncodedMove::new(C4, B5, PieceType::Bishop, false));
        let mv_2 = Move::King(EncodedMove::new(E8, E7, PieceType::King, false));
        let mv_3 = Move::Piece(EncodedMove::new(B5, C4, PieceType::Bishop, false));
        let mv_4 = Move::King(EncodedMove::new(E7, E8, PieceType::King, false));

        game.make_move(mv_1);
        game.make_move(mv_2);
        game.make_move(mv_3);
        game.make_move(mv_4);

        game.make_move(mv_1);
        game.make_move(mv_2);
        game.make_move(mv_3);
        game.make_move(mv_4);

        assert!(game.is_draw())
    }

    #[test]
    fn is_draw_by_repetition_2() {
        let fen = "4k1nn/8/8/8/2B5/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let mv_1 = Move::Piece(EncodedMove::new(C4, B5, PieceType::Bishop, false));
        let mv_2 = Move::King(EncodedMove::new(E8, E7, PieceType::King, false));
        let mv_3 = Move::Piece(EncodedMove::new(B5, C4, PieceType::Bishop, false));
        let mv_4 = Move::King(EncodedMove::new(E7, E8, PieceType::King, false));

        game.make_move(mv_1);
        game.make_move(mv_2);
        game.make_move(mv_3);
        game.make_move(mv_4);

        game.make_move(mv_1);
        game.make_move(mv_2);
        game.make_move(mv_3);

        assert!(!game.is_draw())
    }

    #[test]
    fn is_draw_by_insufficient_material_1() {
        // king + bishop vs king
        let fen = "4k3/8/8/8/2B5/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(game.is_draw());
    }

    #[test]
    fn is_draw_by_insufficient_material_2() {
        // king + bishop vs king + bishop (opposite colors)
        let fen = "4kb2/8/8/8/2B5/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(!game.is_draw());
    }

    #[test]
    fn is_draw_by_insufficient_material_3() {
        // king + bishop vs king + bishop (same colors)
        let fen = "4k1b1/8/8/8/2B5/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(game.is_draw());
    }

    #[test]
    fn is_draw_by_insufficient_material_4() {
        // king + knight vs king
        let fen = "4k3/6n1/8/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(game.is_draw());
    }

    #[test]
    fn is_draw_by_insufficient_material_5() {
        // king vs king
        let fen = "4k3/8/8/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(game.is_draw());
    }

    #[test]
    fn is_draw_by_insufficient_material_6() {
        // king + queen vs king
        let fen = "4k3/5q2/8/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(!game.is_draw());
    }

    #[test]
    fn is_draw_by_insufficient_material_7() {
        // king + rook vs king
        let fen = "4k3/8/8/8/8/8/7R/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(!game.is_draw());
    }

    #[test]
    fn is_draw_by_halfmoves_1() {
        let fen = "4k3/5q2/8/8/8/1Q6/8/4K3 w - - 49 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(!game.is_draw());
    }

    #[test]
    fn is_draw_by_halfmoves_2() {
        let fen = "4k3/5q2/8/8/8/1Q6/8/4K3 w - - 50 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert!(game.is_draw());
    }
}

#[cfg(test)]
pub mod test_is_checkmate {
    use super::*;
    use crate::move_gen::{checkers_pinners_pinned, controlled_squares_with_king_gone};

    #[test]
    fn pos_1() {
        let fen = "rnb1kbnr/ppppppp1/8/8/8/8/PPPPPPP1/RNBQ2Kq w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();

        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing::new(
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        );

        assert!(game.is_checkmate(&legal_check_preprocessing))
    }

    #[test]
    fn pos_2() {
        let fen = "rnb1kbn1/ppppppp1/8/8/8/8/PPPPPPP1/RNBQ2Kq w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();

        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing::new(
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        );

        assert!(!game.is_checkmate(&legal_check_preprocessing))
    }

    #[test]
    fn pos_3() {
        let fen = "rnb1k1n1/ppppppp1/3b4/3q4/8/8/PPPPP3/RNBQ2NK w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();

        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing::new(
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        );

        assert!(!game.is_checkmate(&legal_check_preprocessing))
    }

    #[test]
    fn pos_4() {
        let fen = "rnb1k1n1/ppppppp1/3b4/3q4/8/4P3/PPPP4/RNBQ2BK w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();

        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing::new(
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        );

        assert!(!game.is_checkmate(&legal_check_preprocessing))
    }

    #[test]
    fn pos_5() {
        let fen = "rnb1k1n1/ppppppp1/3b4/3q4/8/4P3/PPPP4/RNBB2BK w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();

        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing::new(
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        );

        assert!(!game.is_checkmate(&legal_check_preprocessing))
    }

    #[test]
    fn pos_6() {
        let fen = "rnb1k1n1/ppppppp1/3b4/3q4/8/4P3/PPPP4/RNBB2RK w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state().side_to_move();

        let (checkers, pinners, pinned) = checkers_pinners_pinned(game.position(), side.opposite());
        let controlled_squares_with_king_gone_bb =
            controlled_squares_with_king_gone(game.mut_position(), side.opposite());
        let legal_check_preprocessing = LegalCheckPreprocessing::new(
            checkers,
            pinners,
            pinned,
            controlled_squares_with_king_gone_bb,
        );

        assert!(!game.is_checkmate(&legal_check_preprocessing))
    }
}

#[cfg(test)]
pub mod test_escape_moves {
    use crate::bitboard;

    use super::*;

    #[test]
    fn pos_1() {
        let fen = "4k3/4q3/8/R7/7Q/3N4/B5N1/4K1B1 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();

        let side = game.state.side_to_move();
        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);
        let escape_mv_list = game.pseudo_legal_escape_moves(side, &legal_check_preprocessing);

        let mut moves_bb_arr = [
            bitboard::EMPTY,
            bitboard::EMPTY,
            bitboard::EMPTY,
            bitboard::EMPTY,
            bitboard::EMPTY,
            bitboard::EMPTY,
        ];

        for mv in escape_mv_list.list().iter() {
            match mv {
                Move::King(king_mv) => {
                    let (_, to) = king_mv.decode_into_squares();
                    println!("{to}");
                    let (_, to_bb) = king_mv.decode_into_bb();
                    moves_bb_arr[PieceType::King.to_usize()] |= to_bb;
                }
                Move::Rook(rook_mv) => {
                    let (_, to_bb) = rook_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Rook.to_usize()] |= to_bb;
                }
                Move::Pawn(pawn_mv) => {
                    let (_, to_bb) = pawn_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Pawn.to_usize()] |= to_bb;
                }
                Move::DoublePawnPush(pawn_mv) => {
                    let (_, to_bb) = pawn_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Pawn.to_usize()] |= to_bb;
                }
                Move::Piece(piece_mv) => {
                    let piece_type = piece_mv.piece_type();
                    let (_, to_bb) = piece_mv.decode_into_bb();
                    moves_bb_arr[piece_type.to_usize()] |= to_bb;
                }
                Move::Castle(_) => {}
                Move::Promotion(pawn_mv) => {
                    let (_, to_bb) = pawn_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Pawn.to_usize()] |= to_bb;
                }
                Move::EnPassant(pawn_mv) => {
                    let (_, to_bb) = pawn_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Pawn.to_usize()] |= to_bb;
                }
            }
        }

        println!("{}", moves_bb_arr[PieceType::King.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::King.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|...#.#..|2
            1|...#.#..|1
              ABCDEFGH
            ",
            ),
            "failed king test"
        );
        println!("{}", moves_bb_arr[PieceType::Bishop.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::Bishop.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|........|7
            6|....#...|6
            5|........|5
            4|........|4
            3|....#...|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed bishop test"
        );
        println!("{}", moves_bb_arr[PieceType::Queen.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::Queen.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|....#...|7
            6|........|6
            5|........|5
            4|....#...|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed queen test"
        );
        println!("{}", moves_bb_arr[PieceType::Rook.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::Rook.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|....#...|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed rook test"
        );
        println!("{}", moves_bb_arr[PieceType::Knight.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::Knight.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|....#...|5
            4|........|4
            3|....#...|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed knight test"
        );
    }
}

#[cfg(test)]
pub mod test_loud_moves {
    use crate::bitboard;

    use super::*;

    #[test]
    fn pos_1() {
        let fen = "r1bqr2k/ppp3bp/2np2p1/8/2BnPQ2/2N2N2/PPPB1PP1/2KR3R w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let side = game.state.side_to_move();
        let loud_mv_list = game.pseudo_legal_loud_moves(side);

        let mut moves_bb_arr = [
            bitboard::EMPTY,
            bitboard::EMPTY,
            bitboard::EMPTY,
            bitboard::EMPTY,
            bitboard::EMPTY,
            bitboard::EMPTY,
        ];

        for mv in loud_mv_list.list().iter() {
            match mv {
                Move::King(king_mv) => {
                    let (_, to) = king_mv.decode_into_squares();
                    println!("{to}");
                    let (_, to_bb) = king_mv.decode_into_bb();
                    moves_bb_arr[PieceType::King.to_usize()] |= to_bb;
                }
                Move::Rook(rook_mv) => {
                    let (_, to_bb) = rook_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Rook.to_usize()] |= to_bb;
                }
                Move::Pawn(pawn_mv) => {
                    let (_, to_bb) = pawn_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Pawn.to_usize()] |= to_bb;
                }
                Move::DoublePawnPush(pawn_mv) => {
                    let (_, to_bb) = pawn_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Pawn.to_usize()] |= to_bb;
                }
                Move::Piece(piece_mv) => {
                    let piece_type = piece_mv.piece_type();
                    let (_, to_bb) = piece_mv.decode_into_bb();
                    moves_bb_arr[piece_type.to_usize()] |= to_bb;
                }
                Move::Castle(_) => {}
                Move::Promotion(pawn_mv) => {
                    let (_, to_bb) = pawn_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Pawn.to_usize()] |= to_bb;
                }
                Move::EnPassant(pawn_mv) => {
                    let (_, to_bb) = pawn_mv.decode_into_bb();
                    moves_bb_arr[PieceType::Pawn.to_usize()] |= to_bb;
                }
            }
        }

        println!("{}", moves_bb_arr[PieceType::King.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::King.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed king test"
        );
        println!("{}", moves_bb_arr[PieceType::Bishop.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::Bishop.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed bishop test"
        );
        println!("{}", moves_bb_arr[PieceType::Queen.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::Queen.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|........|7
            6|...#....|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed queen test"
        );
        println!("{}", moves_bb_arr[PieceType::Rook.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::Rook.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|.......#|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed rook test"
        );
        println!("{}", moves_bb_arr[PieceType::Knight.to_usize()].to_string(),);
        assert_eq!(
            moves_bb_arr[PieceType::Knight.to_usize()].to_string(),
            unindent::unindent(
                "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|...#....|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
            ),
            "failed knight test"
        );
    }
}

#[cfg(test)]
pub mod test_is_stalemate {
    use super::*;

    #[test]
    fn stalemate_pos_1() {
        let fen = "4k3/4Pn2/4K3/7B/8/8/8/8 b - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(game.is_stalemate(legal_check_preprocessing))
    }

    #[test]
    fn stalemate_pos_2() {
        let fen = "2b5/8/1n4r1/KPp3rk/6r1/8/8/8 w - c6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(game.is_stalemate(legal_check_preprocessing))
    }

    #[test]
    fn stalemate_pos_3() {
        let fen = "3bb3/k7/8/1Pp5/KN5r/7r/8/8 w - c6 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(!game.is_stalemate(legal_check_preprocessing))
    }

    #[test]
    fn stalemate_pos_4() {
        let fen = "3bb3/k7/8/1qp5/KN5r/7r/8/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(!game.is_stalemate(legal_check_preprocessing))
    }

    #[test]
    fn stalemate_pos_5() {
        let fen = "3bb3/k7/8/1bp5/KN5r/7r/8/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(!game.is_stalemate(legal_check_preprocessing))
    }

    #[test]
    fn stalemate_pos_6() {
        let fen = "3bb3/k7/8/1Rp5/KN5r/7r/8/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(game.is_stalemate(legal_check_preprocessing))
    }

    #[test]
    fn stalemate_pos_7() {
        let fen = "3bb3/k7/8/1Np5/KN5r/7r/8/8 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(game.is_stalemate(legal_check_preprocessing))
    }

    #[test]
    fn stalemate_pos_8() {
        let fen = "3bb3/k7/8/1Np5/KN5r/7r/8/4R3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(!game.is_stalemate(legal_check_preprocessing))
    }

    #[test]
    fn stalemate_pos_9() {
        let fen = "3bb3/k7/8/1Np5/KN5r/7r/8/4Q3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let mut game = result.unwrap();
        let side = game.state.side_to_move();
        let legal_check_preprocessing = &LegalCheckPreprocessing::from(&mut game, side);

        assert!(!game.is_stalemate(legal_check_preprocessing))
    }
}
