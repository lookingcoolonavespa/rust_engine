use crate::mv::Move;

use super::SEARCH_DEPTH;

pub struct KillerMoveTable([[Option<Move>; 2]; SEARCH_DEPTH as usize]);

impl KillerMoveTable {
    pub fn new() -> KillerMoveTable {
        KillerMoveTable([[None; 2]; SEARCH_DEPTH as usize])
    }

    pub fn insert(&mut self, mv: Move, ply: usize) {
        debug_assert!(ply <= SEARCH_DEPTH as usize);
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
