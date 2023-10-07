const REGULAR_ISIZE: isize = 0;
const ESCAPE_ISIZE: isize = 1;
const LOUD_ISIZE: isize = 2;

pub enum MoveType {
    Regular = REGULAR_ISIZE,
    Escape = ESCAPE_ISIZE,
    Loud = LOUD_ISIZE,
}
