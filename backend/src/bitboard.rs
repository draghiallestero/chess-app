use std::ops::BitAnd;
use std::ops::BitOr;

use bit_iter::BitIter;

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
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
    pub fn unset(&mut self, pos: u8) {
        self.board &= !(1u64 << pos);
    }
    pub fn count(&self) -> u8 {
        self.board.count_ones() as u8
    }
    pub fn reverse_bits(&self) -> BitBoard {
        BitBoard {
            board: self.board.reverse_bits(),
        }
    }
    pub fn iter(&self) -> bit_iter::BitIter<u64> {
        BitIter::from(self.board)
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
