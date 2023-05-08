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
mod piece;
mod piece_type;
mod search;
mod side;
mod square;
mod state;
mod uci;
mod util;

fn main() {
    uci::main();
}
