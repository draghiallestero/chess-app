use crate::{BitBoard, en_passant::EnPassant};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::{array::from_fn, collections::HashMap, sync::LazyLock};

use arrayvec::ArrayVec;
use itertools::Itertools;
use rand::RngExt;

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

#[derive(Default)]
pub struct MagicBitboard {
    blocker_bitboard: BitBoard,
    magic_number: u64,
    // TODO: store blocker_bitboard_bits?
    attack_bitboards: Vec<BitBoard>,
}

impl MagicBitboard {
    pub fn get_attack_bitboard(&self, occupied_squares: BitBoard) -> &BitBoard {
        let index = (self.blocker_bitboard & occupied_squares)
            .board
            .wrapping_mul(self.magic_number)
            >> (64 - self.blocker_bitboard.count());
        &self.attack_bitboards[index as usize]
    }
}

fn generate_sliding_piece_magic_bitboards(cardinals: bool, diagonals: bool) -> [MagicBitboard; 64] {
    // Utility
    let to_pos = |rank, file| 8 * rank + file;

    // Rank/file deltas
    let deltas: Vec<(i8, i8)> = {
        let mut deltas = Vec::default();
        if cardinals {
            let mut new_deltas = vec![(0, 1), (0, -1), (1, 0), (-1, 0)];
            deltas.append(&mut new_deltas);
        }
        if diagonals {
            let mut new_deltas = vec![(1, 1), (1, -1), (-1, 1), (-1, 1)];
            deltas.append(&mut new_deltas);
        }
        deltas
    };
    let limits: Vec<(u8, u8)> = {
        let mut limits = Vec::default();
        if cardinals {
            let mut new_limits = vec![(u8::MAX, 7), (u8::MAX, 0), (7, u8::MAX), (0, u8::MAX)];
            limits.append(&mut new_limits);
        }
        if diagonals {
            let mut new_limits = vec![(7, 7), (7, 0), (0, 7), (0, 0)];
            limits.append(&mut new_limits);
        }
        limits
    };

    // RNG
    let mut rng = rand::rngs::StdRng::from_seed([10; 32]);
    // let mut rng = rand::rng();

    let mut mgs: [MagicBitboard; 64] = from_fn(|_| MagicBitboard::default());

    for pos in 0u8..64u8 {
        let (rank, file) = from_pos(pos);

        let add_delta = |x, delta: &i8| ((x as i8).wrapping_add(*delta)) as u8;

        let mut mg: MagicBitboard = MagicBitboard::default();

        // Find blocker bitboards
        mg.blocker_bitboard = {
            let mut blocker_bitboard = BitBoard::default();
            for ((rank_delta, file_delta), (rank_limit, file_limit)) in
                deltas.iter().zip(limits.iter())
            {
                let mut current_rank = add_delta(rank, rank_delta);
                let mut current_file = add_delta(file, file_delta);

                // Exclude board edges
                while current_rank <= 7 && current_file <= 7 {
                    if current_rank != *rank_limit && current_file != *file_limit {
                        blocker_bitboard.set(to_pos(current_rank, current_file));
                    }
                    current_rank = add_delta(current_rank, rank_delta);
                    current_file = add_delta(current_file, file_delta);
                }
            }
            blocker_bitboard
        };

        // Find blocker combination bitboards
        let blocker_combination_bitboards = {
            let mut blocker_combination_bitboards = Vec::default();
            for active_blocker_positions in mg
                .blocker_bitboard
                .iter()
                .map(|x| vec![None, Some(x)])
                .multi_cartesian_product()
            {
                let mut blocker_combination_bitboard = BitBoard::default();
                for pos in active_blocker_positions {
                    match pos {
                        Some(pos) => blocker_combination_bitboard.set(pos as u8),
                        None => (),
                    }
                }
                blocker_combination_bitboards.push(blocker_combination_bitboard);
            }
            blocker_combination_bitboards
        };

        // Find attack bitboards
        let attack_bitboards = {
            let mut attack_bitboards = Vec::default();
            for blocker_combination_bitboard in blocker_combination_bitboards.iter() {
                let mut attack_bitboard = BitBoard::default();
                for (rank_delta, file_delta) in &deltas {
                    let mut current_rank = add_delta(rank, rank_delta);
                    let mut current_file = add_delta(file, file_delta);

                    while current_rank <= 7 && current_file <= 7 {
                        let current_pos = to_pos(current_rank, current_file);
                        attack_bitboard.set(current_pos);
                        if blocker_combination_bitboard.get(current_pos) {
                            break;
                        }
                        current_rank = add_delta(current_rank, rank_delta);
                        current_file = add_delta(current_file, file_delta);
                    }
                }
                attack_bitboards.push(attack_bitboard);
            }
            attack_bitboards
        };

        // Find magic number
        'magic_number: loop {
            // Use this code to find a new random number if you need to
            let generate_random_numbers = false;
            if generate_random_numbers {
                let mut new_random = || -> u64 { rng.random() };
                mg.magic_number = new_random();
                mg.magic_number &= new_random();

                // Need a small number of bits, otherwise conflicts are more likely
                // if mg.magic_number.count_ones() < 4 || mg.magic_number.count_ones() > 8 {
                if mg.magic_number.count_ones() > 8 {
                    continue;
                }
            } else {
                // But we use these values found by running said code
                mg.magic_number = [
                    36038834634375168,
                    9241404032919298048,
                    2341889399645536384,
                    1188959097789087873,
                    1188954699714271232,
                    144117387904681985,
                    4647717014486385024,
                    3494793449368978048,
                    1153625194365992960,
                    9359078411210752,
                    865957903297810432,
                    985265565859968,
                    4900057424125755520,
                    577023736893670912,
                    1125909051740168,
                    2392541597552768,
                    36359200267305088,
                    45106640164233226,
                    3527783326162944,
                    578858237542027264,
                    9289773877302320,
                    9228016923793425921,
                    74766811809808,
                    144121785284296833,
                    27162345992110088,
                    9242547520716951556,
                    2306407342147665952,
                    712642449637632,
                    10377419544581914752,
                    10394870898515510280,
                    2308097025231552792,
                    558347863045,
                    351914596239392,
                    5764677960502091776,
                    310783562972413968,
                    870329426238246912,
                    4926258777424896,
                    1189231785880323072,
                    1155184299553652740,
                    1122093760652,
                    10376329345382776834,
                    40532695683448832,
                    81064935027966016,
                    432381032068677640,
                    288253466567049232,
                    23362431724093448,
                    8806025396241,
                    38281147679113220,
                    70372560994432,
                    9042934456456448,
                    297387177709404288,
                    9525113761913372800,
                    328806755410903168,
                    5909853009097031808,
                    562967284810240,
                    144115755028366848,
                    288863834435813393,
                    576497037262061698,
                    18034191870468106,
                    4503617092460617,
                    281545911304259,
                    562967268557058,
                    622804879364,
                    26526800217090,
                ][pos as usize];
            }

            // Verify all blocker combination bitboards result in either unique indexes or shared indexes into the same attack bitboard
            let mut index_to_attack_bitboard: HashMap<u64, BitBoard> = HashMap::default();
            for (blocker_combination_bitboard, attack_bitboard) in blocker_combination_bitboards
                .iter()
                .zip(attack_bitboards.iter())
            {
                let index = blocker_combination_bitboard
                    .board
                    .wrapping_mul(mg.magic_number)
                    >> (64 - mg.blocker_bitboard.count());
                match index_to_attack_bitboard.insert(index, *attack_bitboard) {
                    Some(present_attack_bitboard) => {
                        if present_attack_bitboard.board != attack_bitboard.board {
                            continue 'magic_number;
                        }
                    }
                    None => (),
                }
            }

            // Fill in attack bitboards
            mg.attack_bitboards.resize(
                *index_to_attack_bitboard.keys().max().unwrap() as usize + 1,
                BitBoard::default(),
            );
            for (index, attack_bitboard) in index_to_attack_bitboard {
                mg.attack_bitboards[index as usize] = attack_bitboard;
            }

            break;
        }

        mgs[pos as usize] = mg;
    }

    mgs
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
        let current_pos_lists = &mut target_pos_lists_2d[pos as usize];
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
                current_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            }
            // East
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_file < 7 {
                current_file += 1;
                current_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            }
            // North
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank < 7 {
                current_rank += 1;
                current_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            }
            // South
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank > 0 {
                current_rank -= 1;
                current_pos_lists[current_direction].push(to_pos(current_rank, current_file));
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
                current_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            }
            // South-east
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank > 0 && current_file < 7 {
                current_rank -= 1;
                current_file += 1;
                current_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            }
            // North-east
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank < 7 && current_file < 7 {
                current_rank += 1;
                current_file += 1;
                current_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            }
            // North-west
            current_rank = rank;
            current_file = file;
            current_direction += 1;
            while current_rank < 7 && current_file > 0 {
                current_rank += 1;
                current_file -= 1;
                current_pos_lists[current_direction].push(to_pos(current_rank, current_file));
            }
        }
    }
}

