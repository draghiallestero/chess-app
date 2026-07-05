pub mod bitboard;
pub mod move_sets;
pub use bitboard::BitBoard;
use std::{
    cell::RefCell,
    iter::StepBy,
    ops::{Neg, Not, Range},
    sync::LazyLock,
};

use arrayvec::ArrayVec;
use bit_iter::BitIter;

use crate::move_sets::{
    BISHOP_TARGET_POS_LISTS_2D, KING_TARGET_POS_LISTS_2D, KNIGHT_TARGET_POS_LISTS_2D,
    PAWN_ATTACK_POS_LISTS_2D, PAWN_TARGET_POS_LISTS_2D, QUEEN_TARGET_POS_LISTS_2D,
    ROOK_TARGET_POS_LISTS_2D, from_pos, to_pos,
};

#[derive(Default, Clone, Copy)]
enum CastlingStatus {
    #[default]
    BothAvailable,
    KingSideAvailable,
    QueenSideAvailable,
    Unavailable,
}

#[derive(Clone, Copy)]
enum Piece {
    Pawns,
    Rooks,
    Knights,
    Bishops,
    Queens,
    Kings,
}

#[derive(Default, Clone, Copy)]
pub struct Player {
    pub pawns: BitBoard,
    pub rooks: BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard,
    pub queens: BitBoard,
    pub kings: BitBoard,
    castling_status: CastlingStatus,
    king_starting_pos: u8,
}

impl Player {
    pub fn occupied_squares(&self) -> BitBoard {
        self.pawns | self.rooks | self.knights | self.bishops | self.queens | self.kings
    }

    fn move_piece(&mut self, piece: Piece, pos: u8, target_pos: u8) {
        let process_piece = move |piece: &mut BitBoard| {
            piece.unset(pos);
            piece.set(target_pos);
        };
        match piece {
            Piece::Pawns => process_piece(&mut self.pawns),
            Piece::Rooks => process_piece(&mut self.rooks),
            Piece::Knights => process_piece(&mut self.knights),
            Piece::Bishops => process_piece(&mut self.bishops),
            Piece::Queens => process_piece(&mut self.queens),
            Piece::Kings => process_piece(&mut self.kings),
        }
    }

