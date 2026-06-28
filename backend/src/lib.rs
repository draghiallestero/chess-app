mod bitboard;

use std::{iter::StepBy, ops::Range, sync::LazyLock};

use arrayvec::ArrayVec;
use bitboard::BitBoard;

#[derive(Default)]
pub struct Player {
    pawns: BitBoard,
    rooks: BitBoard,
    knights: BitBoard,
    bishops: BitBoard,
    queens: BitBoard,
    kings: BitBoard,
}

impl Player {
    pub fn occupied_squares(&self) -> BitBoard {
        self.pawns | self.rooks | self.knights | self.bishops | self.queens | self.kings
    }
}

// impl Default for Player {
//     fn default() -> Self {
//         Player {
//             pawns: 0,
//             rooks: 0,
//             knights: 0,
//             bishops: 0,
//             queens: 0,
//             kings: 0,
//         }
//     }
// }

#[derive(Default)]
pub struct Board {
    player: Player,
    opponent: Player,
}

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
    let to_pos = |rank, file| 8 * rank + file;

    let mut target_pos_lists_2d =
        std::array::from_fn(|_| std::array::from_fn(|_| ArrayVec::<u8, 7>::new()));
    for pos in 0u8..64u8 {
        let current_target_pos_lists = &mut target_pos_lists_2d[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        // South-west
        let mut current_rank = rank;
        let mut current_file = file;
        let mut current_direction = 0;
        while current_rank > 0 && current_file > 0 {
            current_target_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            current_rank -= 1;
            current_file -= 1;
        }
        // South-east
        current_rank = rank;
        current_file = file;
        current_direction = 1;
        while current_rank > 0 && current_file < 7 {
            current_target_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            current_rank -= 1;
            current_file += 1;
        }
        // North-east
        current_rank = rank;
        current_file = file;
        current_direction = 2;
        while current_rank < 7 && current_file < 7 {
            current_target_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            current_rank += 1;
            current_file += 1;
        }
        // North-west
        current_rank = rank;
        current_file = file;
        current_direction = 3;
        while current_rank < 7 && current_file > 0 {
            current_target_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            current_rank += 1;
            current_file -= 1;
        }
    }
    target_pos_lists_2d
});

// impl Default for Board {
//     fn default() -> Self {
//         Board {
//             white: Player::default(),
//             black: Player::default(),
//         }
//     }
// }

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
        for pos in 8u8..56u8 {
            // No more left
            if pawn_count == 0 {
                break;
            }

            let pawn_at_pos = self.player.pawns.get(pos);
            let target_pos = pos + 8;
            if pawn_at_pos {
                // Pawns can move one square forwards
                check_and_push_pawn_move(pos, target_pos);
                // Pawns on their initial rank can optionally move two squares
                if pos < 16 {
                    let target_pos = pos + 16;
                    if pawn_at_pos {
                        check_and_push_pawn_move(pos, target_pos);
                        // No more left
                        pawn_count -= 1;
                        if pawn_count == 0 {
                            break;
                        }
                    }
                }
                pawn_count -= 1;
            }
        }

        // Rooks
        let mut rook_count = self.player.rooks.count();
        let mut check_and_push_rook_moves =
            |pos, target_pos_range: StepBy<Range<u8>>, reverse_range| {
                let mut loop_body = |pos, target_pos| {
                    let target_pos_free = !occupied_squares.get(target_pos);
                    if target_pos_free {
                        let new_move = Move {
                            from: pos,
                            to: target_pos,
                        };
                        moves.push(new_move);
                        return true;
                    }
                    return false;
                };
                if reverse_range {
                    for target_pos in target_pos_range.rev() {
                        loop_body(pos, target_pos);
                    }
                } else {
                    for target_pos in target_pos_range {
                        loop_body(pos, target_pos);
                    }
                }
            };
        for pos in 0u8..64u8 {
            // No more left
            if rook_count == 0 {
                break;
            }

            let rook_at_pos = self.player.rooks.get(pos);
            if rook_at_pos {
                // Rooks can slide across the board in the cardinal directions
                let (rank, file) = (pos / 8, pos % 8);
                // Left
                if file != 0 {
                    check_and_push_rook_moves(pos, (8 * rank..8 * rank + file).step_by(1), true);
                }
                // Right
                if file != 7 {
                    check_and_push_rook_moves(
                        pos,
                        (8 * rank + file + 1..8 * (rank + 1)).step_by(1),
                        false,
                    );
                }
                // Up
                if rank != 7 {
                    check_and_push_rook_moves(pos, (8 * (rank + 1) + file..0).step_by(8), true);
                }
                // Down
                if rank != 0 {
                    check_and_push_rook_moves(pos, (file..8 * rank + file).step_by(8), true);
                }

                rook_count -= 1;
            }
        }

        // Knights
        let mut knight_count = self.player.knights.count();
        for pos in 0u8..64u8 {
            // No more left
            if knight_count == 0 {
                break;
            }

            let rook_at_pos = self.player.knights.get(pos);
            if rook_at_pos {
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

                knight_count -= 1;
            }
        }

        // Bishops
        let mut bishop_count = self.player.bishops.count();
        for pos in 0u8..64u8 {
            // No more left
            if bishop_count == 0 {
                break;
            }

            let rook_at_pos = self.player.bishops.get(pos);
            if rook_at_pos {
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

                bishop_count -= 1;
            }
        }

        moves
    }

    // pub fn opposing_view(&self) -> Board
    // {

    // }
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
    fn perf() {}
}
