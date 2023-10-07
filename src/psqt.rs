use crate::{
    bitboard::BOARD_LENGTH,
    piece_type::{BISHOP_ID, KING_ID, KNIGHT_ID, PAWN_ID, PIECE_TYPE_COUNT, QUEEN_ID, ROOK_ID},
    score::Score,
};
use raw_psqt::*;

// RawPsqt contains scores for 8 ranks of 4 files. Scores are flipped along the files to get the
// scores for the remaining files ie. the score of the first file along the rank also applies to
// the last file along the rank (score of a1 is equal to score of h1)
type RawPsqt = [Score; 32];
type Psqt = [Score; BOARD_LENGTH];
type PsqtTable = [Psqt; PIECE_TYPE_COUNT];

pub static PSQT: [PsqtTable; 2] = [generate_white_psqt(), generate_black_psqt()];

#[rustfmt::skip]
mod raw_psqt {
    use crate::score::Score;

    use super::{RawPsqt, Psqt};

    // psqt are calculated for white
    pub const KNIGHT_PSQT: RawPsqt = [
        Score(-50, -50, -50), Score(-20, -50, -50), Score(-20, -20, -20), Score(-30, -30, -30),
        Score(-20, -20, -20), Score(-10, -10, -10), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(20, 20, 20), Score(0, 20, 20),
        Score(0, 0, 0), Score(0, 0, 0), Score(10, 10, 10), Score(30, 30, 30),
        Score(0, 0, 0), Score(10, 10, 10), Score(20, 20, 20), Score(40, 40, 40),
        Score(0, 0, 0), Score(30, 30, 30), Score(30, 60, 60), Score(30, 30, 30),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
    ];
    pub const BISHOP_PSQT: RawPsqt = [
        Score(0, 0, 0), Score(-20, -20, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(20, 20, 20), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(20, 20, 20), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(20, 20, 20), Score(20, 20, 20),
        Score(0, 0, 0), Score(20, 20, 20), Score(0, 0, 0), Score(30, 30, 30),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
    ];
    pub const ROOK_PSQT: RawPsqt = [
        Score(0, 0, 0), Score(0, 0, 0), Score(30, 30, 0), Score(20, 30, 30),
        Score(0, 10, 0), Score(0, 0, 0), Score(0, 0, 0), Score(20, 20, 20),
        Score(-20, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(-20, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 20, 20), Score(0, 20, 20), Score(0, 20, 20), Score(0, 20, 20),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
    ];
    pub const QUEEN_PSQT: RawPsqt = [
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(110, 0, 0),
        Score(-20, -20, 0), Score(-10, -10, 0), Score(0, 20, 0), Score(0, 20, 0),
        Score(-10, 0, 0), Score(-10, 0, 0), Score(30, 0, 0), Score(-30, 0, 0),
        Score(-10, 0, 0), Score(-10, 0, 0), Score(-10, 0, 30), Score(-50, 20, 30),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 20), Score(-50, 20, 40),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 20, 20), Score(0, 20, 20), Score(0, 20, 20), Score(0, 20, 20),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
    ];
    pub const KING_PSQT: RawPsqt = [
        Score(0, 0, -20), Score(50, 0, 0), Score(0, 0, 0), Score(-20, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(-100, 0, 30), Score(-100, 0, 30),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 20), Score(0, 0, 20),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 20), Score(0, 0, 20),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
    ];
    pub const PAWN_PSQT: Psqt = [
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(10, 10, 0), Score(0, 0, 0), Score(20, 0, 0), Score(20, 0, 0), Score(20, 0, 0), Score(-30, 0, 0), Score(0, 0, 0), Score(20, 20, 0),
        Score(10, 10, 0), Score(0, 0, 0), Score(20, 20, 0), Score(50, 40, 0), Score(51, 40, 0), Score(0, 0, 0), Score(-50, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 30, 30), Score(0, 30, 30), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
        Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0), Score(0, 0, 0),
    ];
}

const fn get_piece_psqt(raw_psqt: [Score; 32]) -> [Score; BOARD_LENGTH] {
    let mut psqt = [Score(0, 0, 0); BOARD_LENGTH];
    let mut rank = 0;
    while rank < 8 {
        let mut file = 0;
        while file < 4 {
            let sq = (rank * 8) + file;
            psqt[sq] = raw_psqt[(rank * 4) + file];
            let sq = (rank * 8) + (7 - file);
            psqt[sq] = raw_psqt[(rank * 4) + file];

            file += 1;
        }

        rank += 1;
    }
    psqt
}

