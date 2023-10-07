use std::fmt;

use subenum::subenum;

use crate::{
    bitboard::{self, BB},
    game::Game,
    move_gen::{
        check_legal::{is_en_passant_pinned_on_rank, pin_direction, LegalCheckPreprocessing},
        escape_check, pseudo_legal,
    },
    move_list::MoveList,
    mv::{EncodedMove, Move, PromotionMove},
    side::Side,
    square::{self, Square},
    state::State,
};

pub const PIECE_TYPE_COUNT: usize = 6;

pub const PAWN_ID: isize = 0;
pub const KNIGHT_ID: isize = 1;
pub const BISHOP_ID: isize = 2;
pub const ROOK_ID: isize = 3;
pub const QUEEN_ID: isize = 4;
pub const KING_ID: isize = 5;

#[subenum(PromoteType)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PieceType {
    Pawn = PAWN_ID,
    #[subenum(PromoteType)]
    Knight = KNIGHT_ID,
    #[subenum(PromoteType)]
    Bishop = BISHOP_ID,
    #[subenum(PromoteType)]
    Rook = ROOK_ID,
    #[subenum(PromoteType)]
    Queen = QUEEN_ID,
    King = KING_ID,
}

const fn generate_piece_type_map() -> [PieceType; PIECE_TYPE_COUNT] {
    let mut piece_type_map: [PieceType; PIECE_TYPE_COUNT] = [PieceType::Pawn; PIECE_TYPE_COUNT];
    piece_type_map[PAWN_ID as usize] = PieceType::Pawn;
    piece_type_map[KNIGHT_ID as usize] = PieceType::Knight;
    piece_type_map[BISHOP_ID as usize] = PieceType::Bishop;
    piece_type_map[ROOK_ID as usize] = PieceType::Rook;
    piece_type_map[QUEEN_ID as usize] = PieceType::Queen;
    piece_type_map[KING_ID as usize] = PieceType::King;

    return piece_type_map;
}

const fn generate_piece_type_score_map() -> [u32; PIECE_TYPE_COUNT] {
    let mut piece_type_score_map: [u32; PIECE_TYPE_COUNT] = [0; PIECE_TYPE_COUNT];
    piece_type_score_map[PAWN_ID as usize] = 100;
    piece_type_score_map[KNIGHT_ID as usize] = 300;
    piece_type_score_map[BISHOP_ID as usize] = 350;
    piece_type_score_map[ROOK_ID as usize] = 500;
    piece_type_score_map[QUEEN_ID as usize] = 900;
    piece_type_score_map[KING_ID as usize] = 10000;

    return piece_type_score_map;
}

pub const PIECE_TYPE_MAP: [PieceType; PIECE_TYPE_COUNT] = generate_piece_type_map();

pub const PIECE_TYPE_SCORE_MAP: [u32; PIECE_TYPE_COUNT] = generate_piece_type_score_map();

pub const PROMOTE_TYPE_ARR: [PromoteType; 4] = [
    PromoteType::Knight,
    PromoteType::Bishop,
    PromoteType::Rook,
    PromoteType::Queen,
];
const PIECE_CHARS: [char; PIECE_TYPE_COUNT] = ['p', 'n', 'b', 'r', 'q', 'k'];

impl PieceType {
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn to_usize(self) -> usize {
        self as usize
    }

    pub fn to_char(self) -> char {
        PIECE_CHARS[self.to_usize()]
    }

    pub fn score(self) -> u32 {
        PIECE_TYPE_SCORE_MAP[self.to_usize()]
    }

    pub fn pseudo_legal_moves_bb(
        self,
        from: Square,
        friendly_occupied: BB,
        enemy_occupied: BB,
        state: &State,
        side: Side,
        en_passant: Option<Square>,
    ) -> BB {
        match self {
            PieceType::Pawn => {
                pseudo_legal::pawn(from, friendly_occupied, enemy_occupied, en_passant, side)
            }
            PieceType::Knight => pseudo_legal::knight_attacks(from, friendly_occupied),
            PieceType::Bishop => {
                pseudo_legal::bishop_attacks(from, friendly_occupied, enemy_occupied)
            }
            PieceType::Rook => pseudo_legal::rook_attacks(from, friendly_occupied, enemy_occupied),
            PieceType::Queen => {
                pseudo_legal::queen_attacks(from, friendly_occupied, enemy_occupied)
            }
            PieceType::King => pseudo_legal::king_attacks(from, friendly_occupied),
        }
    }

