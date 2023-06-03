#[cfg(test)]
extern crate unindent;

mod bitboard;
mod eval;
mod fen;
mod game;
mod move_gen;
mod move_list;
mod mv;
mod perft;
mod phase;
mod piece;
mod piece_type;
mod psqt;
mod score;
mod search;
mod side;
mod square;
mod state;
pub mod uci;
mod util;

fn main() {
    uci::main();
}
