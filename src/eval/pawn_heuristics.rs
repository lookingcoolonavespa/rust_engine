use crate::{bitboard::BB, side::Side};

pub fn king_safety() {}

pub fn passed_isolated_double_backward_pawns_count(
    bb_pawns: BB,
    bb_side: BB,
    side: Side,
) -> (u8, u8, u8, u8) {
    let mut passed_pawn_count = 0;
    let mut isolated_pawn_count = 0;
    let mut doubled_pawn_count = 0;
    let mut backwards_pawn_count = 0;

    let bb_pawns_side = bb_pawns & bb_side;
    for pawn_sq in bb_pawns_side.iter() {
        let doubled_pawn_bb = pawn_sq.file_mask() & bb_pawns_side ^ BB::new(pawn_sq);
        if doubled_pawn_bb.not_empty() {
            doubled_pawn_count += 1;
        }

        let bb_pawns_opp_side = bb_pawns & !bb_side;
        let is_passed_pawn =
            (bb_pawns_opp_side & (pawn_sq.file_mask() | pawn_sq.files_adjacent_mask())).empty();
        if is_passed_pawn {
            passed_pawn_count += 1;
        }

        let attached_pawns_bb = bb_pawns_side & pawn_sq.files_adjacent_mask();
        if attached_pawns_bb.empty() {
            isolated_pawn_count += 1;
        } else if attached_pawns_bb.count_ones() == 1 {
            let pawn_is_behind = if side == Side::White {
                attached_pawns_bb.bitscan() > pawn_sq
            } else {
                attached_pawns_bb.bitscan() < pawn_sq
            };

            if pawn_is_behind && doubled_pawn_bb.empty() {
                backwards_pawn_count += 1;
            }
        }
    }

    (
        passed_pawn_count,
        isolated_pawn_count,
        doubled_pawn_count / 2, // need to divide by two because doubled pawns are counted twice
        backwards_pawn_count,
    )
}

#[cfg(test)]
mod test_pawn_heuristics {
    use crate::{game::Game, piece_type::PieceType};

    use super::*;

    #[test]
    fn test_pos_1() {
        let fen = "4k3/2pppppp/8/8/P2PP3/2P1P3/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();

        let expected = (1, 1, 1, 1);
        let actual = passed_isolated_double_backward_pawns_count(
            game.position().bb_pieces()[PieceType::Pawn.to_usize()],
            game.position().bb_side(Side::White),
            Side::White,
        );

        assert_eq!(expected, actual);
    }
}