    pub fn pseudo_legal_escape_moves_bb(
        self,
        from: Square,
        friendly_occupied: BB,
        enemy_occupied: BB,
        side: Side,
        en_passant: Option<Square>,
        legal_check_preprocessing: &LegalCheckPreprocessing,
        check_ray: BB,
    ) -> BB {
        match self {
            PieceType::Pawn => escape_check::pawn(
                from,
                friendly_occupied,
                enemy_occupied,
                en_passant,
                side,
                check_ray,
            ),
            PieceType::Knight => escape_check::knight(from, friendly_occupied, check_ray),
            PieceType::Bishop => {
                escape_check::bishop(from, friendly_occupied, enemy_occupied, check_ray)
            }
            PieceType::Rook => {
                escape_check::rook(from, friendly_occupied, enemy_occupied, check_ray)
            }
            PieceType::Queen => {
                escape_check::queen(from, friendly_occupied, enemy_occupied, check_ray)
            }
            PieceType::King => {
                escape_check::king(from, friendly_occupied, legal_check_preprocessing)
            }
        }
    }

    pub fn pseudo_legal_loud_moves_bb(
        self,
        from: Square,
        friendly_occupied: BB,
        enemy_occupied: BB,
        state: &State,
        side: Side,
        en_passant: Option<Square>,
    ) -> BB {
        match self {
            PieceType::Pawn => {
                pseudo_legal::pawn_loud(from, friendly_occupied, enemy_occupied, en_passant, side)
            }
            PieceType::Knight => pseudo_legal::knight_captures(from, enemy_occupied),
            PieceType::Bishop => {
                pseudo_legal::bishop_captures(from, friendly_occupied, enemy_occupied)
            }
            PieceType::Rook => pseudo_legal::rook_captures(from, friendly_occupied, enemy_occupied),
            PieceType::Queen => {
                pseudo_legal::queen_captures(from, friendly_occupied, enemy_occupied)
            }
            PieceType::King => pseudo_legal::king_captures(from, enemy_occupied),
        }
    }

