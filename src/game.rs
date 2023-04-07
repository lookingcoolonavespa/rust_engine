use core::fmt;

use crate::{
    bitboard::BB,
    piece_type::PieceType,
    position::Position,
    side::{self, Side},
    square::Square,
    state::State,
    util::grid_to_string,
};

pub struct Game {
    position: Position,
    state: State,
}

impl Game {
    pub fn at(self, sq: Square) -> Option<(PieceType, Side)> {
        let sq_bb = BB::new(sq);
        for (i, bb) in self.position.bb_pieces.iter().enumerate() {
            if (bb.to_owned() & sq_bb).not_empty() {
                let piece_type = PieceType::try_from(i).unwrap();
                let side = if (bb.to_owned() & self.position.bb_sides[side::WHITE.to_usize()])
                    .not_empty()
                {
                    side::WHITE
                } else {
                    side::BLACK
                };
                return Some((piece_type, side));
            }
        }

        None
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let props = vec![
            ("side to move", self.state.side_to_move.to_str().to_string()),
            (" castling rights", self.state.castle_rights.to_string()),
            (
                "en-passant",
                self.state
                    .en_passant
                    .map_or("-".to_string(), |s| s.to_string()),
            ),
            ("half-move clock", self.state.halfmoves.to_string()),
            ("full-move number", self.state.fullmoves.to_string()),
            ("FEN", self.to_fen()),
        ];
        let s = grid_to_string(|sq: Square| -> char {
            let (pc, side) = self.at(sq);
            if pc.is_none() {
                '.'
            } else {
                pc.to_char()
            }
        });

        write!(f, "{}", &s)
    }
}
