use crate::board::Board;

#[derive(Default, Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub board: Board,
    pub evaluation: i16,
}