    pub fn push_bb_to_move_list(
        self,
        mv_list: &mut MoveList,
        moves_bb: BB,
        from: Square,
        side: Side,
        enemy_occupied: BB,
        state: &State,
        en_passant: Option<Square>,
    ) {
        return match self {
            PieceType::Pawn => {
                let promote_rank_bb = bitboard::ROW_8 | bitboard::ROW_1;

                for to in moves_bb.iter() {
                    let is_en_passant = to == en_passant.unwrap_or(square::NULL);
                    let is_capture = enemy_occupied.is_set(to) || is_en_passant;

                    if promote_rank_bb.is_set(to) {
                        for promote_type in PROMOTE_TYPE_ARR.iter() {
                            mv_list.push_move(Move::Promotion(PromotionMove::new(
                                from,
                                to,
                                promote_type,
                                is_capture,
                            )))
                        }
                    } else {
                        let move_type = if pseudo_legal::is_double_pawn_push(from, to, side) {
                            Move::DoublePawnPush
                        } else if is_en_passant {
                            Move::EnPassant
                        } else {
                            Move::Pawn
                        };

                        mv_list.push_move(move_type(EncodedMove::new(
                            from,
                            to,
                            PieceType::Pawn,
                            is_capture,
                        )));
                    }
                }
            }
            PieceType::King => {
                mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                    Move::King(EncodedMove::new(from, to, self, enemy_occupied.is_set(to)))
                });
            }
            PieceType::Rook => {
                mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                    Move::Rook(EncodedMove::new(from, to, self, enemy_occupied.is_set(to)))
                });
            }
            _ => {
                mv_list.insert_moves(from, moves_bb, |from, to| -> Move {
                    Move::Piece(EncodedMove::new(from, to, self, enemy_occupied.is_set(to)))
                });
            }
        };
    }

    pub fn has_legal_moves(
        self,
        game: &Game,
        from: Square,
        friendly_occupied: BB,
        enemy_occupied: BB,
        side: Side,
        pinned_pieces_bb: BB,
        king_sq: Square,
        legal_check_preprocessing: &LegalCheckPreprocessing,
    ) -> bool {
        match self {
            PieceType::Pawn => {
                let en_passant = game.state().en_passant();
                let en_passant_capture_sq = game.state().en_passant_capture_sq();
                let mut pseudo_legal_moves =
                    pseudo_legal::pawn(from, friendly_occupied, enemy_occupied, en_passant, side);
                let pinned = pinned_pieces_bb.is_set(from);

                if en_passant.is_some()
                    && pseudo_legal_moves.is_set(en_passant.unwrap())
                    && is_en_passant_pinned_on_rank(
                        game.position(),
                        side,
                        from,
                        en_passant_capture_sq.unwrap(),
                        king_sq,
                    )
                {
                    pseudo_legal_moves ^= BB::new(en_passant.unwrap());
                }

                if pseudo_legal_moves.empty() {
                    return false;
                }

                return !pinned || (pseudo_legal_moves & pin_direction(from, king_sq)).not_empty();
            }
            PieceType::Knight => {
                let pseudo_legal_moves = pseudo_legal::knight_attacks(from, friendly_occupied);
                let pinned = pinned_pieces_bb.is_set(from);

                if pseudo_legal_moves.empty() || pinned {
                    return false;
                } else {
                    return true;
                }
            }
            PieceType::Bishop => {
                let pseudo_legal_moves =
                    pseudo_legal::bishop_attacks(from, friendly_occupied, enemy_occupied);
                let pinned = pinned_pieces_bb.is_set(from);

                if pseudo_legal_moves.empty() {
                    return false;
                }

                return !pinned || (pseudo_legal_moves & pin_direction(from, king_sq)).not_empty();
            }
            PieceType::Rook => {
                let pseudo_legal_moves =
                    pseudo_legal::rook_attacks(from, friendly_occupied, enemy_occupied);
                let pinned = pinned_pieces_bb.is_set(from);

                if pseudo_legal_moves.empty() {
                    return false;
                }

                return !pinned || (pseudo_legal_moves & pin_direction(from, king_sq)).not_empty();
            }
            PieceType::Queen => {
                let pseudo_legal_moves =
                    pseudo_legal::queen_attacks(from, friendly_occupied, enemy_occupied);
                let pinned = pinned_pieces_bb.is_set(from);

                if pseudo_legal_moves.empty() {
                    return false;
                }

                return !pinned || (pseudo_legal_moves & pin_direction(from, king_sq)).not_empty();
            }
            PieceType::King => {
                let safe_squares = pseudo_legal::king_attacks(from, friendly_occupied)
                    & !legal_check_preprocessing.controlled_squares_with_king_gone_bb();

                return safe_squares.not_empty();
            }
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl PromoteType {
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn to_usize(self) -> usize {
        self as usize
    }
}

impl TryFrom<char> for PromoteType {
    type Error = &'static str;

    fn try_from(v: char) -> Result<Self, Self::Error> {
        match v {
            'n' => Ok(PromoteType::Knight),
            'b' => Ok(PromoteType::Bishop),
            'r' => Ok(PromoteType::Rook),
            'q' => Ok(PromoteType::Queen),
            _ => {
                println!(
                    "PromoteType/TryFrom, encountered an valid piece, piece: {}",
                    v
                );
                Err("PromoteType/TryFrom, encountered an valid piece")
            }
        }
    }
}

impl TryFrom<char> for PieceType {
    type Error = &'static str;

    fn try_from(v: char) -> Result<Self, Self::Error> {
        match v {
            'p' => Ok(PieceType::Pawn),
            'n' => Ok(PieceType::Knight),
            'b' => Ok(PieceType::Bishop),
            'r' => Ok(PieceType::Rook),
            'q' => Ok(PieceType::Queen),
            'k' => Ok(PieceType::King),
            _ => {
                println!(
                    "PieceType/TryFrom, encountered an valid piece, piece: {}",
                    v
                );
                Err("PieceType/TryFrom, encountered an valid piece")
            }
        }
    }
}
