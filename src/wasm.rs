use crate::{
    bitboard,
    fen::STARTING_POSITION_FEN,
    game::Game,
    move_gen::{
        check_legal::LegalCheckPreprocessing,
        pseudo_legal::{self},
    },
    move_list::MoveList,
    mv::{castle::Castle, Decode, Move},
    piece::Piece,
    piece_type::{PieceType, PromoteType},
    search::{Depth, MoveFinder, DEFAULT_DEPTH, DEFAULT_MAX_DEPTH},
    side::Side,
    square::{self, Square, ALL_SQUARES},
    uci::{algebra_to_move, input_position, move_to_algebra},
};
use wasm_bindgen::prelude::*;

//A macro to provide `println!(..)`-style syntax for `console.log` logging.
// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

#[wasm_bindgen]
pub fn set_console_error_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct ClientGameInterface {
    game: Game,
    move_finder: MoveFinder,
}

impl ClientGameInterface {
    fn move_notation_from_numbers(from: u32, to: u32) -> String {
        format!(
            "{}{}",
            square::ALL_SQUARES[from as usize].to_string(),
            square::ALL_SQUARES[to as usize].to_string()
        )
    }

    fn pseudo_legal_moves_at_sq(&self, from: Square, piece: Piece) -> MoveList {
        let side = piece.side();
        let piece_type = piece.piece_type();

        let friendly_occupied = self.game.position().bb_side(side);
        let enemy_occupied = self.game.position().bb_side(side.opposite());

        let en_passant = {
            if self.game.state().side_to_move() == side {
                self.game.state().en_passant()
            } else {
                None
            }
        };
        let moves_bb = piece_type.pseudo_legal_moves_bb(
            from,
            friendly_occupied,
            enemy_occupied,
            side,
            en_passant,
        );

        let mut mv_list: MoveList = MoveList::new();
        piece_type.push_bb_to_move_list(
            &mut mv_list,
            moves_bb,
            from,
            side,
            enemy_occupied,
            en_passant,
        );
        if piece_type == PieceType::King {
            let castle_rights = self.game.state().castle_rights();
            if castle_rights.can(side, Castle::Queenside) {
                mv_list.push_move(Move::Castle(Castle::Queenside))
            }
            if castle_rights.can(side, Castle::Kingside) {
                mv_list.push_move(Move::Castle(Castle::Kingside))
            }
        }

        mv_list
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
                let piece_type = mv.piece_type();

                let friendly_occupied = self.game.position().bb_side(side);
                let enemy_occupied = self.game.position().bb_side(side.opposite());

                let pseudo_legal_moves = piece_type.pseudo_legal_moves_bb(
                    from,
                    friendly_occupied,
                    enemy_occupied,
                    side,
                    self.game.state().en_passant(),
                );
                pseudo_legal_moves.is_set(to)
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
impl ClientGameInterface {
    pub fn active_side(&self) -> String {
        self.game.state().side_to_move().to_string()
    }

    pub fn change_search_depth(&mut self, depth: Depth) {
        self.move_finder.change_search_depth(depth)
    }

    pub fn change_max_depth(&mut self, depth: Depth) {
        self.move_finder.change_max_depth(depth)
    }

    pub fn from_history(history: &str) -> ClientGameInterface {
        let mut game = Game::from_fen(STARTING_POSITION_FEN).unwrap();
        if history != "" {
            input_position(&format!("position startpos moves {}", history), &mut game);
        };

        ClientGameInterface {
            game: game.clone(),
            move_finder: MoveFinder::new(game, DEFAULT_DEPTH, DEFAULT_MAX_DEPTH),
        }
    }

    pub fn make_move(&mut self, move_notation: &str) {
        if move_notation != "" {
            input_position(&format!("position moves {}", move_notation), &mut self.game);
        }
    }

    pub fn is_promotion(&mut self, from: u32, to: u32) -> bool {
        let at_from = self.game.position().at(Square(from as usize));
        if at_from.is_none() {
            return false;
        }

        let (side, pc) = at_from.unwrap().decode();
        println!("side: {}, pieceType: {}", side, pc);
        if pc != PieceType::Pawn {
            return false;
        }

        let promote_rank_bb = if side == Side::White {
            bitboard::ROW_8
        } else {
            bitboard::ROW_1
        };
        if !promote_rank_bb.is_set(Square(to as usize)) {
            return false;
        }

        return self.validate_move(from, to, side == Side::White);
    }

    pub fn validate_move(&mut self, from: u32, to: u32, is_white: bool) -> bool {
        let side = if is_white == true { Side::White } else { Side::Black };

        let move_notation = ClientGameInterface::move_notation_from_numbers(from, to);

        let mv_result = algebra_to_move(&move_notation, &self.game);
        assert!(mv_result.is_ok(), "{} is not a valid move", move_notation);
        let mv = mv_result.unwrap();

        let mut game = self.game.clone();
        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);

        self.is_move_pseudo_legal(mv, side) && game.is_legal(mv, &legal_check_preprocessing)
    }

    pub fn legal_moves_at_sq(&mut self, from: u32) -> Vec<u32> {
        let from = ALL_SQUARES[from as usize];

        let piece_result = self.game.position().at(from);
        assert!(
            piece_result.is_some(),
            "no piece found at {}\n{}",
            from,
            self.game.position()
        );
        let piece = piece_result.unwrap();
        let pseudo_legal_mv_list = self.pseudo_legal_moves_at_sq(from, piece);

        let mut game = self.game.clone();
        let side = piece.side();
        let mut legal_moves: Vec<u32> =
            Vec::with_capacity(pseudo_legal_mv_list.list().len() as usize);
        let legal_check_preprocessing = LegalCheckPreprocessing::from(&mut game, side);
        for mv in pseudo_legal_mv_list.list() {
            println!("{mv}");
            if game.is_legal(*mv, &legal_check_preprocessing) {
                println!("is legal");
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

    pub fn engine_move(&mut self) -> String {
        let (best_move, _) = self.move_finder.get().unwrap();

        move_to_algebra(best_move, self.game.state().side_to_move())
    }

    pub fn to_string(&self) -> String {
        let mut string = "".to_string();

        let char_at = |sq: Square| -> char {
            let result = self.game.position().at(sq);
            if let Some(_) = result {
                let piece = result.unwrap();
                let (side, pc) = piece.decode();

                match side {
                    Side::White => pc.to_char().to_uppercase().next().unwrap(),
                    Side::Black => pc.to_char(),
                }
            } else {
                '.'
            }
        };

        for row in (0..8).rev() {
            for col in 0..8 {
                string.push(char_at(Square::from(row, col)));
            }
        }

        format!("{}", &string)
    }

    pub fn name_of_square(square: usize) -> String {
        assert!(square < 64, "{} is not a valid square", square);
        ALL_SQUARES[square].to_string()
    }

    pub fn file_of_square(square: usize) -> usize {
        assert!(
            square < 64,
            "{} is an invalid square (square must be between 0 and 63)",
            square
        );
        ALL_SQUARES[square].file()
    }

    pub fn rank_of_square(square: usize) -> usize {
        assert!(
            square < 64,
            "{} is an invalid square (square must be between 0 and 63)",
            square
        );
        ALL_SQUARES[square].rank()
    }

    pub fn make_move_notation(from: usize, to: usize, promote_piece: Option<char>) -> String {
        assert!(
            from < 64,
            "{} is an invalid square (square must be between 0 and 63)",
            from
        );
        assert!(
            to < 64,
            "{} is an invalid square (square must be between 0 and 63)",
            to
        );

        let mut move_notation = format!(
            "{}{}",
            ALL_SQUARES[from].to_string(),
            ALL_SQUARES[to].to_string()
        );
        if promote_piece.is_some() {
            let promote_piece = promote_piece.unwrap();
            assert!(
                PromoteType::try_from(promote_piece).is_ok(),
                "{} is not a valid promotion piece",
                promote_piece
            );

            move_notation = format!("{}{}", move_notation, promote_piece);
        }

        move_notation
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_string() {
        let game = ClientGameInterface::from_history("");
        let expected = "rnbqkbnrpppppppp................................PPPPPPPPRNBQKBNR";

        assert_eq!(game.to_string(), expected)
    }

    #[test]
    fn test_to_string_2() {
        let game = ClientGameInterface::from_history("e2e4 e7e5");
        let expected = "rnbqkbnrpppp.ppp............p.......P...........PPPP.PPPRNBQKBNR";

        assert_eq!(game.to_string(), expected)
    }

    #[test]
    fn file_of_square() {
        let square = 0;
        let expected = 0;
        assert_eq!(expected, ClientGameInterface::file_of_square(square));
    }

    #[test]
    fn file_of_square_2() {
        let square = 7;
        let expected = 7;
        assert_eq!(expected, ClientGameInterface::file_of_square(square));
    }

    #[test]
    fn rank_of_square() {
        let square = 0;
        let expected = 0;
        assert_eq!(expected, ClientGameInterface::rank_of_square(square));
    }

    #[test]
    fn rank_of_square_2() {
        let square = 63;
        let expected = 7;
        assert_eq!(expected, ClientGameInterface::file_of_square(square));
    }

    #[test]
    fn make_move_notation() {
        let from = 0;
        let to = 1;
        let promote_piece = None;

        let expected = "a1b1";
        assert_eq!(
            ClientGameInterface::make_move_notation(from, to, promote_piece),
            expected
        );
    }

    #[test]
    fn make_move_notation_2() {
        let from = 0;
        let to = 1;
        let promote_piece = Some('q');

        let expected = "a1b1q";
        assert_eq!(
            ClientGameInterface::make_move_notation(from, to, promote_piece),
            expected
        );
    }

    #[test]
    #[should_panic]
    fn make_move_notation_3() {
        let from = 0;
        let to = 1;
        let promote_piece = Some('k');

        ClientGameInterface::make_move_notation(from, to, promote_piece);
    }

    #[test]
    fn is_promotion_1() {
        let mut game = ClientGameInterface::from_history(
            &unindent::unindent(
                "e2e4 e7e5 b1c3 b8c6 f1c4 g8f6 g1f3 f8c5 d2d3 h7h6 c3d5 e8g8 d5f6 d8f6 c2c3 d7d6
            b2b4 c5b6 e1g1 c6e7 a2a4 a7a5 b4a5 b6a5 c1b2 e7g6 d3d4 c8g4 h2h3 g4f3 d1f3 g6f4
            c4a2 f4e6 f3d1 f8d8 a1c1 g8f8 a2d5 a8b8 d4e5 d6e5 b2a3 f8g8 d1e1 e6f4 d5c4 f4d3
            c4d3 d8d3 a3b4 a5b6 a4a5 b6a7 c1d1 b8d8 d1d3 d8d3 e1e2 f6d8 b4e7 d8d7 e7a3 d3c3
            a3b2 c3c5 f1a1 d7b5 e2d2 g8h7 a1c1 c5c1 d2c1 b5a5 b2c3 a5c5 c1b2 f7f6 h3h4 b7b5
            b2d2 a7b6 c3b2 c7c6 h4h5 c5e7 b2a1 c6c5 d2d5 e7d8 d5d8 b6d8 a1b2 d8e7 g1h2 c5c4
            h2g1 h7g8 b2c3 g8f7 f2f4 f7e6 f4f5 e6d7 g1f2 d7c6 f2f3 c6c5 c3a1 b5b4 a1b2 c4c3
            b2c1 c5c4 f3e2 b4b3 e2d1 e7c5 g2g3 c4d3 d1e1 c3c2 e1f1 c5d4 f1g2 b3b2 c1b2 d4b2 g3g4",
            )
            .replace("\n", " "),
        );

        let is_promotion = game.is_promotion(square::C2.to_u32(), square::C1.to_u32());

        assert!(is_promotion)
    }

    #[test]
    fn is_promotion_2() {
        let mut game = ClientGameInterface::from_history("");

        let is_promotion = game.is_promotion(square::E2.to_u32(), square::E4.to_u32());

        assert!(!is_promotion)
    }

    #[test]
    fn legal_moves_at_sq_1() {
        let mut game = ClientGameInterface::from_history("");

        let legal_sqs = game.legal_moves_at_sq(square::E2.to_u32());

        println!("{:?}", legal_sqs.iter().map(|v| Square(*v as usize)));

        assert_eq!(legal_sqs.len(), 2);
    }

    #[test]
    fn legal_moves_castling() {
        let mut game = ClientGameInterface::from_history("e2e4 d7d5 f1e2 c8d7 g1f3 b8c6");

        let legal_sqs = game.legal_moves_at_sq(square::E1.to_u32());

        println!(
            "{:?}",
            legal_sqs
                .iter()
                .map(|v| Square(*v as usize).to_string())
                .collect::<Vec<String>>()
        );

        assert_eq!(legal_sqs.len(), 2);
    }
}
