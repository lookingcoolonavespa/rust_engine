use std::fmt;

use crate::bitboard::BB;

#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    Opening,
    Middle,
    End,
}

const ENDGAME_TRANSITION_PIECES_COUNT: u32 = 7;
const ENDGAME_TRANSITION_PAWN_COUNT: u32 = 3;
impl Phase {
    pub fn get(bb_occupied: BB, bb_pawns: BB, fullmoves: u16) -> Phase {
        let bb_pieces = bb_occupied & !bb_pawns;

        if bb_pieces.count_ones() < ENDGAME_TRANSITION_PIECES_COUNT
            || bb_pawns.count_ones() <= ENDGAME_TRANSITION_PAWN_COUNT
        {
            return Phase::End;
        }

        if fullmoves > 12 {
            return Phase::Middle;
        }

        Phase::Opening
    }
}

#[cfg(test)]
mod test {
    use crate::{fen::STARTING_POSITION_FEN, game::Game, piece_type::PAWN_ID};

    use super::*;

    #[test]
    fn phase_get() {
        let result = Game::from_fen(STARTING_POSITION_FEN);
        assert!(result.is_ok());
        let game = result.unwrap();

        assert_eq!(
            Phase::get(
                game.position().bb_occupied(),
                game.position().bb_pieces()[PAWN_ID as usize],
                game.state().fullmoves()
            )
            .to_string(),
            "Opening"
        );

        assert_eq!(
            game.position().phase().to_string(),
            Phase::get(
                game.position().bb_occupied(),
                game.position().bb_pieces()[PAWN_ID as usize],
                game.state().fullmoves()
            )
            .to_string()
        );
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Phase::Opening => "Opening",
                Phase::Middle => "Middle",
                Phase::End => "End",
            }
        )
    }
}
