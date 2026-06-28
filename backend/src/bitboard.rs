use std::ops::BitAnd;
use std::ops::BitOr;

#[derive(Default, Copy, Clone)]
pub struct BitBoard {
    pub board: u64,
}

impl BitBoard {
    pub fn new(board: u64) -> BitBoard {
        BitBoard { board: board }
    }
    pub fn get(&self, pos: u8) -> bool {
        ((self.board >> pos) & 1u64) == 1u64
    }
    pub fn set(&mut self, pos: u8) {
        self.board |= 1u64 << pos;
    }
    pub fn count(&self) -> u8 {
        self.board.count_ones() as u8
    }
    pub fn reverse_bits(&self) -> BitBoard {
        BitBoard {
            board: self.board.reverse_bits(),
        }
    }
}

impl BitAnd for BitBoard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard {
            board: self.board & rhs.board,
        }
    }
}

impl BitOr for BitBoard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard {
            board: self.board | rhs.board,
        }
    }
}
