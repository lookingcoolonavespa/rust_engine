use core::fmt;

use crate::bitboard::{BB, BISHOP_RAYS, FILE_A, FILE_H, KNIGHT_JUMPS, ROOK_RAYS, ROW_1};

pub type Internal = usize;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct Square(pub Internal);

impl Square {
    pub fn new(sq: Internal) -> Square {
        Square(sq)
    }

    pub fn from(rank: usize, file: usize) -> Square {
        Square((rank * 8) + file)
    }

    pub fn to_usize(self) -> usize {
        self.0
    }

    pub fn to_u16(self) -> u16 {
        self.0 as u16
    }

    pub fn to_u8(self) -> u8 {
        self.0 as u8
    }

    pub fn distance(self, sq: Square) -> u32 {
        let diff = self.0 as i32 - sq.0 as i32;
        diff.unsigned_abs()
    }

    pub fn to_u32(self) -> u32 {
        self.0 as u32
    }

    pub fn change_rank(self, rank: Internal) -> Square {
        assert!(rank < 8);
        Square((self.0 & 7) | (rank * 8))
    }

    pub fn rank_down(self) -> Square {
        assert!(self.0 > 7);
        Square(self.0 - 8)
    }

    pub fn rank_up(self) -> Square {
        assert!(self.0 < 57);
        Square(self.0 + 8)
    }

    pub fn file(&self) -> usize {
        self.0 & 7
    }

    pub fn file_mask(self) -> BB {
        FILE_A << (self.0 & 7)
    }

    pub fn files_adjacent_mask(self) -> BB {
        let file_mask = self.file_mask();
        let left_file_mask = file_mask >> 1 & !FILE_H;
        let right_file_mask = file_mask << 1 & !FILE_A;

        left_file_mask | right_file_mask
    }

    pub fn rank(&self) -> usize {
        self.0 >> 3
    }

    pub fn rank_mask(self) -> BB {
        ROW_1 << (8 * (self.0 >> 3))
    }

    pub fn diagonal_mask(self) -> BB {
        DIAGONALS[self.0]
    }

    pub fn anti_diagonal_mask(self) -> BB {
        ANTI_DIAGONALS[self.0]
    }

    pub fn bishop_rays(self) -> BB {
        BISHOP_RAYS[self.0]
    }

    pub fn rook_rays(self) -> BB {
        ROOK_RAYS[self.0]
    }

    pub fn knight_jumps(self) -> BB {
        KNIGHT_JUMPS[self.0]
    }

