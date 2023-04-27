use crate::{
    bitboard::{self, BB, FILE_A, FILE_B, FILE_G, FILE_H, NOT_FILE_A, NOT_FILE_H},
    side::Side,
};

fn occluded_east_slider_fill(pieces_bb: BB, occupied: BB) -> BB {
    let empty = !occupied;
    let mut prop = empty & bitboard::NOT_FILE_A;
    let mut gen = pieces_bb;

    // need to shift empty to mask off blocked squares behind piece
    gen |= prop & (gen << 1);
    prop &= prop << 1;
    gen |= prop & (gen << 2);
    prop &= prop << 2;
    gen |= prop & (gen << 4);

    gen
}
fn occluded_north_slider_fill(pieces_bb: BB, occupied: BB) -> BB {
    let empty = !occupied;
    let mut prop = empty;
    let mut gen = pieces_bb;

    // need to shift empty to mask off blocked squares behind piece
    gen |= prop & (gen << 8);
    prop &= prop << 8;
    gen |= prop & (gen << 16);
    prop &= prop << 16;
    gen |= prop & (gen << 32);

    gen
}

fn occluded_south_slider_fill(pieces_bb: BB, occupied: BB) -> BB {
    let empty = !occupied;
    let mut prop = empty;
    let mut gen = pieces_bb;

    // need to shift empty to mask off blocked squares behind piece
    gen |= prop & (gen >> 8);
    prop &= prop >> 8;
    gen |= prop & (gen >> 16);
    prop &= prop >> 16;
    gen |= prop & (gen >> 32);

    gen
}

fn occluded_west_slider_fill(pieces_bb: BB, occupied: BB) -> BB {
    let empty = !occupied;
    let mut prop = empty & bitboard::NOT_FILE_H;
    let mut gen = pieces_bb;

    // need to shift empty to mask off blocked squares behind piece
    gen |= prop & (gen >> 1);
    prop &= prop >> 1;
    gen |= prop & (gen >> 2);
    prop &= prop >> 2;
    gen |= prop & (gen >> 4);

    gen
}

fn occluded_south_east_slider_fill(pieces_bb: BB, occupied: BB) -> BB {
    let empty = !occupied;
    let mut prop = empty & bitboard::NOT_FILE_A;
    let mut gen = pieces_bb;

    // need to shift empty to mask off blocked squares behind piece
    gen |= prop & (gen >> 7);
    prop &= prop >> 7;
    gen |= prop & (gen >> 14);
    prop &= prop >> 14;
    gen |= prop & (gen >> 28);

    gen
}
fn occluded_north_east_slider_fill(pieces_bb: BB, occupied: BB) -> BB {
    let empty = !occupied;
    let mut prop = empty & bitboard::NOT_FILE_A;
    let mut gen = pieces_bb;

    // need to shift empty to mask off blocked squares behind piece
    gen |= prop & (gen << 9);
    prop &= prop << 9;
    gen |= prop & (gen << 18);
    prop &= prop << 18;
    gen |= prop & (gen << 36);

    gen
}

fn occluded_south_west_slider_fill(pieces_bb: BB, occupied: BB) -> BB {
    let empty = !occupied;
    let mut prop = empty & bitboard::NOT_FILE_H;
    let mut gen = pieces_bb;

    // need to shift empty to mask off blocked squares behind piece
    gen |= prop & (gen >> 9);
    prop &= prop >> 9;
    gen |= prop & (gen >> 18);
    prop &= prop >> 18;
    gen |= prop & (gen >> 36);

    gen
}

fn occluded_north_west_slider_fill(pieces_bb: BB, occupied: BB) -> BB {
    let empty = !occupied;
    let mut prop = empty & bitboard::NOT_FILE_H;
    let mut gen = pieces_bb;

    // need to shift empty to mask off blocked squares behind piece
    gen |= prop & (gen << 7);
    prop &= prop << 7;
    gen |= prop & (gen << 14);
    prop &= prop << 14;
    gen |= prop & (gen << 28);

    gen
}

// need to shift occluded_fill bb to include closest blocker
fn west_slider_attacks(pieces_bb: BB, occupied: BB) -> BB {
    (occluded_west_slider_fill(pieces_bb, occupied) >> 1) & bitboard::NOT_FILE_H
}

fn east_slider_attacks(pieces_bb: BB, occupied: BB) -> BB {
    (occluded_east_slider_fill(pieces_bb, occupied) << 1) & bitboard::NOT_FILE_A
}

fn north_slider_attacks(pieces_bb: BB, occupied: BB) -> BB {
    occluded_north_slider_fill(pieces_bb, occupied) << 8
}

fn south_slider_attacks(pieces_bb: BB, occupied: BB) -> BB {
    occluded_south_slider_fill(pieces_bb, occupied) >> 8
}

