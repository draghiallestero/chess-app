use std::{array::from_fn, slice::Iter};

use super::*;

pub struct Moves {
    pub list: [Move; 128],
    pub count: u8,
}

impl Default for Moves {
    fn default() -> Self {
        Moves {
            list: from_fn(|_| Move::default()),
            count: 0,
        }
    }
}

impl Moves {
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn iter(&self) -> Iter<'_, Move> {
        self.list[0..self.count as usize].iter()
    }
}

pub struct MovesVec {
    pub vec: Vec<Box<Moves>>,
}

impl Default for MovesVec {
    fn default() -> Self {
        MovesVec {
            vec: Vec::default(),
        }
    }
}

impl MovesVec {
    pub fn push_moves(&mut self, mut moves: Box<Moves>) {
        moves.count = 0;
        self.vec.push(moves);
    }
    pub fn pop_moves(&mut self) -> Box<Moves> {
        self.vec.pop().unwrap_or(Box::new(Moves::default()))
    }
}
