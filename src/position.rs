use crate::attack_table::AttackTable;
use crate::bitboard;
use crate::bitboard::BB;
use crate::move_gen::pseudo_legal;
use crate::piece_type::PieceType;
use crate::side::*;

#[derive(Clone, PartialEq)]
pub struct Position {
    pub bb_sides: [BB; 2],
    pub bb_pieces: [BB; 6],
    attack_tables: AttackTable,
}
impl Position {
    pub fn new(bb_sides: [BB; 2], bb_pieces: [BB; 6]) -> Position {
        Position {
            bb_sides,
            bb_pieces,
            attack_tables: AttackTable::new(),
        }
    }
    pub fn occupied(self) -> BB {
        self.bb_sides[WHITE.to_usize()] | (self.bb_sides[BLACK.to_usize()])
    }

    pub fn attacks(self) {
        let mut attacks = bitboard::EMPTY;
        for (i, _) in self.bb_pieces.iter().enumerate() {
            let piece_type = PieceType::try_from(i).unwrap();
            attacks |= pseudo_legal::attacks_for_piece_type(&self, piece_type);
        }
    }
}