    fn remove_piece(&mut self, pos: u8) {
        self.pawns.unset(pos);
        self.rooks.unset(pos);
        self.knights.unset(pos);
        self.bishops.unset(pos);
        self.queens.unset(pos);
        self.kings.unset(pos);
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
pub struct Board {
    pub player: Player,
    pub opponent: Player,

    en_passant_pos: Option<u8>,
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

        // Pawns
        for pos in BitIter::from(self.player.pawns.board).map(|x| x as u8) {
            let (rank, file) = from_pos(pos);

            // Move
            let target_pos_list = &PAWN_TARGET_POS_LISTS_2D[pos as usize];
            for (i, target_pos) in target_pos_list.iter().enumerate() {
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    let mut new_board = self.new_move_board();
                    new_board.player.move_piece(Piece::Pawns, pos, *target_pos);
                    // En passant
                    if i == 1 {
                        new_board.en_passant_pos = Some(*target_pos);
                    }
                    try_add_move(pos, *target_pos, new_board);
                } else {
                    break;
                }
            }

            // Attack
            let attack_pos_list = &PAWN_ATTACK_POS_LISTS_2D[pos as usize];
            for attack_pos in attack_pos_list {
                let attack_pos_occupied = opponent_occupied_squares.get(*attack_pos);
                if attack_pos_occupied {
                    let mut new_board = self.new_move_board();
                    new_board.player.move_piece(Piece::Pawns, pos, *attack_pos);
                    new_board.opponent.remove_piece(*attack_pos);
                    try_add_move(pos, *attack_pos, new_board);
                }
            }

            // Attack en passant
            if rank == 4 {
                match self.en_passant_pos {
                    Some(en_passant_pos) => {
                        // The other player's rank is from their point-of-view, so adjust
                        let en_passant_pos = 63 - en_passant_pos;
                        let (en_passant_rank, en_passant_file) = from_pos(en_passant_pos);
                        let attack_pos = to_pos(en_passant_rank + 1, en_passant_file);
                        if file.wrapping_sub(1) == en_passant_file || file + 1 == en_passant_file {
                            let mut new_board = self.new_move_board();
                            new_board.player.move_piece(Piece::Pawns, pos, attack_pos);
                            new_board.opponent.remove_piece(en_passant_pos);
                            try_add_move(pos, attack_pos, new_board);
                        }
                    }
                    None => (),
                }
            }
        }

        // Rooks
        let update_castling_status_when_rook_moves = |pos, new_board: &mut Board| {
            if pos == 0 {
                match new_board.player.castling_status {
                    CastlingStatus::BothAvailable => {
                        new_board.player.castling_status = CastlingStatus::KingSideAvailable
                    }
                    CastlingStatus::QueenSideAvailable => {
                        new_board.player.castling_status = CastlingStatus::Unavailable
                    }
                    _ => (),
                }
            } else if pos == 7 {
                match new_board.player.castling_status {
                    CastlingStatus::BothAvailable => {
                        new_board.player.castling_status = CastlingStatus::QueenSideAvailable
                    }
                    CastlingStatus::KingSideAvailable => {
                        new_board.player.castling_status = CastlingStatus::Unavailable
                    }
                    _ => (),
                }
            }
        };
        for pos in BitIter::from(self.player.rooks.board).map(|x| x as u8) {
            for direction in 0..4 {
                let target_pos_list = &ROOK_TARGET_POS_LISTS_2D[pos as usize];
                for target_pos in &target_pos_list[direction] {
                    // Move
                    let target_pos_free = !occupied_squares.get(*target_pos);
                    if target_pos_free {
                        let mut new_board = self.new_move_board();
                        new_board.player.move_piece(Piece::Rooks, pos, *target_pos);
                        update_castling_status_when_rook_moves(pos, &mut new_board);
                        try_add_move(pos, *target_pos, new_board);
                    } else {
                        let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                        if attack_pos_occupied {
                            // Attack
                            let mut new_board = self.new_move_board();
                            new_board.player.move_piece(Piece::Rooks, pos, *target_pos);
                            new_board.opponent.remove_piece(*target_pos);
                            update_castling_status_when_rook_moves(pos, &mut new_board);
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
                let mut new_board = self.new_move_board();
                new_board
                    .player
                    .move_piece(Piece::Knights, pos, *target_pos);
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    // Move
                    try_add_move(pos, *target_pos, new_board);
                } else {
                    let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                    if attack_pos_occupied {
                        // Attack
                        new_board.opponent.remove_piece(*target_pos);
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
                        let mut new_board = self.new_move_board();
                        new_board
                            .player
                            .move_piece(Piece::Bishops, pos, *target_pos);
                        try_add_move(pos, *target_pos, new_board);
                    } else {
                        let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                        if attack_pos_occupied {
                            // Attack
                            let mut new_board = self.new_move_board();
                            new_board
                                .player
                                .move_piece(Piece::Bishops, pos, *target_pos);
                            new_board.opponent.remove_piece(*target_pos);
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
                        let mut new_board = self.new_move_board();
                        new_board.player.move_piece(Piece::Queens, pos, *target_pos);
                        try_add_move(pos, *target_pos, new_board);
                    } else {
                        let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                        if attack_pos_occupied {
                            // Attack
                            let mut new_board = self.new_move_board();
                            new_board.player.move_piece(Piece::Queens, pos, *target_pos);
                            new_board.opponent.remove_piece(*target_pos);
                            try_add_move(pos, *target_pos, new_board);
                        }
                        break;
                    }
                }
            }
        }

        // Kings
        let update_castling_status_when_king_moves =
            |pos, king_starting_pos, new_board: &mut Board| {
                if pos == king_starting_pos {
                    new_board.player.castling_status = CastlingStatus::Unavailable
                }
            };
        for pos in BitIter::from(self.player.kings.board).map(|x| x as u8) {
            let target_pos_list = &KING_TARGET_POS_LISTS_2D[pos as usize];
            for target_pos in target_pos_list {
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    // Move
                    let mut new_board = self.new_move_board();
                    new_board.player.move_piece(Piece::Kings, pos, *target_pos);
                    update_castling_status_when_king_moves(
                        pos,
                        self.player.king_starting_pos,
                        &mut new_board,
                    );
                    try_add_move(pos, *target_pos, new_board);
                } else {
                    let attack_pos_occupied = opponent_occupied_squares.get(*target_pos);
                    if attack_pos_occupied {
                        // Attack
                        let mut new_board = self.new_move_board();
                        new_board.player.move_piece(Piece::Kings, pos, *target_pos);
                        new_board.opponent.remove_piece(*target_pos);
                        update_castling_status_when_king_moves(
                            pos,
                            self.player.king_starting_pos,
                            &mut new_board,
                        );
                        try_add_move(pos, *target_pos, new_board);
                    }
                }
            }

            // Castling
            let process_castling = RefCell::new(|neighbor_pos, target_pos, rook_pos| {
                let rook_target_pos = neighbor_pos;
                let neighbor_free = !occupied_squares.get(neighbor_pos);
                let target_pos_free = !occupied_squares.get(target_pos);
                if neighbor_free && target_pos_free {
                    let mut new_board = self.new_move_board();
                    new_board.player.move_piece(Piece::Kings, pos, target_pos);
                    new_board
                        .player
                        .move_piece(Piece::Rooks, rook_pos, rook_target_pos);
                    new_board.player.castling_status = CastlingStatus::Unavailable;
                    try_add_move(pos, target_pos, new_board);
                }
            });
            let process_kingside_castling = || {
                let file = pos;
                let neighbor_pos = file + 1;
                let target_pos = file + 2;
                let rook_pos = 7;
                (process_castling.borrow_mut())(neighbor_pos, target_pos, rook_pos);
            };
            let process_queenside_castling = || {
                let file = pos;
                let neighbor_pos = file - 1;
                let target_pos = file - 2;
                let rook_pos = 0;
                (process_castling.borrow_mut())(neighbor_pos, target_pos, rook_pos);
            };
            match self.player.castling_status {
                CastlingStatus::BothAvailable => {
                    process_kingside_castling();
                    process_queenside_castling();
                }
                _ => (),
            }
        }

        moves
    }

    fn new_move_board(&self) -> Board {
        // For the next move, always reset some members
        Board {
            en_passant_pos: None,
            ..*self
        }
    }

    pub fn flip_view(&self) -> Board {
        Board {
            player: self.opponent.flip_view(),
            opponent: self.player.flip_view(),
            ..*self
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
            king_starting_pos: 4,
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
            king_starting_pos: 3,
            ..Player::default()
        };
        Board {
            player: player,
            opponent: opponent,
            en_passant_pos: None,
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
