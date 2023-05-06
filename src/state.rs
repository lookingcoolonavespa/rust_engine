pub mod castle_rights;
pub mod position;
pub mod zobrist;

use core::fmt;
use std::collections::HashMap;

use self::{castle_rights::CastleRights, zobrist::Zobrist};
use crate::{
    mv::castle::Castle,
    side::{Side, SIDE_MAP},
    square::{self, Square},
};

type ZobristTable = HashMap<u64, u8>;
#[derive(Clone)]
pub struct State {
    en_passant: Option<Square>,
    side_to_move: Side,
    castle_rights: CastleRights,
    zobrist: Zobrist,
    halfmoves: u16,
    fullmoves: u16,
    zobrist_table: ZobristTable,
}

impl State {
    pub fn new(
        en_passant: Option<Square>,
        side_to_move: Side,
        castle_rights: CastleRights,
        halfmoves: u16,
        fullmoves: u16,
        zobrist: Zobrist,
    ) -> State {
        State {
            en_passant,
            side_to_move,
            castle_rights,
            halfmoves,
            fullmoves,
            zobrist,
            zobrist_table: HashMap::new(),
        }
    }

    pub fn decode_from(&mut self, encoded_state: EncodedState) {
        self.zobrist.hash_en_passant(self.en_passant);
        self.zobrist.hash_side(self.side_to_move);
        self.zobrist.hash_castle_rights_all(self.castle_rights);

        self.en_passant = encoded_state.en_passant();
        self.side_to_move = encoded_state.side_to_move();
        self.castle_rights = encoded_state.castle_rights();
        self.halfmoves = encoded_state.halfmoves();
        self.fullmoves = encoded_state.fullmoves();

        self.zobrist.hash_en_passant(self.en_passant);
        self.zobrist.hash_castle_rights_all(self.castle_rights);
    }

    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    pub fn side_to_move(&self) -> Side {
        self.side_to_move
    }

    pub fn castle_rights(&self) -> CastleRights {
        self.castle_rights
    }

    pub fn halfmoves(&self) -> u16 {
        self.halfmoves
    }

    pub fn fullmoves(&self) -> u16 {
        self.fullmoves
    }

    pub fn zobrist(&self) -> &Zobrist {
        &self.zobrist
    }

    pub fn mut_zobrist(&mut self) -> &mut Zobrist {
        &mut self.zobrist
    }

    pub fn zobrist_table(&self) -> &ZobristTable {
        &self.zobrist_table
    }

    pub fn encode(&self) -> EncodedState {
        EncodedState::new(self)
    }

    pub fn en_passant_capture_sq(&self) -> Option<Square> {
        match self.en_passant {
            Some(sq) => {
                if self.side_to_move == Side::White {
                    Some(sq.change_rank(sq.rank() - 1))
                } else {
                    Some(sq.change_rank(sq.rank() + 1))
                }
            }
            None => None,
        }
    }

    pub fn set_en_passant(&mut self, en_passant_sq: Square) {
        self.zobrist.hash_en_passant(self.en_passant);
        self.en_passant = Some(en_passant_sq);
        self.zobrist.hash_en_passant(Some(en_passant_sq));
    }

    pub fn remove_en_passant(&mut self) {
        self.zobrist.hash_en_passant(self.en_passant);
        self.en_passant = None;
    }

    pub fn remove_castle_rights(&mut self, side: Side, castle: Castle) {
        if self.castle_rights.can(side, castle) {
            self.zobrist.hash_castle_rights_single(side, castle);
            self.castle_rights = self.castle_rights.remove_rights(side, castle);
        }
    }

    pub fn remove_castle_rights_for_color(&mut self, side: Side) {
        if self.castle_rights.can(side, Castle::Kingside) {
            self.zobrist
                .hash_castle_rights_single(side, Castle::Kingside);
        }

        if self.castle_rights.can(side, Castle::Queenside) {
            self.zobrist
                .hash_castle_rights_single(side, Castle::Queenside);
        }
        self.castle_rights = self.castle_rights.remove_rights_for_color(side);
    }

    pub fn reset_halfmoves(&mut self) {
        self.halfmoves = 0;
    }

    pub fn increase_halfmoves(&mut self) {
        self.halfmoves += 1;
    }

    pub fn increase_fullmoves(&mut self) {
        self.fullmoves += 1;
    }

    pub fn update_side_to_move(&mut self) {
        self.side_to_move = self.side_to_move.opposite();
        self.zobrist.hash_side(self.side_to_move);
    }

    pub fn rollback_zobrist_table(&mut self, zobrist: Zobrist) {
        debug_assert!(
            self.zobrist_table
                .get(&zobrist.to_u64())
                .expect("rolled back zobrist table but zobrist was not stored in table")
                > &0u8
        );
        self.zobrist_table
            .entry(zobrist.to_u64())
            .and_modify(|count| *count -= 1);
    }

