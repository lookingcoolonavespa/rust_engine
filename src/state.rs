use core::fmt;

use crate::{castle_rights::CastleRights, side::Side, square::Square};

pub struct State {
    pub en_passant: Option<Square>,
    pub side_to_move: Side,
    pub castle_rights: CastleRights,
    pub halfmoves: u16,
    pub fullmoves: u16,
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
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let props = vec![
            ("side to move", self.side_to_move.to_str().to_string()),
            (" castling rights", self.castle_rights.to_string()),
            (
                "en-passant",
                self.en_passant.map_or("-".to_string(), |s| s.to_string()),
            ),
            ("half-move clock", self.halfmoves.to_string()),
            ("full-move number", self.fullmoves.to_string()),
        ];

        write!(f, "{}", &s)
    }
}
