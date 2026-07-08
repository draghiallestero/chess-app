use std::{sync::LazyLock, thread::current};

use arrayvec::ArrayVec;

pub fn from_pos(pos: u8) -> (u8, u8) {
    (pos / 8, pos % 8)
}
pub fn to_pos(rank: u8, file: u8) -> u8 {
    8 * rank + file
}
pub fn to_chars(pos: u8) -> &'static str {
    let chars_array = [
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2",
        "h2", "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4",
        "g4", "h4", "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6",
        "f6", "g6", "h6", "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8",
        "e8", "f8", "g8", "h8",
    ];
    chars_array[pos as usize]
}
pub fn from_chars(chars: &str) -> u8 {
    let chars_array = [
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2",
        "h2", "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4",
        "g4", "h4", "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6",
        "f6", "g6", "h6", "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8",
        "e8", "f8", "g8", "h8",
    ];
    chars_array.iter().position(move |x| *x == chars).unwrap() as u8
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
pub static ROOK_TARGET_POS_LISTS_2D: LazyLock<[[ArrayVec<u8, 7>; 4]; 64]> = LazyLock::new(|| {
    let mut target_pos_lists_2d: [[ArrayVec<u8, 7>; 4]; 64] =
        std::array::from_fn(|_| std::array::from_fn(|_| ArrayVec::<u8, 7>::new()));
    generate_sliding_piece_target_pos_lists_2d(&mut target_pos_lists_2d, true, false);
    target_pos_lists_2d
});

// Knights can move over two squares then one square, either first horizontally or first vertically
pub static KNIGHT_TARGET_POS_LISTS_2D: LazyLock<[ArrayVec<u8, 8>; 64]> = LazyLock::new(|| {
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
pub static BISHOP_TARGET_POS_LISTS_2D: LazyLock<[[ArrayVec<u8, 7>; 4]; 64]> = LazyLock::new(|| {
    let mut target_pos_lists_2d: [[ArrayVec<u8, 7>; 4]; 64] =
        std::array::from_fn(|_| std::array::from_fn(|_| ArrayVec::<u8, 7>::new()));
    generate_sliding_piece_target_pos_lists_2d(&mut target_pos_lists_2d, false, true);
    target_pos_lists_2d
});

// Queens can slide along the cardinal and diagonal directions
pub static QUEEN_TARGET_POS_LISTS_2D: LazyLock<[[ArrayVec<u8, 7>; 8]; 64]> = LazyLock::new(|| {
    let mut target_pos_lists_2d: [[ArrayVec<u8, 7>; 8]; 64] =
        std::array::from_fn(|_| std::array::from_fn(|_| ArrayVec::<u8, 7>::new()));
    generate_sliding_piece_target_pos_lists_2d(&mut target_pos_lists_2d, true, true);
    target_pos_lists_2d
});

// Kings can move one square in any direction
pub static KING_TARGET_POS_LISTS_2D: LazyLock<[ArrayVec<u8, 8>; 64]> = LazyLock::new(|| {
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
