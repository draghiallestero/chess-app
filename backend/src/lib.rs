pub mod bitboard;
pub use bitboard::BitBoard;
use std::{iter::StepBy, ops::Range, sync::LazyLock};

use arrayvec::ArrayVec;
use bit_iter::BitIter;

#[derive(Default)]
pub struct Player {
    pub pawns: BitBoard,
    pub rooks: BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard,
    pub queens: BitBoard,
    pub kings: BitBoard,
}

impl Player {
    pub fn occupied_squares(&self) -> BitBoard {
        self.pawns | self.rooks | self.knights | self.bishops | self.queens | self.kings
    }

    pub fn flip_view(&self) -> Player {
        Player {
            pawns: self.pawns.reverse_bits(),
            rooks: self.rooks.reverse_bits(),
            knights: self.knights.reverse_bits(),
            bishops: self.bishops.reverse_bits(),
            queens: self.queens.reverse_bits(),
            kings: self.kings.reverse_bits(),
        }
    }
}

pub struct Board {
    pub player: Player,
    pub opponent: Player,
}

fn generate_sliding_piece_target_pos_lists_2d<const DIRECTIONS: usize>(
    target_pos_lists_2d: &mut [[ArrayVec<u8, 7>; DIRECTIONS]; 64],
    cardinals: bool,
    diagonals: bool,
) {
    // Utility
    let to_pos = |rank, file| 8 * rank + file;

    // Process each direction
    for pos in 0u8..64u8 {
        let current_target_pos_lists = &mut target_pos_lists_2d[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        let mut current_rank;
        let mut current_file;
        let mut current_direction = 0;

        if cardinals {
            // West
            current_rank = rank;
            current_file = file;
            current_direction = 0;
            while current_file > 0 {
                current_file -= 1;
                current_target_pos_lists[current_direction]
                    .push(to_pos(current_rank, current_file));
            }
            // East
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_file < 7 {
                current_file += 1;
                current_target_pos_lists[current_direction]
                    .push(to_pos(current_rank, current_file));
            }
            // North
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank < 7 {
                current_rank += 1;
                current_target_pos_lists[current_direction]
                    .push(to_pos(current_rank, current_file));
            }
            // South
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank > 0 {
                current_rank -= 1;
                current_target_pos_lists[current_direction]
                    .push(to_pos(current_rank, current_file));
            }
        }
        if diagonals {
            // South-west
            current_rank = rank;
            current_file = file;
            current_direction += if current_direction == 0 { 0 } else { 1 };
            while current_rank > 0 && current_file > 0 {
                current_rank -= 1;
                current_file -= 1;
                current_target_pos_lists[current_direction]
                    .push(to_pos(current_rank, current_file));
            }
            // South-east
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank > 0 && current_file < 7 {
                current_rank -= 1;
                current_file += 1;
                current_target_pos_lists[current_direction]
                    .push(to_pos(current_rank, current_file));
            }
            // North-east
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank < 7 && current_file < 7 {
                current_rank += 1;
                current_file += 1;
                current_target_pos_lists[current_direction]
                    .push(to_pos(current_rank, current_file));
            }
            // North-west
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank < 7 && current_file > 0 {
                current_rank += 1;
                current_file -= 1;
                current_target_pos_lists[current_direction]
                    .push(to_pos(current_rank, current_file));
            }
        }
    }
}

// Rooks can slide along the cardinal directions
static ROOK_TARGET_POS_LISTS_2D: LazyLock<[[ArrayVec<u8, 7>; 4]; 64]> = LazyLock::new(|| {
    let mut target_pos_lists_2d: [[ArrayVec<u8, 7>; 4]; 64] =
        std::array::from_fn(|_| std::array::from_fn(|_| ArrayVec::<u8, 7>::new()));
    generate_sliding_piece_target_pos_lists_2d(&mut target_pos_lists_2d, true, false);
    target_pos_lists_2d
});

// Knights can move over two squares then one square, either first horizontally or first vertically
static KNIGHT_TARGET_POS_LISTS: LazyLock<[ArrayVec<u8, 8>; 64]> = LazyLock::new(|| {
    let to_pos = |rank, file| 8 * rank + file;

    let mut target_pos_lists = std::array::from_fn(|_| ArrayVec::<u8, 8>::new());
    for pos in 0u8..64u8 {
        let current_target_pos_list = &mut target_pos_lists[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        // Constraints
        let ranks_from_top = 7 - rank;
        let ranks_from_bottom = rank;
        let files_from_left = file;
        let files_from_right = 7 - file;

        // West-south-west corner
        if ranks_from_bottom >= 1 && files_from_left >= 2 {
            current_target_pos_list.push(to_pos(rank - 1, file - 2));
        }
        // South-south-west corner
        if ranks_from_bottom >= 2 && files_from_left >= 1 {
            current_target_pos_list.push(to_pos(rank - 2, file - 1));
        }
        // South-south-east corner
        if ranks_from_bottom >= 2 && files_from_right >= 1 {
            current_target_pos_list.push(to_pos(rank - 2, file + 1));
        }
        // East-south-east corner
        if ranks_from_bottom >= 1 && files_from_right >= 2 {
            current_target_pos_list.push(to_pos(rank - 1, file + 2));
        }
        // East-north-east corner
        if ranks_from_top >= 1 && files_from_right >= 2 {
            current_target_pos_list.push(to_pos(rank + 1, file + 2));
        }
        // North-north-east corner
        if ranks_from_top >= 2 && files_from_right >= 1 {
            current_target_pos_list.push(to_pos(rank + 2, file + 1));
        }
        // North-north-west corner
        if ranks_from_top >= 2 && files_from_left >= 1 {
            current_target_pos_list.push(to_pos(rank + 2, file - 1));
        }
        // West-north-west corner
        if ranks_from_top >= 1 && files_from_left >= 2 {
            current_target_pos_list.push(to_pos(rank + 1, file - 2));
        }
    }
    target_pos_lists
});

// Bishops can slide along the diagonal directions
static BISHOP_TARGET_POS_LISTS_2D: LazyLock<[[ArrayVec<u8, 7>; 4]; 64]> = LazyLock::new(|| {
    let mut target_pos_lists_2d: [[ArrayVec<u8, 7>; 4]; 64] =
        std::array::from_fn(|_| std::array::from_fn(|_| ArrayVec::<u8, 7>::new()));
    generate_sliding_piece_target_pos_lists_2d(&mut target_pos_lists_2d, false, true);
    target_pos_lists_2d
});

// Queens can slide along the cardinal and diagonal directions
static QUEEN_TARGET_POS_LISTS_2D: LazyLock<[[ArrayVec<u8, 7>; 8]; 64]> = LazyLock::new(|| {
    let mut target_pos_lists_2d: [[ArrayVec<u8, 7>; 8]; 64] =
        std::array::from_fn(|_| std::array::from_fn(|_| ArrayVec::<u8, 7>::new()));
    generate_sliding_piece_target_pos_lists_2d(&mut target_pos_lists_2d, true, true);
    target_pos_lists_2d
});

// Kings can move one square in any direction
static KING_TARGET_POS_LISTS: LazyLock<[ArrayVec<u8, 8>; 64]> = LazyLock::new(|| {
    let to_pos = |rank, file| 8 * rank + file;

    let mut target_pos_lists = std::array::from_fn(|_| ArrayVec::<u8, 8>::new());
    for pos in 0u8..64u8 {
        let current_target_pos_list = &mut target_pos_lists[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        // Constraints
        let ranks_from_top = 7 - rank;
        let ranks_from_bottom = rank;
        let files_from_left = file;
        let files_from_right = 7 - file;

        // West
        if files_from_left >= 1 {
            current_target_pos_list.push(to_pos(rank, file - 1));
        }
        // South-west
        if ranks_from_bottom >= 1 && files_from_left >= 1 {
            current_target_pos_list.push(to_pos(rank - 1, file - 1));
        }
        // South
        if ranks_from_bottom >= 1 {
            current_target_pos_list.push(to_pos(rank - 1, file));
        }
        // South-east
        if ranks_from_bottom >= 1 && files_from_right >= 1 {
            current_target_pos_list.push(to_pos(rank - 1, file + 1));
        }
        // East
        if files_from_right >= 1 {
            current_target_pos_list.push(to_pos(rank, file + 1));
        }
        // North-east
        if ranks_from_top >= 1 && files_from_right >= 1 {
            current_target_pos_list.push(to_pos(rank + 1, file + 1));
        }
        // North
        if ranks_from_top >= 1 {
            current_target_pos_list.push(to_pos(rank + 1, file));
        }
        // North-West
        if ranks_from_top >= 1 && files_from_left >= 1 {
            current_target_pos_list.push(to_pos(rank + 1, file - 1));
        }
    }
    target_pos_lists
});

impl Board {
    pub fn generate_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        // Occupied squares
        let player_occupied_squares = self.player.occupied_squares();
        let opponent_occupied_squares = self.opponent.occupied_squares();
        let occupied_squares = player_occupied_squares | opponent_occupied_squares;

        // Pawns
        let mut pawn_count = self.player.pawns.count();
        let mut check_and_push_pawn_move = |pos, target_pos| {
            let target_pos_free = !occupied_squares.get(target_pos);
            if target_pos_free {
                let new_move = Move {
                    from: pos,
                    to: target_pos,
                };
                moves.push(new_move);
            }
        };
        for pos in BitIter::from(self.player.pawns.board).map(|x| x as u8) {
            let target_pos = pos + 8;
            // Pawns can move one square forwards
            check_and_push_pawn_move(pos, target_pos);
            // Pawns on their initial rank can optionally move two squares
            if pos < 16 {
                let target_pos = pos + 16;
                check_and_push_pawn_move(pos, target_pos);
                // No more left
                pawn_count -= 1;
                if pawn_count == 0 {
                    break;
                }
            }
        }

        // Rooks
        for pos in BitIter::from(self.player.rooks.board).map(|x| x as u8) {
            for direction in 0..4 {
                let target_pos_list = &ROOK_TARGET_POS_LISTS_2D[pos as usize];
                for target_pos in &target_pos_list[direction] {
                    let target_pos_free = !occupied_squares.get(*target_pos);
                    if target_pos_free {
                        let new_move = Move {
                            from: pos,
                            to: *target_pos,
                        };
                        moves.push(new_move);
                    } else {
                        break;
                    }
                }
            }
        }

        // Knights
        for pos in BitIter::from(self.player.knights.board).map(|x| x as u8) {
            let target_pos_list = &KNIGHT_TARGET_POS_LISTS[pos as usize];
            for target_pos in target_pos_list {
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    let new_move = Move {
                        from: pos,
                        to: *target_pos,
                    };
                    moves.push(new_move);
                }
            }
        }

        // Bishops
        for pos in BitIter::from(self.player.bishops.board).map(|x| x as u8) {
            for direction in 0..4 {
                let target_pos_list = &BISHOP_TARGET_POS_LISTS_2D[pos as usize];
                for target_pos in &target_pos_list[direction] {
                    let target_pos_free = !occupied_squares.get(*target_pos);
                    if target_pos_free {
                        let new_move = Move {
                            from: pos,
                            to: *target_pos,
                        };
                        moves.push(new_move);
                    } else {
                        break;
                    }
                }
            }
        }

        // Queens
        for pos in BitIter::from(self.player.queens.board).map(|x| x as u8) {
            for direction in 0..4 {
                let target_pos_list = &QUEEN_TARGET_POS_LISTS_2D[pos as usize];
                for target_pos in &target_pos_list[direction] {
                    let target_pos_free = !occupied_squares.get(*target_pos);
                    if target_pos_free {
                        let new_move = Move {
                            from: pos,
                            to: *target_pos,
                        };
                        moves.push(new_move);
                    } else {
                        break;
                    }
                }
            }
        }

        // Kings
        for pos in BitIter::from(self.player.kings.board).map(|x| x as u8) {
            let target_pos_list = &KING_TARGET_POS_LISTS[pos as usize];
            for target_pos in target_pos_list {
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    let new_move = Move {
                        from: pos,
                        to: *target_pos,
                    };
                    moves.push(new_move);
                }
            }
        }

        moves
    }
}

impl Default for Board {
    fn default() -> Board {
        let player = Player {
            pawns: BitBoard::new(
                0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_1111_0000_0000,
            ),
            rooks: BitBoard::new(
                0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1000_0001,
            ),
            knights: BitBoard::new(
                0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0010,
            ),
            bishops: BitBoard::new(
                0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0100,
            ),
            queens: BitBoard::new(
                0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1000,
            ),
            kings: BitBoard::new(
                0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000,
            ),
        };
        let opponent = Player {
            pawns: BitBoard::new(
                0b0000_0000_1111_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
            ),
            rooks: BitBoard::new(
                0b1000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
            ),
            knights: BitBoard::new(
                0b0100_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
            ),
            bishops: BitBoard::new(
                0b0010_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
            ),
            queens: BitBoard::new(
                0b0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
            ),
            kings: BitBoard::new(
                0b0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000,
            ),
        };
        Board {
            player: player,
            opponent: opponent,
        }
    }
}

pub struct Square {
    rank: i32,
    file: i32,
}

pub struct Move {
    from: u8,
    to: u8,
}

#[cfg(test)]
mod tests {
    fn perft() {}
}
