pub mod bitboard;
mod board;
pub mod en_passant;
pub mod move_sets;
pub mod piece_square_tables;
pub use bitboard::BitBoard;
use piece_square_tables::MIDGAME_PAWNS;
use std::{
    cell::RefCell,
    cmp::{Ordering, max, max_by, min},
    fs::OpenOptions,
    iter::StepBy,
    ops::{Neg, Not, Range},
    sync::LazyLock,
    thread::current,
};

use arrayvec::ArrayVec;
use bit_iter::BitIter;

use crate::{
    en_passant::EnPassant,
    move_sets::{
        BISHOP_TARGET_POS_LISTS_2D, EN_PASSANT_LISTS, KING_TARGET_POS_LISTS_2D,
        KNIGHT_TARGET_POS_LISTS_2D, PAWN_ATTACK_POS_LISTS_2D, PAWN_TARGET_POS_LISTS_2D,
        QUEEN_TARGET_POS_LISTS_2D, ROOK_TARGET_POS_LISTS_2D, from_chars, from_pos, to_chars,
        to_pos,
    },
    piece_square_tables::{
        MIDGAME_BISHOPS, MIDGAME_KINGS, MIDGAME_KNIGHTS, MIDGAME_QUEENS, MIDGAME_ROOKS,
    },
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

    pub fn evaluate_position(&self) -> i16 {
        let pawns = 100;
        let knights = 320;
        let bishops = 330;
        let rooks = 500;
        let queens = 900;
        let kings = 20000;

        let material_value = pawns * (self.pawns.count() as i16)
            + knights * (self.knights.count() as i16)
            + bishops * (self.bishops.count() as i16)
            + rooks * (self.rooks.count() as i16)
            + queens * (self.queens.count() as i16)
            + kings * (self.kings.count() as i16);

        let calc_pst_value = |bitboard: BitBoard, pst: [i16; 64]| {
            let mut value = 0;
            for pos in BitIter::from(bitboard.board) {
                value += pst[pos];
            }
            value
        };
        let pawn_pst = calc_pst_value(self.pawns, MIDGAME_PAWNS);
        let knights_pst = calc_pst_value(self.knights, MIDGAME_KNIGHTS);
        let bishops_pst = calc_pst_value(self.bishops, MIDGAME_BISHOPS);
        let rooks_pst = calc_pst_value(self.rooks, MIDGAME_ROOKS);
        let queens_pst = calc_pst_value(self.queens, MIDGAME_QUEENS);
        let kings_pst = calc_pst_value(self.kings, MIDGAME_KINGS);
        let pst_value = pawn_pst + knights_pst + bishops_pst + rooks_pst + queens_pst + kings_pst;

        material_value + pst_value
    }
}

#[derive(Clone, Copy)]
pub struct Board {
    pub player: Player,
    pub opponent: Player,