// Pawns can move forward one or two pieces
pub static PAWN_TARGET_POS_LISTS_2D: LazyLock<[ArrayVec<u8, 2>; 64]> = LazyLock::new(|| {
    let to_pos = |rank, file| 8 * rank + file;

    let mut target_pos_lists_2d = std::array::from_fn(|_| ArrayVec::<u8, 2>::new());
    for pos in 8u8..56u8 {
        let current_pos_list = &mut target_pos_lists_2d[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        current_pos_list.push(to_pos(rank + 1, file));
        if rank == 1 {
            current_pos_list.push(to_pos(rank + 2, file));
        }
    }
    target_pos_lists_2d
});

// Pawns can attack diagonally forward
pub static PAWN_ATTACK_POS_LISTS_2D: LazyLock<[ArrayVec<u8, 2>; 64]> = LazyLock::new(|| {
    let to_pos = |rank, file| 8 * rank + file;

    let mut target_pos_lists = std::array::from_fn(|_| ArrayVec::<u8, 2>::new());
    for pos in 8u8..56u8 {
        let current_pos_list = &mut target_pos_lists[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        if file > 0 {
            current_pos_list.push(to_pos(rank + 1, file - 1));
        }
        if file < 7 {
            current_pos_list.push(to_pos(rank + 1, file + 1));
        }
    }
    target_pos_lists
});

// En passant
pub static EN_PASSANT_LISTS: LazyLock<[EnPassant; 64]> = LazyLock::new(|| {
    let to_pos = |rank, file| 8 * rank + file;

    let mut en_passant_list = std::array::from_fn(|_| EnPassant::default());
    for pos in 8u8..16u8 {
        let current_en_passant = &mut en_passant_list[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        // Attacker positions are adjacent to the player target position
        current_en_passant.target_pos = 63 - to_pos(rank + 1, file);
        if file > 0 {
            current_en_passant.attacker_pos_list[0] = 63 - to_pos(rank + 2, file - 1);
        }
        if file < 7 {
            current_en_passant.attacker_pos_list[1] = 63 - to_pos(rank + 2, file + 1);
        }
    }
    en_passant_list
});

// Rooks can slide along the cardinal directions
pub static ROOK_MAGIC_BITBOARDS: LazyLock<[MagicBitboard; 64]> =
    LazyLock::new(|| generate_sliding_piece_magic_bitboards(true, false));

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
        let current_pos_list = &mut target_pos_lists[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        // Constraints
        let ranks_from_top = 7 - rank;
        let ranks_from_bottom = rank;
        let files_from_left = file;
        let files_from_right = 7 - file;

        // West-south-west corner
        if ranks_from_bottom >= 1 && files_from_left >= 2 {
            current_pos_list.push(to_pos(rank - 1, file - 2));
        }
        // South-south-west corner
        if ranks_from_bottom >= 2 && files_from_left >= 1 {
            current_pos_list.push(to_pos(rank - 2, file - 1));
        }
        // South-south-east corner
        if ranks_from_bottom >= 2 && files_from_right >= 1 {
            current_pos_list.push(to_pos(rank - 2, file + 1));
        }
        // East-south-east corner
        if ranks_from_bottom >= 1 && files_from_right >= 2 {
            current_pos_list.push(to_pos(rank - 1, file + 2));
        }
        // East-north-east corner
        if ranks_from_top >= 1 && files_from_right >= 2 {
            current_pos_list.push(to_pos(rank + 1, file + 2));
        }
        // North-north-east corner
        if ranks_from_top >= 2 && files_from_right >= 1 {
            current_pos_list.push(to_pos(rank + 2, file + 1));
        }
        // North-north-west corner
        if ranks_from_top >= 2 && files_from_left >= 1 {
            current_pos_list.push(to_pos(rank + 2, file - 1));
        }
        // West-north-west corner
        if ranks_from_top >= 1 && files_from_left >= 2 {
            current_pos_list.push(to_pos(rank + 1, file - 2));
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
        let current_pos_list = &mut target_pos_lists[pos as usize];
        let (rank, file) = (pos / 8, pos % 8);

        // Constraints
        let ranks_from_top = 7 - rank;
        let ranks_from_bottom = rank;
        let files_from_left = file;
        let files_from_right = 7 - file;

        // West
        if files_from_left >= 1 {
            current_pos_list.push(to_pos(rank, file - 1));
        }
        // South-west
        if ranks_from_bottom >= 1 && files_from_left >= 1 {
            current_pos_list.push(to_pos(rank - 1, file - 1));
        }
        // South
        if ranks_from_bottom >= 1 {
            current_pos_list.push(to_pos(rank - 1, file));
        }
        // South-east
        if ranks_from_bottom >= 1 && files_from_right >= 1 {
            current_pos_list.push(to_pos(rank - 1, file + 1));
        }
        // East
        if files_from_right >= 1 {
            current_pos_list.push(to_pos(rank, file + 1));
        }
        // North-east
        if ranks_from_top >= 1 && files_from_right >= 1 {
            current_pos_list.push(to_pos(rank + 1, file + 1));
        }
        // North
        if ranks_from_top >= 1 {
            current_pos_list.push(to_pos(rank + 1, file));
        }
        // North-West
        if ranks_from_top >= 1 && files_from_left >= 1 {
            current_pos_list.push(to_pos(rank + 1, file - 1));
        }
    }
    target_pos_lists
});
