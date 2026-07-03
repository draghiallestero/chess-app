pub mod bitboard;
pub mod move_sets;
pub use bitboard::BitBoard;
use std::{
    iter::StepBy,
    ops::{Neg, Not, Range},
    sync::LazyLock,
};

use arrayvec::ArrayVec;
use bit_iter::BitIter;

use crate::move_sets::{
    BISHOP_TARGET_POS_LISTS_2D, KING_TARGET_POS_LISTS_2D, KNIGHT_TARGET_POS_LISTS_2D,
    PAWN_ATTACK_POS_LISTS_2D, PAWN_TARGET_POS_LISTS_2D, QUEEN_TARGET_POS_LISTS_2D,
    ROOK_TARGET_POS_LISTS_2D,
};

#[derive(Default, Clone, Copy)]
enum CastlingStatus {
    #[default]
    BothAvailable,
    KingSideAvailable,
    QueenSideAvailable,
    Unavailable,
}

#[derive(Default, Clone, Copy)]
pub struct Player {
    pub pawns: BitBoard,
    pub rooks: BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard,
    pub queens: BitBoard,
    pub kings: BitBoard,
    pub castling_status: CastlingStatus,
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
            ..*self
        }
    }
}

#[derive(Clone, Copy)]
enum EnPassantStatus {
    Unavailable,
    Available,
    Consumed,
}

#[derive(Clone, Copy)]
pub struct Board {
    pub player: Player,
    pub opponent: Player,

    en_passant_available: EnPassantStatus,
}

impl Board {
    pub fn generate_legal_moves(&self) -> Vec<Move> {
        // Setup return vector
        let mut moves = Vec::new();
        let mut try_add_move = |pos, target_pos, new_board: Board| {
            if !new_board.is_king_in_check() {
                moves.push(Move {
                    from: pos,
                    to: target_pos,
                    board: new_board,
                })
            }
        };

        // Occupied squares
        let player_occupied_squares = self.player.occupied_squares();
        let opponent_occupied_squares = self.opponent.occupied_squares();
        let occupied_squares = player_occupied_squares | opponent_occupied_squares;

        // Attack utilities
        let remove_opponent_piece = |new_board: &mut Board, attack_pos: u8| {
            new_board.opponent.pawns.unset(attack_pos);
            new_board.opponent.rooks.unset(attack_pos);
            new_board.opponent.knights.unset(attack_pos);
            new_board.opponent.bishops.unset(attack_pos);
            new_board.opponent.queens.unset(attack_pos);
            new_board.opponent.kings.unset(attack_pos);
        };

        // Pawns
        for pos in BitIter::from(self.player.pawns.board).map(|x| x as u8) {
            // Move
            let target_pos_list = &PAWN_TARGET_POS_LISTS_2D[pos as usize];
            for target_pos in target_pos_list {
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    let mut new_board = *self;
                    new_board.player.pawns.unset(pos);
                    new_board.player.pawns.set(*target_pos);
                    try_add_move(pos, *target_pos, new_board);
                }
            }

            // Attack
            let attack_pos_list = &PAWN_ATTACK_POS_LISTS_2D[pos as usize];
            for attack_pos in attack_pos_list {
                let attack_pos_occupied = opponent_occupied_squares.get(*attack_pos);
                if attack_pos_occupied {
                    let mut new_board = *self;
                    new_board.player.pawns.unset(pos);
                    new_board.player.pawns.set(*attack_pos);
                    remove_opponent_piece(&mut new_board, *attack_pos);
                    try_add_move(pos, *attack_pos, new_board);
                }
            }
        }

        // Rooks
        for pos in BitIter::from(self.player.rooks.board).map(|x| x as u8) {
            for direction in 0..4 {
                let target_pos_list = &ROOK_TARGET_POS_LISTS_2D[pos as usize];
                for target_pos in &target_pos_list[direction] {
                    // Move
                    let target_pos_free = !occupied_squares.get(*target_pos);
                    if target_pos_free {
                        let mut new_board = *self;
                        new_board.player.rooks.unset(pos);
                        new_board.player.rooks.set(*target_pos);
                        try_add_move(pos, *target_pos, new_board);
                    } else {
                        let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                        if attack_pos_occupied {
                            // Attack
                            let mut new_board = *self;
                            new_board.player.rooks.unset(pos);
                            new_board.player.rooks.set(*target_pos);
                            remove_opponent_piece(&mut new_board, *target_pos);
                            try_add_move(pos, *target_pos, new_board);
                        }
                        break;
                    }
                }
            }
        }

