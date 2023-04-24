#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub x: isize,
    pub y: isize
}

#[derive(PartialEq)]
pub enum Move {
    Left,
    Right
}