#[cfg(test)]
use crate::bitboard::{self, BB};

use crate::square::Square;

pub fn grid_to_string<F: Fn(Square) -> char>(char_at: F) -> String {
    let mut string = "  ABCDEFGH\n".to_string();

    let row_chars = ['1', '2', '3', '4', '5', '6', '7', '8'];
    for row in (0..8).rev() {
        string += &format!("{}|", row_chars[row]);
        for col in 0..8 {
            string.push(char_at(Square::from(row, col)));
        }
        string += &format!("|{}\n", row_chars[row]);
    }

    string + &"  ABCDEFGH\n".to_string()
}

#[cfg(test)]
pub fn get_bb_from_array_of_squares(sq_arr: &[Square]) -> BB {
    let mut bb = bitboard::EMPTY;
    for sq in sq_arr {
        bb |= BB::new(sq);
    }
    bb
}