    pub partial_halfmove_count: i32,
    pub halfmove_count: i32,
    en_passant: EnPassant,
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
            partial_halfmove_count: 0,
            halfmove_count: 0,
            en_passant: EnPassant::default(),
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
            en_passant: EnPassant::default(),
        }
    }

    fn new_move_board(&self) -> Board {
        // For the next move, always reset some members
        Board {
            partial_halfmove_count: self.partial_halfmove_count + 1,
            halfmove_count: self.halfmove_count + 1,
            en_passant: EnPassant::default(),
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
        if rank < 7 {
            if file > 0 {
                let current_pos = to_pos(rank + 1, file - 1);
                if self.opponent.pawns.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                }
            }
            if file < 7 {
                let current_pos = to_pos(rank + 1, file + 1);
                if self.opponent.pawns.get(current_pos) {
                    if RETURN_EARLY {
                        return (attacking_positions, true);
                    }
                    attacking_positions.push(current_pos);
                    return (attacking_positions, true);
                }
            }
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

    pub fn evaluate_position(&self) -> i16 {
        let player_score = self.player.evaluate_position();
        let oppponent_score = self.opponent.evaluate_position();
        player_score - oppponent_score
    }

    pub fn search_for_best_move(&self, max_depth: usize) -> Option<Move> {
        let moves = {
            let mut moves = Vec::default();
            self.generate_legal_moves(&mut moves);
            moves
        };
        if moves.is_empty() {
            return None;
        }
        let mut best_move = Some(moves[0]);

        let mut next_depth_moves_vec = vec![Vec::default(); max_depth - 1];
        let mut alpha = i16::MIN + 1;
        let beta = i16::MAX - 1;
        for _move in moves {
            let alpha_candidate = -_move.board.search_for_best_move_impl(
                next_depth_moves_vec.as_mut_slice(),
                -beta,
                -alpha,
            );
            if alpha_candidate > alpha {
                alpha = alpha_candidate;
                best_move = Some(_move);
            }
        }
        best_move
    }

    pub fn search_for_best_move_impl(
        &self,
        moves_slice: &mut [Vec<Move>],
        alpha: i16,
        beta: i16,
    ) -> i16 {
        if moves_slice.is_empty() {
            return self.evaluate_position();
        }

        let (mut moves, mut moves_slice_split) = moves_slice.split_first_mut().unwrap();
        self.generate_legal_moves(&mut moves);

        if moves.is_empty() {
            return self.evaluate_position();
        }

        let mut alpha = alpha;
        for _move in moves {
            let alpha_candidate =
                -_move
                    .board
                    .search_for_best_move_impl(&mut moves_slice_split, -beta, -alpha);
            alpha = max(alpha, alpha_candidate);
        }

        alpha
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
        let side_to_move = if board.halfmove_count % 2 == 0 {
            'w'
        } else {
            'b'
        };
        fen.push(side_to_move);

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
        match board.en_passant.target_pos {
            64 => fen.push('-'),
            _ => fen.push_str(to_chars(board.en_passant.target_pos)),
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
        let mut player_casting_status = CastlingStatus::Unavailable;
        let mut opponent_casting_status = CastlingStatus::Unavailable;
        let c = fen.chars().nth(fen_ix).unwrap();
        if c == '-' {
            fen_ix += 1;
        } else {
            loop {
                let c = fen.chars().nth(fen_ix).unwrap();
                match c {
                    'K' => player_casting_status = CastlingStatus::KingSideAvailable,
                    'Q' => {
                        player_casting_status = match player_casting_status {
                            CastlingStatus::KingSideAvailable => CastlingStatus::BothAvailable,
                            CastlingStatus::Unavailable => CastlingStatus::QueenSideAvailable,
                            _ => player_casting_status,
                        }
                    }
                    'k' => opponent_casting_status = CastlingStatus::KingSideAvailable,
                    'q' => {
                        opponent_casting_status = match opponent_casting_status {
                            CastlingStatus::KingSideAvailable => CastlingStatus::BothAvailable,
                            CastlingStatus::Unavailable => CastlingStatus::QueenSideAvailable,
                            _ => opponent_casting_status,
                        }
                    }
                    _ => break,
                }
                fen_ix += 1;
            }
        }
        if side_to_move == 'w' {
            board.player.castling_status = player_casting_status;
            board.opponent.castling_status = opponent_casting_status;
        } else {
            board.opponent.castling_status = player_casting_status;
            board.player.castling_status = opponent_casting_status;
        }

        // En passant
        fen_ix += 1;
        let c = fen.chars().nth(fen_ix).unwrap();
        if c != '-' {
            let chars = &fen[fen_ix..fen_ix + 2];
            let target_pos = from_chars(chars);
            if side_to_move == 'w' {
                let pos = target_pos + 8;
                board.en_passant = EN_PASSANT_LISTS[63 - pos as usize];
            } else {
                let pos = target_pos - 8;
                board.en_passant = EN_PASSANT_LISTS[pos as usize].flip_view();
            }
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

        // Fullmove counter
        let chars = &fen[fen_ix..fen.len()];
        board.halfmove_count = 2 * (i32::from_str_radix(chars, 10).unwrap() - 1);
        if side_to_move == 'b' {
            board.halfmove_count += 1;
        }

        board
    }
}

#[derive(Default, Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
    pub board: Board,
    pub evaluation: i16,
}

#[cfg(test)]
mod tests {
    use core::num;
    use std::{array::from_fn, default};

    use super::*;

    fn perft_for_depth(
        board: Board,
        depth: usize,
        max_depth: usize,
        moves_array: &mut [Vec<Move>],
        divide: bool,
    ) -> i64 {
        if depth == max_depth {
            return 1;
        }

        // Split off current vector and get moves
        let (mut moves, mut moves_array) = moves_array.split_first_mut().unwrap();
        board.generate_legal_moves(&mut moves);

        let mut divide_strings = Vec::default();

        let mut num_moves = 0;
        for _move in moves {
            let moves_for_move =
                perft_for_depth(_move.board, depth + 1, max_depth, &mut moves_array, false);
            num_moves += moves_for_move;
            if divide {
                divide_strings.push(format!(
                    "{}{}: {}",
                    to_chars(if _move.board.halfmove_count % 2 == 1 {
                        _move.from
                    } else {
                        63 - _move.from
                    }),
                    to_chars(if _move.board.halfmove_count % 2 == 1 {
                        _move.to
                    } else {
                        63 - _move.to
                    }),
                    moves_for_move
                ));
            }
        }
        // Compare to
        // stockfish
        // position fen <fen>
        // go perft <depth>
        // And wrap that in <cmd> | tail -n +2 | head -n -3 | sort
        if divide {
            divide_strings.sort();
            for divide_string in divide_strings {
                println!("{}", divide_string);
            }
        }
        num_moves
    }

    #[test]
    fn perft() {
        let mut moves_array: [Vec<Move>; 7] = from_fn(|_| Vec::default());

        // Initial position
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft_for_depth(board, 0, 1, &mut moves_array, false), 20);
        assert_eq!(perft_for_depth(board, 0, 2, &mut moves_array, false), 400);
        assert_eq!(perft_for_depth(board, 0, 3, &mut moves_array, false), 8902);
        assert_eq!(
            perft_for_depth(board, 0, 4, &mut moves_array, false),
            197281
        );
        assert_eq!(
            perft_for_depth(board, 0, 5, &mut moves_array, false),
            4865609
        );
        assert_eq!(
            perft_for_depth(board, 0, 6, &mut moves_array, true),
            119060324
        );
        // assert_eq!(
        //     perft_for_depth(board, 0, 7, &mut moves_array, true),
        //     3195901860
        // );

        // Position 2
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft_for_depth(board, 0, 1, &mut moves_array, false), 48);
        assert_eq!(perft_for_depth(board, 0, 2, &mut moves_array, false), 2039);
        assert_eq!(perft_for_depth(board, 0, 3, &mut moves_array, true), 97862);
        assert_eq!(
            perft_for_depth(board, 0, 4, &mut moves_array, false),
            4085603
        );
        // assert_eq!(perft_for_depth(board, 0, 5, &mut moves_array, false), 193690690);

        // Position 3
        let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft_for_depth(board, 0, 1, &mut moves_array, false), 14);
        assert_eq!(perft_for_depth(board, 0, 2, &mut moves_array, false), 191);
        assert_eq!(perft_for_depth(board, 0, 3, &mut moves_array, true), 2812);
        assert_eq!(perft_for_depth(board, 0, 4, &mut moves_array, false), 43238);
        // assert_eq!(perft_for_depth(board, 0, 5, &mut moves_array, false), 674624);

        // Position 4
        let board =
            Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(perft_for_depth(board, 0, 1, &mut moves_array, false), 6);
        assert_eq!(perft_for_depth(board, 0, 2, &mut moves_array, false), 264);
        assert_eq!(perft_for_depth(board, 0, 3, &mut moves_array, true), 9467);
        assert_eq!(
            perft_for_depth(board, 0, 4, &mut moves_array, false),
            422333
        );
        // assert_eq!(perft_for_depth(board, 0, 5, &mut moves_array, false), 15833292);

        // Position 5
        let board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft_for_depth(board, 0, 1, &mut moves_array, false), 44);
        assert_eq!(perft_for_depth(board, 0, 2, &mut moves_array, false), 1486);
        assert_eq!(perft_for_depth(board, 0, 3, &mut moves_array, true), 62379);
        assert_eq!(
            perft_for_depth(board, 0, 4, &mut moves_array, false),
            2103487
        );
        // assert_eq!(perft_for_depth(board, 0, 5, &mut moves_array, false), 89941194);

        // Position 6
        let board = Board::from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        );
        assert_eq!(perft_for_depth(board, 0, 1, &mut moves_array, false), 46);
        assert_eq!(perft_for_depth(board, 0, 2, &mut moves_array, false), 2079);
        assert_eq!(perft_for_depth(board, 0, 3, &mut moves_array, true), 89890);
        assert_eq!(
            perft_for_depth(board, 0, 4, &mut moves_array, false),
            3894594
        );
        // assert_eq!(perft_for_depth(board, 0, 5, &mut moves_array, false), 164075551);
    }

    #[test]
    fn fen() {
        // Initial position
        let expected = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let actual = Board::default().to_fen();
        assert_eq!(expected, actual);

        // Round trip check
        let actual = Board::from_fen(&expected).to_fen();
        assert_eq!(expected, actual);

        // Several round trip checks
        let expected = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let actual = Board::from_fen(&expected).to_fen();
        assert_eq!(expected, actual);
        let expected = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
        let actual = Board::from_fen(&expected).to_fen();
        assert_eq!(expected, actual);
        let expected = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        let actual = Board::from_fen(&expected).to_fen();
        assert_eq!(expected, actual);
    }
}
