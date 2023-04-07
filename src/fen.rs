use crate::{
    bitboard::{self, BB},
    castle_rights::{self, CastleRights},
    piece_type::PieceType,
    position::Position,
    side::{BLACK, SIDE_MAP, WHITE},
    square::{self, Square},
    state::State,
};

pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QqKk - 0 1";

pub fn load_fen(fen: &str) -> Result<(Position, State), String> {
    let mut fen_state = fen.split(" ");

    let fen_board = fen_state.next().ok_or("fen string is empty")?;
    let (bb_sides, bb_pieces) = parse_fen_board(fen_board)?;

    let fen_side_to_move = fen_state.next().ok_or("fen string is missing fields")?;
    let side_to_move = SIDE_MAP
        .get(fen_side_to_move)
        .ok_or("side to move is an invalid color")?;

    let fen_castle = fen_state.next().ok_or("fen string is missing fields")?;
    let castle_rights = parse_fen_castle(fen_castle)?;

    let fen_en_passant = fen_state.next().ok_or("fen string is missing fields")?;
    let en_passant = parse_fen_en_passant(fen_en_passant)?;

    let fen_halfmoves = fen_state.next().ok_or("fen string is missing fields")?;
    let halfmoves = fen_halfmoves
        .chars()
        .next()
        .unwrap()
        .to_digit(10)
        .ok_or("halfmoves is not a number")?;

    let fen_fullmoves = fen_state.next().ok_or("fen string is missing fields")?;
    let fullmoves = fen_fullmoves
        .chars()
        .next()
        .unwrap()
        .to_digit(10)
        .ok_or("fullmoves is not a number")?;

    Ok((
        Position::new(bb_sides, bb_pieces),
        State::new(
            en_passant,
            side_to_move.to_owned(),
            castle_rights,
            halfmoves as u16,
            fullmoves as u16,
        ),
    ))
}

fn parse_fen_board(fen_board: &str) -> Result<([BB; 2], [BB; 6]), String> {
    let bb_pieces = [
        bitboard::EMPTY,
        bitboard::EMPTY,
        bitboard::EMPTY,
        bitboard::EMPTY,
        bitboard::EMPTY,
        bitboard::EMPTY,
    ];

    let bb_sides = [bitboard::EMPTY, bitboard::EMPTY];

    let mut rank = 7;
    let mut file = 0;
    for c in fen_board.chars() {
        if c == '/' {
            file = 0;
            rank -= 1;
            continue;
        }

        if c.is_numeric() {
            file += c.to_digit(10).unwrap();
        }

        let piece_lowercase = c
            .to_lowercase()
            .next()
            .ok_or("invalid character in fen board")?;
        let piece_type: PieceType = PieceType::try_from(piece_lowercase)?;

        if rank > 7 || rank < 0 || file > 7 || rank < 0 {
            return Err("fen board contains too many files or ranks".to_string());
        }

        let sq = Square::from(rank as usize, file as usize);
        let sq_bb = BB::new(sq);
        bb_pieces[piece_type.to_usize()] |= sq_bb;

        let side = if piece_lowercase == c { BLACK } else { WHITE };
        bb_sides[side.to_usize()] |= sq_bb;
    }

    Ok((bb_sides, bb_pieces))
}

fn parse_fen_castle(fen_castle: &str) -> Result<CastleRights, String> {
    if fen_castle == "-" {
        return Ok(castle_rights::NONE);
    }

    let castle_rights = castle_rights::NONE;
    for c in fen_castle.chars() {
        if c == 'K' {
            castle_rights.set(castle_rights::KINGSIDE & castle_rights::WHITE);
        } else if c == 'Q' {
            castle_rights.set(castle_rights::QUEENSIDE & castle_rights::WHITE);
        } else if c == 'k' {
            castle_rights.set(castle_rights::KINGSIDE & castle_rights::BLACK);
        } else if c == 'q' {
            castle_rights.set(castle_rights::QUEENSIDE & castle_rights::BLACK);
        } else {
            return Err("invalid character in castle rights".to_string());
        }
    }

    Ok(castle_rights)
}

fn parse_fen_en_passant(fen_en_passant: &str) -> Result<Option<Square>, String> {
    if fen_en_passant == "-" {
        return Ok(None);
    }

    if fen_en_passant.len() != 2 {
        return Err("invalid en passant square".to_string());
    }

    let file_char: char = fen_en_passant.chars().nth(0).unwrap();
    let rank_char: char = fen_en_passant.chars().nth(1).unwrap();

    if !square::RANKS.contains(&rank_char) {
        return Err("invalid en passant square".to_string());
    }
    if !square::FILES.contains(&file_char) {
        return Err("invalid en passant square".to_string());
    }

    let file = file_char as usize - 'a' as usize;
    let rank = rank_char as usize - '1' as usize;
    let en_passant_sq = Square::from(rank, file);

    Ok(Some(en_passant_sq))
}

#[cfg(test)]
pub mod fen_test {}