fn south_west_slider_attacks(pieces_bb: BB, occupied: BB) -> BB {
    (occluded_south_west_slider_fill(pieces_bb, occupied) >> 9) & bitboard::NOT_FILE_H
}

fn south_east_slider_attacks(pieces_bb: BB, occupied: BB) -> BB {
    (occluded_south_east_slider_fill(pieces_bb, occupied) >> 7) & bitboard::NOT_FILE_A
}

fn north_west_slider_attacks(pieces_bb: BB, occupied: BB) -> BB {
    (occluded_north_west_slider_fill(pieces_bb, occupied) << 7) & bitboard::NOT_FILE_H
}

fn north_east_slider_attacks(pieces_bb: BB, occupied: BB) -> BB {
    (occluded_north_east_slider_fill(pieces_bb, occupied) << 9) & bitboard::NOT_FILE_A
}

pub fn file_rank_attacks(pieces_bb: BB, occupied: BB) -> BB {
    north_slider_attacks(pieces_bb, occupied)
        | south_slider_attacks(pieces_bb, occupied)
        | east_slider_attacks(pieces_bb, occupied)
        | west_slider_attacks(pieces_bb, occupied)
}

pub fn diagonal_attacks(pieces_bb: BB, occupied: BB) -> BB {
    north_west_slider_attacks(pieces_bb, occupied)
        | north_east_slider_attacks(pieces_bb, occupied)
        | south_east_slider_attacks(pieces_bb, occupied)
        | south_west_slider_attacks(pieces_bb, occupied)
}

pub fn knight_jumps(knights_bb: BB) -> BB {
    let right_one = (knights_bb << 1) & NOT_FILE_A;
    let right_two = (knights_bb << 2) & !(FILE_A | FILE_B);
    let left_one = (knights_bb >> 1) & NOT_FILE_H;
    let left_two = (knights_bb >> 2) & !(FILE_H | FILE_G);

    let attacks_one = right_one | left_one;
    let attacks_two = right_two | left_two;

    attacks_one << 16 | attacks_one >> 16 | attacks_two << 8 | attacks_two >> 8
}

pub fn pawn_controlled_squares(pawns_bb: BB, side: Side) -> BB {
    let right_attack = if side == Side::White {
        (pawns_bb << 9) & NOT_FILE_A
    } else {
        (pawns_bb >> 7) & NOT_FILE_A
    };
    let left_attack = if side == Side::White {
        (pawns_bb << 7) & NOT_FILE_H
    } else {
        (pawns_bb >> 9) & NOT_FILE_H
    };

    left_attack | right_attack
}

#[cfg(test)]
pub mod test_pawn_attacks {
    use super::*;
    use crate::{game::Game, piece_type::PieceType};

    #[test]
    fn pawn_attacks_1() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let pawns_bb = game.position().bb_pc(PieceType::Pawn, Side::Black);
        let attacks = pawn_controlled_squares(pawns_bb, Side::Black);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|########|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn pawn_attacks_2() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let pawns_bb = game.position().bb_pc(PieceType::Pawn, Side::White);
        let attacks = pawn_controlled_squares(pawns_bb, Side::White);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|########|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }
}
#[cfg(test)]
pub mod test_knight_jumps {
    use super::*;
    use crate::{game::Game, piece_type::PieceType};

    #[test]
    fn knight_jumps_1() {
        let fen = "6k1/N7/8/8/8/6N1/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let knights_bb = game.position().bb_pc(PieceType::Knight, Side::White);
        let attacks = knight_jumps(knights_bb);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|..#.....|8
            7|........|7
            6|..#.....|6
            5|.#...#.#|5
            4|....#...|4
            3|........|3
            2|....#...|2
            1|.....#.#|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }
}

#[cfg(test)]
pub mod test_slider_attacks {
    use super::*;
    use crate::{game::Game, side};

