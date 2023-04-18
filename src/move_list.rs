use crate::{bitboard::BB, mv::Move, square::Square};

pub struct MoveList(Vec<Move>);

impl MoveList {
    pub fn new() -> MoveList {
        MoveList(Vec::with_capacity(60))
    }

    pub fn iter(&self) -> std::slice::Iter<Move> {
        self.0.iter()
    }

    pub fn insert_moves<F: Fn(Square, Square) -> Move>(&mut self, from: Square, target: BB, f: F) {
        for to in target.iter() {
            self.0.push(f(from, to))
        }
    }

    pub fn push_move(&mut self, mv: Move) {
        self.0.push(mv)
    }
}