    pub fn is_light_sq(self) -> bool {
        self.0 % 2 == 0
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self == &NULL {
            return write!(f, "null");
        }
        let file_char = FILES[self.file()];
        let rank_char = RANKS[self.rank()];
        let str = format!("{}{}", file_char, rank_char);
        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod test_masks {
    use crate::bitboard::{FILE_B, FILE_G};

    use super::*;

    #[test]
    fn files_adjacent_mask_1() {
        assert_eq!(A1.files_adjacent_mask(), FILE_B)
    }

    #[test]
    fn files_adjacent_mask_2() {
        assert_eq!(H1.files_adjacent_mask(), FILE_G)
    }

    #[test]
    fn files_adjacent_mask_3() {
        assert_eq!(C1.files_adjacent_mask(), FILE_B | FILE_B << 2)
    }
}

#[cfg(test)]
mod test_comparison {
    use super::*;

    #[test]
    fn less_than_1() {
        assert!(A1 < B2)
    }

    #[test]
    fn less_than_2() {
        assert!(!(A1 > B2))
    }

    #[test]
    fn greater_than() {
        assert!(B2 > A1)
    }
}
#[cfg(test)]
mod test_display {
    use super::*;

    #[test]
    fn a1() {
        let fmt_str = A1.to_string();
        let expected = "a1";

        assert_eq!(fmt_str, expected);
    }

    #[test]
    fn h8() {
        let fmt_str = H8.to_string();
        let expected = "h8";

        assert_eq!(fmt_str, expected);
    }

    #[test]
    fn c5() {
        let fmt_str = C5.to_string();
        let expected = "c5";

        assert_eq!(fmt_str, expected);
    }
}

pub const A1: Square = Square(0);
pub const B1: Square = Square(1);
pub const C1: Square = Square(2);
pub const D1: Square = Square(3);
pub const E1: Square = Square(4);
pub const F1: Square = Square(5);
pub const G1: Square = Square(6);
pub const H1: Square = Square(7);
pub const A2: Square = Square(8);
pub const B2: Square = Square(9);
pub const C2: Square = Square(10);
pub const D2: Square = Square(11);
pub const E2: Square = Square(12);
pub const F2: Square = Square(13);
pub const G2: Square = Square(14);
pub const H2: Square = Square(15);
pub const A3: Square = Square(16);
pub const B3: Square = Square(17);
pub const C3: Square = Square(18);
pub const D3: Square = Square(19);
pub const E3: Square = Square(20);
pub const F3: Square = Square(21);
pub const G3: Square = Square(22);
pub const H3: Square = Square(23);
pub const A4: Square = Square(24);
pub const B4: Square = Square(25);
pub const C4: Square = Square(26);
pub const D4: Square = Square(27);
pub const E4: Square = Square(28);
pub const F4: Square = Square(29);
pub const G4: Square = Square(30);
pub const H4: Square = Square(31);
pub const A5: Square = Square(32);
pub const B5: Square = Square(33);
pub const C5: Square = Square(34);
pub const D5: Square = Square(35);
pub const E5: Square = Square(36);
pub const F5: Square = Square(37);
pub const G5: Square = Square(38);
pub const H5: Square = Square(39);
pub const A6: Square = Square(40);
pub const B6: Square = Square(41);
pub const C6: Square = Square(42);
pub const D6: Square = Square(43);
pub const E6: Square = Square(44);
pub const F6: Square = Square(45);
pub const G6: Square = Square(46);
pub const H6: Square = Square(47);
pub const A7: Square = Square(48);
pub const B7: Square = Square(49);
pub const C7: Square = Square(50);
pub const D7: Square = Square(51);
pub const E7: Square = Square(52);
pub const F7: Square = Square(53);
pub const G7: Square = Square(54);
pub const H7: Square = Square(55);
pub const A8: Square = Square(56);
pub const B8: Square = Square(57);
pub const C8: Square = Square(58);
pub const D8: Square = Square(59);
pub const E8: Square = Square(60);
pub const F8: Square = Square(61);
pub const G8: Square = Square(62);
pub const H8: Square = Square(63);
pub const NULL: Square = Square(64);

pub const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

pub static ALL: [Square; 64] = [
    A1, B1, C1, D1, E1, F1, G1, H1, A2, B2, C2, D2, E2, F2, G2, H2, A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4, A5, B5, C5, D5, E5, F5, G5, H5, A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7, A8, B8, C8, D8, E8, F8, G8, H8,
];

pub static DIAGONALS: [BB; 64] = [
    BB(0),
    BB(256),
    BB(66048),
    BB(16909312),
    BB(4328785920),
    BB(1108169199616),
    BB(283691315109888),
    BB(72624976668147712),
    BB(2),
    BB(65540),
    BB(16908296),
    BB(4328783888),
    BB(1108169195552),
    BB(283691315101760),
    BB(72624976668131456),
    BB(145249953336262656),
    BB(516),
    BB(16778248),
    BB(4328523792),
    BB(1108168675360),
    BB(283691314061376),
    BB(72624976666050688),
    BB(145249953332101120),
    BB(290499906664136704),
    BB(132104),
    BB(4295231504),
    BB(1108102090784),
    BB(283691180892224),
    BB(72624976399712384),
    BB(145249952799424512),
    BB(290499905598783488),
    BB(580999811180789760),
    BB(33818640),
    BB(1099579265056),
    BB(283674135240768),
    BB(72624942308409472),
    BB(145249884616818688),
    BB(290499769233571840),
    BB(580999538450366464),
    BB(1161999072605765632),
    BB(8657571872),
    BB(281492291854400),
    BB(72620578621636736),
    BB(145241157243273216),
    BB(290482314486480896),
    BB(580964628956184576),
    BB(1161929253617401856),
    BB(2323857407723175936),
    BB(2216338399296),
    BB(72062026714726528),
    BB(144124053429452800),
    BB(288248106858840064),
    BB(576496213700902912),
    BB(1152992423106838528),
    BB(2305983746702049280),
    BB(4611686018427387904),
    BB(567382630219904),
    BB(1134765260439552),
    BB(2269530520813568),
    BB(4539061024849920),
    BB(9078117754732544),
    BB(18155135997837312),
    BB(36028797018963968),
    BB(0),
];

pub static ANTI_DIAGONALS: [BB; 64] = [
    BB(9241421688590303744),
    BB(36099303471055872),
    BB(141012904183808),
    BB(550831656960),
    BB(2151686144),
    BB(8404992),
    BB(32768),
    BB(0),
    BB(4620710844295151616),
    BB(9241421688590303233),
    BB(36099303471054850),
    BB(141012904181764),
    BB(550831652872),
    BB(2151677968),
    BB(8388640),
    BB(64),
    BB(2310355422147510272),
    BB(4620710844295020800),
    BB(9241421688590041601),
    BB(36099303470531586),
    BB(141012903135236),
    BB(550829559816),
    BB(2147491856),
    BB(16416),
    BB(1155177711056977920),
    BB(2310355422114021376),
    BB(4620710844228043008),
    BB(9241421688456086017),
    BB(36099303202620418),
    BB(141012367312900),
    BB(549757915144),
    BB(4202512),
    BB(577588851233521664),
    BB(1155177702483820544),
    BB(2310355404967706624),
    BB(4620710809935413504),
    BB(9241421619870827009),
    BB(36099166032102402),
    BB(140738026276868),
    BB(1075843080),
    BB(288793326105133056),
    BB(577586656505233408),
    BB(1155173313027244032),
    BB(2310346626054553600),
    BB(4620693252109107456),
    BB(9241386504218214913),
    BB(36028934726878210),
    BB(275415828484),
    BB(144115188075855872),
    BB(288231475663339520),
    BB(576462955621646336),
    BB(1152925911260069888),
    BB(2305851822520205312),
    BB(4611703645040410880),
    BB(9223407290080821761),
    BB(70506452091906),
    BB(0),
    BB(281474976710656),
    BB(564049465049088),
    BB(1128103225065472),
    BB(2256206466908160),
    BB(4512412933881856),
    BB(9024825867763968),
    BB(18049651735527937),
];
