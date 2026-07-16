use std::cell::RefCell;

use bit_iter::BitIter;

use crate::*;

impl Board {
    pub fn generate_legal_moves(&self, globals: &mut Globals) -> Box<Moves> {
        let mut moves = globals.moves_vec.pop_moves();
        let player_occupied_squares = self.player.occupied_squares();
        let opponent_occupied_squares = self.opponent.occupied_squares();
        let occupied_squares = player_occupied_squares | opponent_occupied_squares;

        self.generate_pawn_moves(
            globals,
            &mut moves,
            player_occupied_squares,
            opponent_occupied_squares,
            occupied_squares,
        );
        self.generate_knight_moves(
            globals,
            &mut moves,
            player_occupied_squares,
            opponent_occupied_squares,
            occupied_squares,
        );
        self.generate_rook_moves(
            globals,
            &mut moves,
            player_occupied_squares,
            opponent_occupied_squares,
            occupied_squares,
        );
        self.generate_bishop_moves(
            globals,
            &mut moves,
            player_occupied_squares,
            opponent_occupied_squares,
            occupied_squares,
        );
        self.generate_queen_moves(
            globals,
            &mut moves,
            player_occupied_squares,
            opponent_occupied_squares,
            occupied_squares,
        );
        self.generate_king_moves(
            globals,
            &mut moves,
            player_occupied_squares,
            opponent_occupied_squares,
            occupied_squares,
        );

        moves
    }

