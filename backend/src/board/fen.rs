use crate::{
    Board, CastlingStatus,
    move_sets::{EN_PASSANT_LISTS, from_chars, to_chars, to_pos},
};

impl Board {
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
