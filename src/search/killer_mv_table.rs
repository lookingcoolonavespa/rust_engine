use crate::mv::Move;

use super::Depth;

type Table = Vec<[Option<Move>; 2]>;
pub struct KillerMoveTable(Table);

impl KillerMoveTable {
    pub fn new(size: Depth) -> KillerMoveTable {
        let table = vec![[None; 2]; size as usize];
        KillerMoveTable(table)
    }

    pub fn insert(&mut self, mv: Move, ply: usize) {
        let tmp = self.0[ply][0];
        self.0[ply][0] = Some(mv);
        self.0[ply][1] = tmp;
    }

    pub fn get_first(&self, ply: usize) -> Option<Move> {
        self.0[ply][0]
    }

    pub fn get_second(&self, ply: usize) -> Option<Move> {
        self.0[ply][1]
    }
}