const fn flip_pqst(psqt: Psqt) -> Psqt {
    // flip the raw psqt so the psqt is valid for black
    let mut flipped_psqt = [Score(0, 0, 0); BOARD_LENGTH];

    let max_rank = 7;

    let mut rank = 0;
    while rank < 8 {
        let rank_idx = rank * 8;
        let flipped_rank_idx = (max_rank - rank) * 8;
        let mut file = 0;
        while file < 8 {
            flipped_psqt[flipped_rank_idx + file] = psqt[rank_idx + file];
            file += 1;
        }

        rank += 1;
    }

    flipped_psqt
}
const fn generate_white_psqt() -> PsqtTable {
    let mut psqt = [[Score(0, 0, 0); BOARD_LENGTH]; PIECE_TYPE_COUNT];

    psqt[PAWN_ID as usize] = PAWN_PSQT;
    psqt[KNIGHT_ID as usize] = get_piece_psqt(KNIGHT_PSQT);
    psqt[BISHOP_ID as usize] = get_piece_psqt(BISHOP_PSQT);
    psqt[ROOK_ID as usize] = get_piece_psqt(ROOK_PSQT);
    psqt[QUEEN_ID as usize] = get_piece_psqt(QUEEN_PSQT);
    psqt[KING_ID as usize] = get_piece_psqt(KING_PSQT);

    psqt
}
const fn generate_black_psqt() -> PsqtTable {
    let mut psqt = [[Score(0, 0, 0); BOARD_LENGTH]; PIECE_TYPE_COUNT];

    psqt[PAWN_ID as usize] = flip_pqst(PAWN_PSQT);
    psqt[KNIGHT_ID as usize] = flip_pqst(get_piece_psqt(KNIGHT_PSQT));
    psqt[BISHOP_ID as usize] = flip_pqst(get_piece_psqt(BISHOP_PSQT));
    psqt[ROOK_ID as usize] = flip_pqst(get_piece_psqt(ROOK_PSQT));
    psqt[QUEEN_ID as usize] = flip_pqst(get_piece_psqt(QUEEN_PSQT));
    psqt[KING_ID as usize] = flip_pqst(get_piece_psqt(KING_PSQT));

    psqt
}

#[cfg(test)]
mod test_psqt {
    use crate::side::Side;
    use crate::square::*;

    use super::*;

    #[test]
    fn print() {
        println!(
            "{}",
            PSQT[Side::White.to_usize()][QUEEN_ID as usize][D1.to_usize()]
        )
    }

    #[test]
    fn psqt_is_flipped_along_rank() {
        assert_eq!(
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][A1.to_usize()],
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][H1.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][B1.to_usize()],
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][G1.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][C1.to_usize()],
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][F1.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][QUEEN_ID as usize][E1.to_usize()],
            PSQT[Side::White.to_usize()][QUEEN_ID as usize][D1.to_usize()]
        );
    }

    #[test]
    fn pawn_psqt_is_flipped_for_black() {
        assert_eq!(
            PSQT[Side::White.to_usize()][PAWN_ID as usize][A3.to_usize()],
            PSQT[Side::Black.to_usize()][PAWN_ID as usize][A6.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][PAWN_ID as usize][B3.to_usize()],
            PSQT[Side::Black.to_usize()][PAWN_ID as usize][B6.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][PAWN_ID as usize][C3.to_usize()],
            PSQT[Side::Black.to_usize()][PAWN_ID as usize][C6.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][PAWN_ID as usize][E3.to_usize()],
            PSQT[Side::Black.to_usize()][PAWN_ID as usize][E6.to_usize()]
        );
    }

    #[test]
    fn psqt_is_flipped_for_black() {
        println!(
            "white: {:?}\nblack: {:?}",
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize],
            PSQT[Side::Black.to_usize()][KNIGHT_ID as usize]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][A3.to_usize()],
            PSQT[Side::Black.to_usize()][KNIGHT_ID as usize][A6.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][B3.to_usize()],
            PSQT[Side::Black.to_usize()][KNIGHT_ID as usize][B6.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][C3.to_usize()],
            PSQT[Side::Black.to_usize()][KNIGHT_ID as usize][C6.to_usize()]
        );
        assert_eq!(
            PSQT[Side::White.to_usize()][KNIGHT_ID as usize][E3.to_usize()],
            PSQT[Side::Black.to_usize()][KNIGHT_ID as usize][E6.to_usize()]
        );
    }
}
