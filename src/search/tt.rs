use std::collections::HashMap;

use crate::{game::Game, mv::Move};

const TABLE_SIZE: u64 = 0x100000 * 64;
const ENTRY_SIZE: u64 = 24; // TableEntry is 24 bytes

pub struct TranspositionTable {
    map: HashMap<u32, TableEntry>,
    age: u16,
}

struct TableEntry {
    zobrist: u64,
    depth: u8,
    flag: TtFlag,
    eval: i32,
    mv: Option<Move>,
    age: u16,
}

pub enum TtFlag {
    Exact,
    Beta,
    Alpha,
}

impl TableEntry {
    pub fn replace(
        &mut self,
        zobrist: u64,
        depth: u8,
        flag: TtFlag,
        eval: i32,
        mv: Option<Move>,
        age: u16,
    ) {
        self.zobrist = zobrist;
        self.depth = depth;
        self.flag = flag;
        self.eval = eval;
        self.mv = mv;
        self.age = age;
    }
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            map: HashMap::with_capacity((TABLE_SIZE / ENTRY_SIZE) as usize),
            age: 0,
        }
    }

    pub fn update_age(&mut self, game: &Game) {
        self.age = game.state().fullmoves();
    }

    fn get_table_key(&self, zobrist: u64) -> u32 {
        (zobrist % (TABLE_SIZE / ENTRY_SIZE)) as u32
    }

    pub fn store(&mut self, zobrist: u64, depth: u8, flag: TtFlag, eval: i32, mv: Option<Move>) {
        let key = self.get_table_key(zobrist);

        let entry_result = self.map.get_mut(&key);
        if let Some(entry) = entry_result {
            if self.age > entry.age || depth >= entry.depth {
                entry.replace(zobrist, depth, flag, eval, mv, self.age);
            }
        } else {
            self.map.insert(
                key,
                TableEntry {
                    zobrist,
                    depth,
                    flag,
                    eval,
                    mv,
                    age: self.age,
                },
            );
        }
    }

    pub fn probe_val(&self, zobrist: u64, depth: u8, alpha: i32, beta: i32) -> Option<i32> {
        let key = self.get_table_key(zobrist);

        let entry_result = self.map.get(&key);
        if let Some(entry) = entry_result {
            if zobrist != entry.zobrist || depth > entry.depth {
                return None;
            }

            match entry.flag {
                TtFlag::Exact => {
                    return Some(entry.eval);
                }
                TtFlag::Alpha => {
                    if entry.eval <= alpha {
                        // evaluation of the position is smaller than the value of entry.eval
                        return Some(entry.eval);
                    }
                }
                TtFlag::Beta => {
                    // evaluation of the position is at least the value of entry.eval
                    if entry.eval >= beta {
                        return Some(entry.eval);
                    }
                }
            }
        }

        None
    }

    pub fn probe_move(&self, zobrist: u64, depth: u8) -> Option<Move> {
        let key = self.get_table_key(zobrist);
        let entry_result = self.map.get(&key);
        if let Some(entry) = entry_result {
            if zobrist != entry.zobrist
                || depth > entry.depth
                || !matches!(entry.flag, TtFlag::Exact | TtFlag::Beta)
            {
                return None;
            }

            return entry.mv;
        }

        None
    }
}

#[cfg(test)]
pub mod test_tt {
    use std::mem;

    use super::*;

    #[ignore]
    #[test]
    fn print_size_of_table_entry() {
        println!("{}", mem::size_of::<TableEntry>());
    }
}
