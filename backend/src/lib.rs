pub mod bitboard;
pub mod move_sets;
pub use bitboard::BitBoard;
use std::{
    cell::RefCell,
    iter::StepBy,
    ops::{Neg, Not, Range},
    sync::LazyLock,
    thread::current,
};

use arrayvec::ArrayVec;
use bit_iter::BitIter;

use crate::move_sets::{
    BISHOP_TARGET_POS_LISTS_2D, KING_TARGET_POS_LISTS_2D, KNIGHT_TARGET_POS_LISTS_2D,
    PAWN_ATTACK_POS_LISTS_2D, PAWN_TARGET_POS_LISTS_2D, QUEEN_TARGET_POS_LISTS_2D,
    ROOK_TARGET_POS_LISTS_2D, from_chars, from_pos, to_chars, to_pos,
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

    fn add_piece(&mut self, piece: Piece, target_pos: u8) {
        let process_piece = move |piece: &mut BitBoard| {
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

    fn remove_piece(&mut self, piece: Piece, pos: u8) {
        let process_piece = move |piece: &mut BitBoard| {
            piece.unset(pos);
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

    fn remove_any_piece(&mut self, pos: u8) {
        self.pawns.unset(pos);
        self.rooks.unset(pos);
        self.knights.unset(pos);
        self.bishops.unset(pos);
        self.queens.unset(pos);
        self.kings.unset(pos);
    }

    fn move_piece(&mut self, piece: Piece, pos: u8, target_pos: u8) {
        self.remove_piece(piece, pos);
        self.add_piece(piece, target_pos);
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

    pub partial_halfmove_count: i32,
    pub halfmove_count: i32,
    en_passant_pos: Option<u8>,
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
            partial_halfmove_count: 0,
            halfmove_count: 0,
            en_passant_pos: None,
        }
    }
}

impl Board {
    pub fn blank() -> Board {
        Board {
            player: Player::default(),
            opponent: Player::default(),
            partial_halfmove_count: 0,
            halfmove_count: 0,
            en_passant_pos: None,
        }
    }

    pub fn generate_legal_moves(&self) -> Vec<Move> {
        // Setup return vector
        let mut moves = Vec::new();

        // Check whether a move is valid before adding it
        let mut try_add_move = |pos, target_pos, new_board: Board| {
            if !new_board.is_king_in_check() {
                moves.push(Move {
                    from: pos,
                    to: target_pos,
                    board: new_board,
                })
            }
        };

        // Remove attacked piece
        let remove_attacked_piece = |new_board: &mut Board, attack_pos: u8| {
            if (attack_pos == 56 || attack_pos == 63) && new_board.opponent.rooks.get(attack_pos) {
                if attack_pos == 56 {
                    new_board.opponent.castling_status = match new_board.opponent.castling_status {
                        CastlingStatus::BothAvailable => CastlingStatus::QueenSideAvailable,
                        CastlingStatus::KingSideAvailable => CastlingStatus::Unavailable,
                        _ => new_board.opponent.castling_status,
                    }
                } else if attack_pos == 63 {
                    new_board.opponent.castling_status = match new_board.opponent.castling_status {
                        CastlingStatus::BothAvailable => CastlingStatus::KingSideAvailable,
                        CastlingStatus::QueenSideAvailable => CastlingStatus::Unavailable,
                        _ => new_board.opponent.castling_status,
                    }
                }
            }
            new_board.player.remove_piece(Piece::Queens, attack_pos);
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
            for target_pos in target_pos_list {
                let target_pos_free = !occupied_squares.get(*target_pos);
                if target_pos_free {
                    let mut new_board = self.new_move_board();
                    new_board.player.move_piece(Piece::Pawns, pos, *target_pos);
                    // En passant
                    if rank == 1 {
                        new_board.en_passant_pos = Some(*target_pos);
                    }
                    // Promotion
                    if rank == 6 {
                        new_board.player.remove_piece(Piece::Pawns, *target_pos);
                        // Queen promotion goes first as the "preferred" player choice
                        new_board.player.add_piece(Piece::Queens, *target_pos);
                        try_add_move(pos, *target_pos, new_board);
                        // Rook
                        new_board.player.remove_piece(Piece::Queens, *target_pos);
                        new_board.player.add_piece(Piece::Rooks, *target_pos);
                        try_add_move(pos, *target_pos, new_board);
                        // Knight
                        new_board.player.remove_piece(Piece::Rooks, *target_pos);
                        new_board.player.add_piece(Piece::Knights, *target_pos);
                        try_add_move(pos, *target_pos, new_board);
                        // Bishop
                        new_board.player.remove_piece(Piece::Knights, *target_pos);
                        new_board.player.add_piece(Piece::Bishops, *target_pos);
                        try_add_move(pos, *target_pos, new_board);
                    } else {
                        try_add_move(pos, *target_pos, new_board);
                    }
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
                    remove_attacked_piece(&mut new_board, *attack_pos);
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
                            new_board.opponent.remove_any_piece(en_passant_pos);
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
                            remove_attacked_piece(&mut new_board, *target_pos);
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
                        remove_attacked_piece(&mut new_board, *target_pos);
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
                            remove_attacked_piece(&mut new_board, *target_pos);
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
                            remove_attacked_piece(&mut new_board, *target_pos);
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
                        remove_attacked_piece(&mut new_board, *target_pos);
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
            if !self.is_king_in_check() {
                let process_castling = RefCell::new(|neighbor_pos, target_pos, rook_pos| {
                    let rook_target_pos = neighbor_pos;
                    let neighbor_free = !occupied_squares.get(neighbor_pos);
                    let target_pos_free = !occupied_squares.get(target_pos);
                    if neighbor_free && target_pos_free {
                        let mut new_board = self.new_move_board();
                        new_board.player.move_piece(Piece::Kings, pos, neighbor_pos);
                        if !new_board.is_king_in_check() {
                            new_board
                                .player
                                .move_piece(Piece::Kings, neighbor_pos, target_pos);
                            new_board
                                .player
                                .move_piece(Piece::Rooks, rook_pos, rook_target_pos);
                            new_board.player.castling_status = CastlingStatus::Unavailable;
                            try_add_move(pos, target_pos, new_board);
                        }
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
        }

        moves
    }

    fn new_move_board(&self) -> Board {
        // For the next move, always reset some members
        Board {
            partial_halfmove_count: self.partial_halfmove_count + 1,
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

    fn get_attacking_positions<const RETURN_EARLY: bool>(&self, pos: u8) -> (Vec<u8>, bool) {
        let (rank, file) = from_pos(pos);

        let player_occupied_squares = self.player.occupied_squares();
        let opponent_occupied_squares = self.opponent.occupied_squares();

        let mut attacking_positions = Vec::default();

        // Pawns
        let current_pos = to_pos(rank + 1, file);
        if self.opponent.pawns.get(current_pos) {
            if RETURN_EARLY {
                return (attacking_positions, true);
            }
            attacking_positions.push(current_pos);
            return (attacking_positions, true);
        }

        // Cardinal directions
        let current_rank = rank;
        let mut current_file = file;
        // West
        while current_file > 0 {
            current_file -= 1;
            let current_pos = to_pos(current_rank, current_file);
            if player_occupied_squares.get(current_pos) {
                break;
            }
            if opponent_occupied_squares.get(current_pos) {
                if self.opponent.rooks.get(current_pos) || self.opponent.queens.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                } else {
                    break;
                }
            }
        }
        // East
        let current_rank = rank;
        let mut current_file = file;
        while current_file < 7 {
            current_file += 1;
            let current_pos = to_pos(current_rank, current_file);
            if player_occupied_squares.get(current_pos) {
                break;
            }
            if opponent_occupied_squares.get(current_pos) {
                if self.opponent.rooks.get(current_pos) || self.opponent.queens.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                } else {
                    break;
                }
            }
        }
        // North
        let mut current_rank = rank;
        let current_file = file;
        while current_rank < 7 {
            current_rank += 1;
            let current_pos = to_pos(current_rank, current_file);
            if player_occupied_squares.get(current_pos) {
                break;
            }
            if opponent_occupied_squares.get(current_pos) {
                if self.opponent.rooks.get(current_pos) || self.opponent.queens.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                } else {
                    break;
                }
            }
        }
        // South
        let mut current_rank = rank;
        let current_file = file;
        while current_rank > 0 {
            current_rank -= 1;
            let current_pos = to_pos(current_rank, current_file);
            if player_occupied_squares.get(current_pos) {
                break;
            }
            if opponent_occupied_squares.get(current_pos) {
                if self.opponent.rooks.get(current_pos) || self.opponent.queens.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                } else {
                    break;
                }
            }
        }

        // Diagonal directions
        let mut current_rank = rank;
        let mut current_file = file;
        // South-west
        while current_rank > 0 && current_file > 0 {
            current_rank -= 1;
            current_file -= 1;
            let current_pos = to_pos(current_rank, current_file);
            if player_occupied_squares.get(current_pos) {
                break;
            }
            if opponent_occupied_squares.get(current_pos) {
                if self.opponent.bishops.get(current_pos) || self.opponent.queens.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                } else {
                    break;
                }
            }
        }
        // South-east
        let mut current_rank = rank;
        let mut current_file = file;
        while current_rank > 0 && current_file < 7 {
            current_rank -= 1;
            current_file += 1;
            let current_pos = to_pos(current_rank, current_file);
            if player_occupied_squares.get(current_pos) {
                break;
            }
            if opponent_occupied_squares.get(current_pos) {
                if self.opponent.bishops.get(current_pos) || self.opponent.queens.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                } else {
                    break;
                }
            }
        }
        // North-east
        let mut current_rank = rank;
        let mut current_file = file;
        while current_rank < 7 && current_file < 7 {
            current_rank += 1;
            current_file += 1;
            let current_pos = to_pos(current_rank, current_file);
            if player_occupied_squares.get(current_pos) {
                break;
            }
            if opponent_occupied_squares.get(current_pos) {
                if self.opponent.bishops.get(current_pos) || self.opponent.queens.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                } else {
                    break;
                }
            }
        }
        // North-west
        let mut current_rank = rank;
        let mut current_file = file;
        while current_rank < 7 && current_file > 0 {
            current_rank += 1;
            current_file -= 1;
            let current_pos = to_pos(current_rank, current_file);
            if player_occupied_squares.get(current_pos) {
                break;
            }
            if opponent_occupied_squares.get(current_pos) {
                if self.opponent.bishops.get(current_pos) || self.opponent.queens.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                } else {
                    break;
                }
            }
        }

        // Knights
        for current_pos in &KNIGHT_TARGET_POS_LISTS_2D[pos as usize] {
            if self.opponent.knights.get(*current_pos) {
                if RETURN_EARLY {
                    return (attacking_positions, true);
                }
                attacking_positions.push(*current_pos);
                return (attacking_positions, true);
            }
        }

        (attacking_positions, false)
    }

    fn is_king_in_check(&self) -> bool {
        let pos = BitIter::from(self.player.kings.board).next().unwrap() as u8;

        if self.get_attacking_positions::<true>(pos).0.len() != 0 {
            println!(
                "squares attacking king: {:?}",
                self.get_attacking_positions::<true>(pos).0
            );
        }

        return self.get_attacking_positions::<true>(pos).1;
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::default();
        fen.reserve(63);

        // FEN is always from the player's perspective
        let mut board = *self;
        if board.halfmove_count % 2 == 1 {
            board = board.flip_view();
        }

        // Piece placement
        let push_piece = |fen: &mut String, empty_squares: &mut i32, pos| {
            let mut process_case = |c| {
                if *empty_squares != 0 {
                    fen.push_str(&empty_squares.to_string());
                    *empty_squares = 0;
                }
                fen.push(c);
            };
            // Player
            if board.player.pawns.get(pos) {
                process_case('P');
            } else if board.player.knights.get(pos) {
                process_case('N');
            } else if board.player.bishops.get(pos) {
                process_case('B');
            } else if board.player.rooks.get(pos) {
                process_case('R');
            } else if board.player.queens.get(pos) {
                process_case('Q');
            } else if board.player.kings.get(pos) {
                process_case('K');
            }
            // Opponent
            else if board.opponent.pawns.get(pos) {
                process_case('p');
            } else if board.opponent.knights.get(pos) {
                process_case('n');
            } else if board.opponent.bishops.get(pos) {
                process_case('b');
            } else if board.opponent.rooks.get(pos) {
                process_case('r');
            } else if board.opponent.queens.get(pos) {
                process_case('q');
            } else if board.opponent.kings.get(pos) {
                process_case('k');
            }
            // Empty
            else {
                *empty_squares += 1;
            }
        };
        let mut empty_squares = 0;
        let process_empty_squares = |fen: &mut String, empty_squares: &mut i32| {
            if *empty_squares != 0 {
                fen.push_str(&empty_squares.to_string());
                *empty_squares = 0;
            }
        };
        let square_ranges = [
            56u8..64u8,
            48u8..56u8,
            40u8..48u8,
            32u8..40u8,
            24u8..32u8,
            16u8..24u8,
            8u8..16u8,
            0u8..8u8,
        ];
        for (i, square_range) in square_ranges.iter().enumerate() {
            if i != 0 {
                fen.push('/');
            }
            for pos in square_range.clone().into_iter() {
                push_piece(&mut fen, &mut empty_squares, pos);
            }
            process_empty_squares(&mut fen, &mut empty_squares);
        }

        // Side to move
        fen.push(' ');
        fen.push(if board.halfmove_count % 2 == 0 {
            'w'
        } else {
            'b'
        });

        // Castling ability
        fen.push(' ');
        if matches!(board.player.castling_status, CastlingStatus::Unavailable)
            && matches!(board.opponent.castling_status, CastlingStatus::Unavailable)
        {
            fen.push('-');
        } else {
            match board.player.castling_status {
                CastlingStatus::BothAvailable => {
                    fen.push('K');
                    fen.push('Q');
                }
                CastlingStatus::KingSideAvailable => {
                    fen.push('K');
                }
                CastlingStatus::QueenSideAvailable => {
                    fen.push('Q');
                }
                CastlingStatus::Unavailable => {}
            }
            match board.opponent.castling_status {
                CastlingStatus::BothAvailable => {
                    fen.push('k');
                    fen.push('q');
                }
                CastlingStatus::KingSideAvailable => {
                    fen.push('k');
                }
                CastlingStatus::QueenSideAvailable => {
                    fen.push('q');
                }
                CastlingStatus::Unavailable => {}
            }
        }

        // En passant
        fen.push(' ');
        match board.en_passant_pos {
            Some(pos) => fen.push_str(to_chars(pos)),
            None => fen.push('-'),
        }

        // Halfmove clock
        fen.push(' ');
        fen.push_str(&board.partial_halfmove_count.to_string());

        // Fullmove counter
        fen.push(' ');
        fen.push_str(&(board.halfmove_count / 2 + 1).to_string());

        fen
    }

    pub fn from_fen(fen: &str) -> Board {
        let mut board = Board::blank();

        let mut fen_ix = 0;

        // Piece placement
        for rank in (0u8..8u8).rev() {
            if rank != 7 {
                fen_ix += 1;
            }
            let mut file = 0u8;
            while file < 8u8 {
                let pos = to_pos(rank, file);
                let c = fen.chars().nth(fen_ix).unwrap();
                match c {
                    'P' => {
                        board.player.pawns.set(pos);
                        file += 1
                    }
                    'N' => {
                        board.player.knights.set(pos);
                        file += 1
                    }
                    'R' => {
                        board.player.rooks.set(pos);
                        file += 1
                    }
                    'B' => {
                        board.player.bishops.set(pos);
                        file += 1
                    }
                    'Q' => {
                        board.player.queens.set(pos);
                        file += 1
                    }
                    'K' => {
                        board.player.kings.set(pos);
                        file += 1
                    }
                    'p' => {
                        board.opponent.pawns.set(pos);
                        file += 1
                    }
                    'n' => {
                        board.opponent.knights.set(pos);
                        file += 1
                    }
                    'r' => {
                        board.opponent.rooks.set(pos);
                        file += 1
                    }
                    'b' => {
                        board.opponent.bishops.set(pos);
                        file += 1
                    }
                    'q' => {
                        board.opponent.queens.set(pos);
                        file += 1
                    }
                    'k' => {
                        board.opponent.kings.set(pos);
                        file += 1
                    }
                    '1' => file += 1,
                    '2' => file += 2,
                    '3' => file += 3,
                    '4' => file += 4,
                    '5' => file += 5,
                    '6' => file += 6,
                    '7' => file += 7,
                    '8' => file += 8,
                    _ => assert!(false),
                }
                fen_ix += 1;
            }
        }

        // Side to move
        fen_ix += 1;
        let side_to_move = fen.chars().nth(fen_ix).unwrap();
        if side_to_move == 'b' {
            board = board.flip_view();
        }
        fen_ix += 1;

        // Castling ability
        fen_ix += 1;
        loop {
            let c = fen.chars().nth(fen_ix).unwrap();
            match c {
                'K' => board.player.castling_status = CastlingStatus::KingSideAvailable,
                'Q' => {
                    board.player.castling_status = match board.player.castling_status {
                        CastlingStatus::KingSideAvailable => CastlingStatus::BothAvailable,
                        CastlingStatus::Unavailable => CastlingStatus::QueenSideAvailable,
                        _ => board.player.castling_status,
                    }
                }
                'k' => board.opponent.castling_status = CastlingStatus::KingSideAvailable,
                'q' => {
                    board.opponent.castling_status = match board.opponent.castling_status {
                        CastlingStatus::KingSideAvailable => CastlingStatus::BothAvailable,
                        CastlingStatus::Unavailable => CastlingStatus::QueenSideAvailable,
                        _ => board.opponent.castling_status,
                    }
                }
                _ => break,
            }
            fen_ix += 1;
        }

        // En passant
        fen_ix += 1;
        let c = fen.chars().nth(fen_ix).unwrap();
        if c != '-' {
            let chars = &fen[fen_ix..fen_ix + 2];
            board.en_passant_pos = Some(from_chars(chars));
            fen_ix += 1;
        }
        fen_ix += 1;

        // Halfmove clock
        fen_ix += 1;
        let mut fen_ix2 = fen_ix;
        loop {
            let c = fen.chars().nth(fen_ix2).unwrap();
            if c == ' ' {
                break;
            }
            fen_ix2 += 1;
        }
        let chars = &fen[fen_ix..fen_ix2];
        board.partial_halfmove_count = i32::from_str_radix(chars, 10).unwrap();
        fen_ix = fen_ix2 + 1;

        // Halfmove clock
        let chars = &fen[fen_ix..fen.len()];
        board.halfmove_count = 2 * (i32::from_str_radix(chars, 10).unwrap() - 1);
        if side_to_move == 'b' {
            board.halfmove_count += 1;
        }

        board
    }
}

pub struct Move {
    pub from: u8,
    pub to: u8,
    pub board: Board,
}

#[cfg(test)]
mod tests {
    use core::num;

    use super::*;

    fn perft_for_depth(board: Board, depth: i32, max_depth: i32) -> i32 {
        let legal_moves = board.generate_legal_moves();
        if depth == max_depth {
            return 1;
        }

        let mut num_moves = 0;
        for _move in legal_moves {
            num_moves += perft_for_depth(_move.board, depth + 1, max_depth);
        }
        num_moves
    }

    #[test]
    fn perft() {
        let board = Board::default();
        assert_eq!(perft_for_depth(board, 0, 1), 20);
        // assert_eq!(perft_for_depth(board, 0, 2), 400);
    }

    #[test]
    fn fen() {
        // Starting position
        let fen = Board::default().to_fen();
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );

        // Round trip check
        let fen = Board::from_fen(&fen).to_fen();
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );

        // Several round trip checks
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        assert_eq!(fen, Board::from_fen(&fen).to_fen());
        let fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
        assert_eq!(fen, Board::from_fen(&fen).to_fen());
        let fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        assert_eq!(fen, Board::from_fen(&fen).to_fen());
    }
}