    pub fn push_to_zobrist_table(&mut self, zobrist: Zobrist) {
        self.zobrist_table
            .entry(zobrist.to_u64())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    pub fn is_draw_by_repetition(&self, last_zobrist: Zobrist) -> bool {
        let count_result = self.zobrist_table.get(&last_zobrist.to_u64());

        if let Some(count) = count_result {
            count > &2u8
        } else {
            false
        }
    }

    pub fn is_draw_by_halfmoves(&self) -> bool {
        self.halfmoves > 49
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "side to move: {}\ncastling rights: {}\nen-passant: {}\nhalfmoves: {}\nfullmoves: {}",
            self.side_to_move,
            self.castle_rights,
            self.en_passant.map_or("-".to_string(), |s| s.to_string()),
            self.halfmoves,
            self.fullmoves
        )
    }
}

#[derive(Clone, Copy)]
pub struct EncodedState(u32);

impl EncodedState {
    pub fn new(state: &State) -> EncodedState {
        EncodedState(
            state.side_to_move.to_u32() << 31
                | state.castle_rights.to_u32() << 27
                | state.en_passant.unwrap_or(square::NULL).to_u32() << 20
                | (state.halfmoves as u32) << 10
                | (state.fullmoves as u32),
        )
    }

    pub fn side_to_move(&self) -> Side {
        SIDE_MAP[(self.0 >> 31) as usize]
    }

    pub fn castle_rights(&self) -> CastleRights {
        CastleRights::new(((self.0 >> 27) & 15) as u8)
    }

    pub fn en_passant(&self) -> Option<Square> {
        let index = (self.0 >> 20) & 127;
        if index == 64 {
            None
        } else {
            Some(Square::new(index as usize))
        }
    }

    pub fn halfmoves(&self) -> u16 {
        ((self.0 >> 10) & 1023) as u16
    }

    pub fn fullmoves(&self) -> u16 {
        (self.0 & 1023) as u16
    }
}

#[cfg(test)]
pub mod test_display {

    use super::*;
    use crate::square::*;

    #[test]
    pub fn no1() {
        let state = State::new(
            Some(E4),
            Side::White,
            castle_rights::WHITE,
            0,
            0,
            Zobrist(0),
        );
        let expected = unindent::unindent(
            "
                                  side to move: white
                                  castling rights: KQ
                                  en-passant: e4
                                  halfmoves: 0
                                  fullmoves: 0",
        );

        assert_eq!(state.to_string(), expected);
    }
}

#[cfg(test)]
pub mod test_encoded_state {

    use super::*;
    use crate::square::*;

    #[test]
    pub fn no1() {
        let mut state = State::new(
            Some(E4),
            Side::White,
            castle_rights::WHITE,
            0,
            0,
            Zobrist(0),
        );
        let encoded_state = EncodedState::new(&state);
        state.decode_from(encoded_state);
        let expected = unindent::unindent(
            "
                                  side to move: white
                                  castling rights: KQ
                                  en-passant: e4
                                  halfmoves: 0
                                  fullmoves: 0",
        );

        println!("{}", state.to_string());

        assert_eq!(state.to_string(), expected);
    }

    #[test]
    pub fn no_en_passant() {
        let mut state = State::new(None, Side::White, castle_rights::WHITE, 0, 0, Zobrist(0));
        let encoded_state = EncodedState::new(&state);
        state.decode_from(encoded_state);
        let expected = unindent::unindent(
            "
                                  side to move: white
                                  castling rights: KQ
                                  en-passant: -
                                  halfmoves: 0
                                  fullmoves: 0",
        );

        println!("{}", state.to_string());

        assert_eq!(state.to_string(), expected);
    }
}

#[cfg(test)]
pub mod test_zobrist_state {
    use super::*;

    #[test]
    fn castle_rights_decode() {
        let mut zobrist = Zobrist(0);
        zobrist.hash_castle_rights_all(castle_rights::ALL);
        let expected = zobrist.clone();
        println!("expected: {}", expected);

        let mut state = State::new(None, Side::White, castle_rights::ALL, 0, 0, zobrist);
        assert_eq!(state.zobrist, expected);

        let encoded_state = state.encode();

        state.remove_castle_rights_for_color(Side::White);
        let mut only_b_castle_rights_zobrist = Zobrist(0);
        only_b_castle_rights_zobrist.hash_castle_rights_all(castle_rights::BLACK);
        assert_eq!(state.zobrist, only_b_castle_rights_zobrist);

        state.decode_from(encoded_state);

        assert_eq!(expected, state.zobrist);
    }
}