    fn generate_pawn_moves(
        &self,
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        _: BitBoard,
        opponent_occupied_squares: BitBoard,
        occupied_squares: BitBoard,
    ) {
        for pos in BitIter::from(self.player.pawns.board).map(|x| x as u8) {
            let (rank, file) = from_pos(pos);
            if rank == 7 {
                continue;
            }

            // Pawns can only move forward
            let target_pos = to_pos(rank + 1, file);
            let target_pos_free = !occupied_squares.get(target_pos);
            if target_pos_free {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Pawns, pos, target_pos);
                self.pawn_try_add_move(globals, moves, pos, target_pos, new_board);
                // Double push
                if rank == 1 {
                    let target_pos = to_pos(3, file);
                    let target_pos_free = !occupied_squares.get(target_pos);
                    if target_pos_free {
                        let mut new_board = self.new_move_board();
                        new_board.player.move_piece(Piece::Pawns, pos, target_pos);
                        new_board.en_passant = globals.en_passant_list[pos as usize];
                        self.pawn_try_add_move(globals, moves, pos, target_pos, new_board);
                    }
                }
            }

            // Attack
            if file > 0 {
                self.pawn_try_attack(
                    globals,
                    moves,
                    opponent_occupied_squares,
                    pos,
                    rank + 1,
                    file - 1,
                );
            }
            if file < 7 {
                self.pawn_try_attack(
                    globals,
                    moves,
                    opponent_occupied_squares,
                    pos,
                    rank + 1,
                    file + 1,
                );
            }

            // En passant
            if pos == self.en_passant.attacker_pos_list[0]
                || pos == self.en_passant.attacker_pos_list[1]
            {
                let attack_pos = self.en_passant.target_pos - 8;
                let mut new_board = self.new_move_board();
                new_board
                    .player
                    .move_piece(Piece::Pawns, pos, self.en_passant.target_pos);
                new_board.opponent.remove_any_piece(attack_pos);
                self.pawn_try_add_move(globals, moves, pos, attack_pos, new_board);
            }
        }
    }

    fn pawn_try_add_move(
        &self,
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        pos: u8,
        target_pos: u8,
        mut new_board: Board,
    ) {
        let (target_rank, _) = from_pos(target_pos);
        // Promotion
        if target_rank == 7 {
            new_board.player.remove_piece(Piece::Pawns, target_pos);
            // Queen promotion goes first as the "preferred" player choice
            new_board.player.add_piece(Piece::Queens, target_pos);
            Board::try_add_move(globals, moves, pos, target_pos, new_board);
            new_board.player.remove_piece(Piece::Queens, target_pos);
            // Rook
            new_board.player.add_piece(Piece::Rooks, target_pos);
            Board::try_add_move(globals, moves, pos, target_pos, new_board);
            new_board.player.remove_piece(Piece::Rooks, target_pos);
            // Knight
            new_board.player.add_piece(Piece::Knights, target_pos);
            Board::try_add_move(globals, moves, pos, target_pos, new_board);
            new_board.player.remove_piece(Piece::Knights, target_pos);
            // Bishop
            new_board.player.add_piece(Piece::Bishops, target_pos);
            Board::try_add_move(globals, moves, pos, target_pos, new_board);
        } else {
            Board::try_add_move(globals, moves, pos, target_pos, new_board);
        }
    }

    fn pawn_try_attack(
        &self,
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        opponent_occupied_squares: BitBoard,
        pos: u8,
        attack_rank: u8,
        attack_file: u8,
    ) {
        let attack_pos = to_pos(attack_rank, attack_file);
        let attack_pos_occupied = opponent_occupied_squares.get(attack_pos);
        if attack_pos_occupied {
            let mut new_board = self.new_move_board();
            new_board.player.move_piece(Piece::Pawns, pos, attack_pos);
            Board::remove_attacked_piece(&mut new_board, attack_pos);
            self.pawn_try_add_move(globals, moves, pos, attack_pos, new_board);
        }
    }

    fn generate_knight_moves(
        &self,
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        _: BitBoard,
        opponent_occupied_squares: BitBoard,
        occupied_squares: BitBoard,
    ) {
        for pos in BitIter::from(self.player.knights.board).map(|x| x as u8) {
            let attack_bitboard = globals.knight_bitboards[pos as usize];

            // Move
            for target_pos in
                BitIter::from(attack_bitboard.board & !occupied_squares.board).map(|x| x as u8)
            {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Knights, pos, target_pos);
                Board::try_add_move(globals, moves, pos, target_pos, new_board);
            }

            // Attack
            for attack_pos in BitIter::from(attack_bitboard.board & opponent_occupied_squares.board)
                .map(|x| x as u8)
            {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Knights, pos, attack_pos);
                Board::remove_attacked_piece(&mut new_board, attack_pos);
                Board::try_add_move(globals, moves, pos, attack_pos, new_board);
            }
        }
    }

    fn generate_rook_moves(
        &self,
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        _: BitBoard,
        opponent_occupied_squares: BitBoard,
        occupied_squares: BitBoard,
    ) {
        for pos in BitIter::from(self.player.rooks.board).map(|x| x as u8) {
            let attack_bitboard = {
                let mb = &globals.rook_magic_bitboards[pos as usize];
                mb.get_attack_bitboard(occupied_squares)
            };

            // Move
            for target_pos in
                BitIter::from(attack_bitboard.board & !occupied_squares.board).map(|x| x as u8)
            {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Rooks, pos, target_pos);
                self.rook_update_castling_status_after_move(pos, &mut new_board);
                Board::try_add_move(globals, moves, pos, target_pos, new_board);
            }

            // Attack
            for attack_pos in BitIter::from(attack_bitboard.board & opponent_occupied_squares.board)
                .map(|x| x as u8)
            {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Rooks, pos, attack_pos);
                Board::remove_attacked_piece(&mut new_board, attack_pos);
                self.rook_update_castling_status_after_move(pos, &mut new_board);
                Board::try_add_move(globals, moves, pos, attack_pos, new_board);
            }
        }
    }

    fn rook_update_castling_status_after_move(&self, pos: u8, new_board: &mut Board) {
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
    }

    fn generate_bishop_moves(
        &self,
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        _: BitBoard,
        opponent_occupied_squares: BitBoard,
        occupied_squares: BitBoard,
    ) {
        for pos in BitIter::from(self.player.bishops.board).map(|x| x as u8) {
            let mb = &globals.bishop_magic_bitboards[pos as usize];
            let attack_bitboard = mb.get_attack_bitboard(occupied_squares);

            // Move
            for target_pos in
                BitIter::from(attack_bitboard.board & !occupied_squares.board).map(|x| x as u8)
            {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Bishops, pos, target_pos);
                Board::try_add_move(globals, moves, pos, target_pos, new_board);
            }

            // Attack
            for attack_pos in BitIter::from(attack_bitboard.board & opponent_occupied_squares.board)
                .map(|x| x as u8)
            {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Bishops, pos, attack_pos);
                Board::remove_attacked_piece(&mut new_board, attack_pos);
                Board::try_add_move(globals, moves, pos, attack_pos, new_board);
            }
        }
    }

    fn generate_queen_moves(
        &self,
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        _: BitBoard,
        opponent_occupied_squares: BitBoard,
        occupied_squares: BitBoard,
    ) {
        for pos in BitIter::from(self.player.queens.board).map(|x| x as u8) {
            let mg_cardinals = &globals.rook_magic_bitboards[pos as usize];
            let attack_bitboard_cardinals = mg_cardinals.get_attack_bitboard(occupied_squares);
            let mg_diagonals = &globals.bishop_magic_bitboards[pos as usize];
            let attack_bitboard_diagonals = mg_diagonals.get_attack_bitboard(occupied_squares);
            let attack_bitboard = attack_bitboard_cardinals | attack_bitboard_diagonals;

            // Move
            for target_pos in
                BitIter::from(attack_bitboard.board & !occupied_squares.board).map(|x| x as u8)
            {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Queens, pos, target_pos);
                Board::try_add_move(globals, moves, pos, target_pos, new_board);
            }

            // Attack
            for attack_pos in BitIter::from(attack_bitboard.board & opponent_occupied_squares.board)
                .map(|x| x as u8)
            {
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Queens, pos, attack_pos);
                Board::remove_attacked_piece(&mut new_board, attack_pos);
                Board::try_add_move(globals, moves, pos, attack_pos, new_board);
            }
        }
    }

    fn generate_king_moves(
        &self,
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        _: BitBoard,
        opponent_occupied_squares: BitBoard,
        occupied_squares: BitBoard,
    ) {
        // Kings
        for pos in BitIter::from(self.player.kings.board).map(|x| x as u8) {
            let attack_bitboard = globals.king_bitboards[pos as usize];
            for target_pos in
                BitIter::from(attack_bitboard.board & !occupied_squares.board).map(|x| x as u8)
            {
                // Move
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Kings, pos, target_pos);
                new_board.player.castling_status = CastlingStatus::Unavailable;
                Board::try_add_move(globals, moves, pos, target_pos, new_board);
            }
            for attack_pos in BitIter::from(attack_bitboard.board & opponent_occupied_squares.board)
                .map(|x| x as u8)
            {
                // Attack
                let mut new_board = self.new_move_board();
                new_board.player.move_piece(Piece::Kings, pos, attack_pos);
                Board::remove_attacked_piece(&mut new_board, attack_pos);
                new_board.player.castling_status = CastlingStatus::Unavailable;
                Board::try_add_move(globals, moves, pos, attack_pos, new_board);
            }

            // Castling
            let process_castling = RefCell::new(|pos, pos_adder: i8, rook_pos| {
                let mut current_pos = (pos as i8 + pos_adder) as u8;
                // All squares between the king and rook must be free
                while current_pos != rook_pos {
                    if occupied_squares.get(current_pos) {
                        return;
                    }
                    current_pos = (current_pos as i8 + pos_adder) as u8;
                }
                let mut new_board = self.new_move_board();
                if !self.is_king_in_check(globals) {
                    let neighbor_pos = (pos as i8 + pos_adder) as u8;
                    let target_pos = (pos as i8 + 2 * pos_adder) as u8;
                    new_board.player.move_piece(Piece::Kings, pos, neighbor_pos);
                    if !new_board.is_king_in_check(globals) {
                        new_board
                            .player
                            .move_piece(Piece::Kings, neighbor_pos, target_pos);
                        new_board
                            .player
                            .move_piece(Piece::Rooks, rook_pos, neighbor_pos);
                        new_board.player.castling_status = CastlingStatus::Unavailable;
                        Board::try_add_move(globals, moves, pos, target_pos, new_board);
                    }
                }
            });
            let process_kingside_castling = || {
                (process_castling.borrow_mut())(pos, 1, 7);
            };
            let process_queenside_castling = || {
                (process_castling.borrow_mut())(pos, -1, 0);
            };
            match self.player.castling_status {
                CastlingStatus::BothAvailable => {
                    process_kingside_castling();
                    process_queenside_castling();
                }
                CastlingStatus::KingSideAvailable => {
                    process_kingside_castling();
                }
                CastlingStatus::QueenSideAvailable => {
                    process_queenside_castling();
                }
                _ => (),
            }
        }
    }

    // Check whether a move is valid before adding it
    fn try_add_move(
        globals: &mut Globals,
        moves: &mut Box<Moves>,
        pos: u8,
        target_pos: u8,
        new_board: Board,
    ) {
        if !new_board.is_king_in_check(globals) {
            moves.list[moves.count as usize] = Move {
                from: pos,
                to: target_pos,
                board: new_board.flip_view(),
                evaluation: 0,
            };
            moves.count += 1;
        }
    }
    // Remove attacked piece
    fn remove_attacked_piece(new_board: &mut Board, attack_pos: u8) {
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
        new_board.opponent.remove_any_piece(attack_pos);
    }
}
