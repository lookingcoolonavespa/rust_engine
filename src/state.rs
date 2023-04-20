pub mod castle_rights;
pub mod position;

use core::fmt;

use self::castle_rights::CastleRights;
use crate::{mv::castle::Castle, side::Side, square::Square};

pub struct State {
    en_passant: Option<Square>,
    side_to_move: Side,
    castle_rights: CastleRights,
    halfmoves: u16,
    fullmoves: u16,
}

impl State {
    pub fn new(
        en_passant: Option<Square>,
        side_to_move: Side,
        castle_rights: CastleRights,
        halfmoves: u16,
        fullmoves: u16,
    ) -> State {
        State {
            en_passant,
            side_to_move,
            castle_rights,
            halfmoves,
            fullmoves,
        }
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

    pub fn remove_castle_rights(&mut self, side: Side, castle: Castle) {
        self.castle_rights = self.castle_rights.remove_rights(side, castle);
    }

    pub fn remove_castle_rights_for_color(&mut self, side: Side) {
        self.castle_rights = self.castle_rights.remove_rights_for_color(side);
    }

    pub fn update(&mut self, en_passant: Option<Square>, should_increase_halfmoves: bool) {
        self.halfmoves = if should_increase_halfmoves {
            self.halfmoves + 1
        } else {
            0
        };
        self.fullmoves += 1;
        self.en_passant = en_passant;
        self.side_to_move = self.side_to_move.opposite();
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "side to move: {}\ncastling rights: {}\nen-passant: {}\nhalfmoves: {}\nfullmoves: {}",
            self.side_to_move.to_string(),
            self.castle_rights.to_string(),
            self.en_passant.map_or("-".to_string(), |s| s.to_string()),
            self.halfmoves.to_string(),
            self.fullmoves.to_string()
        )
    }
}

#[cfg(test)]
pub mod test_display {

    use super::*;
    use crate::square::*;

    #[test]
    pub fn no1() {
        let state = State::new(Some(E4), Side::White, castle_rights::WHITE, 0, 0);
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