        // Knights
        for pos in BitIter::from(self.player.knights.board).map(|x| x as u8) {
            let target_pos_list = &KNIGHT_TARGET_POS_LISTS_2D[pos as usize];
            for target_pos in target_pos_list {
                let mut new_board = *self;
                new_board.player.knights.unset(pos);
                new_board.player.knights.set(*target_pos);
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    // Move
                    try_add_move(pos, *target_pos, new_board);
                } else {
                    let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                    if attack_pos_occupied {
                        // Attack
                        remove_opponent_piece(&mut new_board, *target_pos);
                        try_add_move(pos, *target_pos, new_board);
                    }
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
                        // Move
                        let mut new_board = *self;
                        new_board.player.bishops.unset(pos);
                        new_board.player.bishops.set(*target_pos);
                        try_add_move(pos, *target_pos, new_board);
                    } else {
                        let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                        if attack_pos_occupied {
                            // Attack
                            let mut new_board = *self;
                            new_board.player.bishops.unset(pos);
                            new_board.player.bishops.set(*target_pos);
                            remove_opponent_piece(&mut new_board, *target_pos);
                            try_add_move(pos, *target_pos, new_board);
                        }
                        break;
                    }
                }
            }
        }

        // Queens
        for pos in BitIter::from(self.player.queens.board).map(|x| x as u8) {
            for direction in 0..8 {
                let target_pos_list = &QUEEN_TARGET_POS_LISTS_2D[pos as usize];
                for target_pos in &target_pos_list[direction] {
                    let target_pos_free = !occupied_squares.get(*target_pos);
                    if target_pos_free {
                        // Move
                        let mut new_board = *self;
                        new_board.player.queens.unset(pos);
                        new_board.player.queens.set(*target_pos);
                        try_add_move(pos, *target_pos, new_board);
                    } else {
                        let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                        if attack_pos_occupied {
                            // Attack
                            let mut new_board = *self;
                            new_board.player.queens.unset(pos);
                            new_board.player.queens.set(*target_pos);
                            remove_opponent_piece(&mut new_board, *target_pos);
                            try_add_move(pos, *target_pos, new_board);
                        }
                        break;
                    }
                }
            }
        }

        // Kings
        for pos in BitIter::from(self.player.kings.board).map(|x| x as u8) {
            let target_pos_list = &KING_TARGET_POS_LISTS_2D[pos as usize];
            for target_pos in target_pos_list {
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    // Move
                    let mut new_board = *self;
                    new_board.player.kings.unset(pos);
                    new_board.player.kings.set(*target_pos);
                    try_add_move(pos, *target_pos, new_board);
                } else {
                    let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                    if attack_pos_occupied {
                        // Attack
                        let mut new_board = *self;
                        new_board.player.kings.unset(pos);
                        new_board.player.kings.set(*target_pos);
                        remove_opponent_piece(&mut new_board, *target_pos);
                        try_add_move(pos, *target_pos, new_board);
                    }
                }
            }
        }

        moves
    }

    pub fn flip_view(&self) -> Board {
        Board {
            player: self.opponent.flip_view(),
            opponent: self.player.flip_view(),
            ..Board::default()
        }
    }

    fn is_king_in_check(&self) -> bool {
        false
    }
}

impl Default for Board {
    fn default() -> Board {
        let player = Player {
            pawns: BitBoard::new(
                (1u64) << 8
                    | (1u64) << 9
                    | (1u64) << 10
                    | (1u64) << 11
                    | (1u64) << 12
                    | (1u64) << 13
                    | (1u64) << 14
                    | (1u64) << 15,
            ),
            rooks: BitBoard::new((1u64) << 0 | (1u64) << 7),
            knights: BitBoard::new((1u64) << 1 | (1u64) << 6),
            bishops: BitBoard::new((1u64) << 2 | (1u64) << 5),
            queens: BitBoard::new((1u64) << 3),
            kings: BitBoard::new((1u64) << 4),
            ..Player::default()
        };
        let opponent = Player {
            pawns: BitBoard::new(
                (1u64) << (63 - 8)
                    | (1u64) << (63 - 9)
                    | (1u64) << (63 - 10)
                    | (1u64) << (63 - 11)
                    | (1u64) << (63 - 12)
                    | (1u64) << (63 - 13)
                    | (1u64) << (63 - 14)
                    | (1u64) << (63 - 15),
            ),
            rooks: BitBoard::new((1u64) << (63 - 0) | (1u64) << (63 - 7)),
            knights: BitBoard::new((1u64) << (63 - 1) | (1u64) << (63 - 6)),
            bishops: BitBoard::new((1u64) << (63 - 2) | (1u64) << (63 - 5)),
            queens: BitBoard::new((1u64) << (63 - 4)),
            kings: BitBoard::new((1u64) << (63 - 3)),
            ..Player::default()
        };
        Board {
            player: player,
            opponent: opponent,
            en_passant_available: EnPassantStatus::Unavailable,
        }
    }
}

pub struct Square {
    rank: i32,
    file: i32,
}

pub struct Move {
    pub from: u8,
    pub to: u8,
    pub board: Board,
}

#[cfg(test)]
mod tests {
    fn perft() {}
}