    #[test]
    fn south_slider_attacks_1() {
        let fen = "2k5/6R1/8/8/8/6b1/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (_, non_diag_attackers) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = south_slider_attacks(non_diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|......#.|6
            5|......#.|5
            4|......#.|4
            3|......#.|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn north_slider_attacks_1() {
        let fen = "2k5/8/6b1/8/8/8/6R1/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (_, non_diag_attackers) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = north_slider_attacks(non_diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|......#.|6
            5|......#.|5
            4|......#.|4
            3|......#.|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn west_slider_attacks_1() {
        let fen = "2k5/8/8/8/8/8/2b3R1/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (_, non_diag_attackers) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = west_slider_attacks(non_diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|..####..|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn west_slider_attacks_edge_of_the_board() {
        let fen = "4k3/8/8/8/8/R7/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (_, non_diag_attackers) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = west_slider_attacks(non_diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn east_slider_attacks_1() {
        let fen = "4k3/8/8/8/8/8/2R5/6K1 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (_, non_diag_attackers) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = east_slider_attacks(non_diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|...#####|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn east_slider_attacks_on_edge_of_board() {
        let fen = "4k3/8/8/8/8/7R/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (_, non_diag_attackers) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = east_slider_attacks(non_diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn south_east_slider_attacks_1() {
        let fen = "4k3/1b6/8/8/8/8/8/4K2B w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::Black);
        let occupied = game.position().bb_occupied();
        let attacks = south_east_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|..#.....|6
            5|...#....|5
            4|....#...|4
            3|.....#..|3
            2|......#.|2
            1|.......#|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn south_east_slider_attacks_on_edge_of_board() {
        let fen = "4k3/8/7b/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::Black);
        let occupied = game.position().bb_occupied();
        let attacks = south_east_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn south_west_slider_attacks_1() {
        let fen = "4k3/6b1/8/8/8/8/8/4K2B w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::Black);
        let occupied = game.position().bb_occupied();
        let attacks = south_west_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|.....#..|6
            5|....#...|5
            4|...#....|4
            3|..#.....|3
            2|.#......|2
            1|#.......|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn south_west_slider_attacks_2() {
        let fen = "rn1qkbnr/p1pppppp/b7/1p6/5P2/P7/1PPPP1PP/RNBQKBNR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let bishops_bb = game
            .position()
            .bb_pc(crate::piece_type::PieceType::Bishop, Side::Black);
        let occupied = game.position().bb_occupied();
        let attacks = south_west_slider_attacks(bishops_bb, occupied);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|....#...|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn south_west_slider_attacks_on_edge_of_board() {
        let fen = "4k3/8/b7/8/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let bishops_bb = game
            .position()
            .bb_pc(crate::piece_type::PieceType::Bishop, Side::Black);
        let occupied = game.position().bb_occupied();
        let attacks = south_west_slider_attacks(bishops_bb, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn north_west_slider_attacks_on_edge_of_board() {
        let fen = "4k3/8/8/8/B7/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = north_west_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }
    #[test]
    fn north_west_slider_attacks_1() {
        let fen = "4k3/8/8/8/8/8/8/4K2B w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = north_west_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|#.......|8
            7|.#......|7
            6|..#.....|6
            5|...#....|5
            4|....#...|4
            3|.....#..|3
            2|......#.|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn north_west_slider_attacks_2() {
        let fen = "4k3/8/8/8/8/8/6b1/4K2B w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = north_west_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|......#.|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn north_east_slider_attacks_on_edge_of_board() {
        let fen = "4k3/8/8/7B/8/8/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = north_east_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|........|6
            5|........|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn north_east_slider_attacks_1() {
        let fen = "4k3/8/5n2/8/8/8/8/B3K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = north_east_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|.....#..|6
            5|....#...|5
            4|...#....|4
            3|..#.....|3
            2|.#......|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn north_east_slider_attacks_2() {
        let fen = "4k3/8/5n2/5n2/8/8/8/BB2K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = north_east_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|........|8
            7|........|7
            6|.....#..|6
            5|....##..|5
            4|...##...|4
            3|..##....|3
            2|.##.....|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }

    #[test]
    fn north_east_slider_attacks_3() {
        let fen = "4k3/8/8/8/8/1BB5/8/4K3 w - - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::White);
        let occupied = game.position().bb_occupied();
        let attacks = north_east_slider_attacks(diag_attackers, occupied);

        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|......##|8
            7|.....##.|7
            6|....##..|6
            5|...##...|5
            4|..##....|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }
}

#[cfg(test)]
pub mod test_diagonal_attacks {
    use crate::game::Game;

    use super::*;

    #[test]
    fn test_1() {
        let fen = "rn1qkbnr/p1pppppp/b7/1p6/5P2/P7/1PPPP1PP/RNBQKBNR w KQkq - 0 1";
        let result = Game::from_fen(fen);
        assert!(result.is_ok());
        let game = result.unwrap();
        let (diag_attackers, _) = game.position().bb_sliders(Side::Black);
        let occupied = game.position().bb_occupied();
        let attacks = diagonal_attacks(diag_attackers, occupied);
        let north_west = north_west_slider_attacks(diag_attackers, occupied);
        let north_east = north_east_slider_attacks(diag_attackers, occupied);
        let south_east = south_east_slider_attacks(diag_attackers, occupied);
        let south_west = south_west_slider_attacks(diag_attackers, occupied);

        println!("{}", north_west);
        println!("{}", north_east);
        println!("{}", south_west);
        println!("{}", south_east);
        println!("{}", attacks);

        let expected = unindent::unindent(
            "
              ABCDEFGH
            8|..#.....|8
            7|.##.#.#.|7
            6|........|6
            5|.#......|5
            4|........|4
            3|........|3
            2|........|2
            1|........|1
              ABCDEFGH
            ",
        );
        assert_eq!(attacks.to_string(), expected);
    }
}
