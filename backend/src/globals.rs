use std::array::from_fn;

use crate::*;

use arrayvec::ArrayVec;

pub struct Globals {
    pub paw_target_pos_lists_2d: [ArrayVec<u8, 2>; 64],
    pub pawn_attack_pos_lists_2d: [ArrayVec<u8, 2>; 64],
    pub en_passant_list: [EnPassant; 64],
    pub knight_bitboards: [BitBoard; 64],
    pub rook_magic_bitboards: [MagicBitboard; 64],
    pub bishop_magic_bitboards: [MagicBitboard; 64],
    pub king_bitboards: [BitBoard; 64],

    pub moves_vec: MovesVec,
}

impl Default for Globals {
    fn default() -> Self {
        // Pawns can move forward one or two pieces
        let paw_target_pos_lists_2d = {
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
        };

        // Pawns can attack diagonally forward
        let pawn_attack_pos_lists_2d = {
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
        };

        // En passant
        let en_passant_list = {
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
        };

        // Knights can move over two squares then one square, either first horizontally or first vertically
        let knight_bitboards = {
            let to_pos = |rank, file| 8 * rank + file;

            let mut bitboards: [BitBoard; 64] = from_fn(|_| BitBoard::default());
            for pos in 0u8..64u8 {
                let (rank, file) = (pos / 8, pos % 8);

                let mut bitboard = BitBoard::default();

                // Constraints
                let ranks_from_top = 7 - rank;
                let ranks_from_bottom = rank;
                let files_from_left = file;
                let files_from_right = 7 - file;

                // West-south-west corner
                if ranks_from_bottom >= 1 && files_from_left >= 2 {
                    bitboard.set(to_pos(rank - 1, file - 2));
                }
                // South-south-west corner
                if ranks_from_bottom >= 2 && files_from_left >= 1 {
                    bitboard.set(to_pos(rank - 2, file - 1));
                }
                // South-south-east corner
                if ranks_from_bottom >= 2 && files_from_right >= 1 {
                    bitboard.set(to_pos(rank - 2, file + 1));
                }
                // East-south-east corner
                if ranks_from_bottom >= 1 && files_from_right >= 2 {
                    bitboard.set(to_pos(rank - 1, file + 2));
                }
                // East-north-east corner
                if ranks_from_top >= 1 && files_from_right >= 2 {
                    bitboard.set(to_pos(rank + 1, file + 2));
                }
                // North-north-east corner
                if ranks_from_top >= 2 && files_from_right >= 1 {
                    bitboard.set(to_pos(rank + 2, file + 1));
                }
                // North-north-west corner
                if ranks_from_top >= 2 && files_from_left >= 1 {
                    bitboard.set(to_pos(rank + 2, file - 1));
                }
                // West-north-west corner
                if ranks_from_top >= 1 && files_from_left >= 2 {
                    bitboard.set(to_pos(rank + 1, file - 2));
                }

                bitboards[pos as usize] = bitboard;
            }
            bitboards
        };

        // Rooks can slide along the cardinal directions
        let rook_magic_bitboards =
            MagicBitboard::generate_sliding_piece_magic_bitboards(true, false);

        // Bishops can slide along the diagonal directions
        let bishop_magic_bitboards =
            MagicBitboard::generate_sliding_piece_magic_bitboards(false, true);

        // Kings can move one square in any direction
        let king_bitboards = {
            let to_pos = |rank, file| 8 * rank + file;

            let mut bitboards: [BitBoard; 64] = from_fn(|_| BitBoard::default());

            for pos in 0u8..64u8 {
                let (rank, file) = (pos / 8, pos % 8);

                let mut bitboard = BitBoard::default();

                // Constraints
                let ranks_from_top = 7 - rank;
                let ranks_from_bottom = rank;
                let files_from_left = file;
                let files_from_right = 7 - file;

                // West
                if files_from_left >= 1 {
                    bitboard.set(to_pos(rank, file - 1));
                }
                // South-west
                if ranks_from_bottom >= 1 && files_from_left >= 1 {
                    bitboard.set(to_pos(rank - 1, file - 1));
                }
                // South
                if ranks_from_bottom >= 1 {
                    bitboard.set(to_pos(rank - 1, file));
                }
                // South-east
                if ranks_from_bottom >= 1 && files_from_right >= 1 {
                    bitboard.set(to_pos(rank - 1, file + 1));
                }
                // East
                if files_from_right >= 1 {
                    bitboard.set(to_pos(rank, file + 1));
                }
                // North-east
                if ranks_from_top >= 1 && files_from_right >= 1 {
                    bitboard.set(to_pos(rank + 1, file + 1));
                }
                // North
                if ranks_from_top >= 1 {
                    bitboard.set(to_pos(rank + 1, file));
                }
                // North-West
                if ranks_from_top >= 1 && files_from_left >= 1 {
                    bitboard.set(to_pos(rank + 1, file - 1));
                }

                bitboards[pos as usize] = bitboard;
            }

            bitboards
        };

        Globals {
            paw_target_pos_lists_2d: paw_target_pos_lists_2d,
            pawn_attack_pos_lists_2d: pawn_attack_pos_lists_2d,
            en_passant_list: en_passant_list,
            knight_bitboards: knight_bitboards,
            rook_magic_bitboards: rook_magic_bitboards,
            bishop_magic_bitboards: bishop_magic_bitboards,
            king_bitboards: king_bitboards,

            moves_vec: MovesVec::default(),
        }
    }
}
