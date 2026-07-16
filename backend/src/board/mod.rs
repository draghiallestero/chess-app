mod attacking_positions;
mod fen;
mod generate_legal_moves;

use crate::*;

use bit_iter::BitIter;

use std::cmp::max;

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

    fn is_king_in_check(&self, globals: &Globals) -> bool {
        let pos = BitIter::from(self.player.kings.board).next().unwrap() as u8;

        return self.is_position_attacked(globals, pos);
    }

    pub fn evaluate_position(&self) -> i16 {
        let player_score = self.player.evaluate_position();
        let oppponent_score = self.opponent.evaluate_position();
        player_score - oppponent_score
    }

    pub fn search_for_best_move(&self, globals: &mut Globals, max_depth: usize) -> Option<Move> {
        let moves = self.generate_legal_moves(globals);
        if moves.is_empty() {
            globals.moves_vec.push_moves(moves);
            return None;
        }
        let mut best_move = Some(moves.list[0]);

        let mut alpha = i16::MIN + 1;
        let beta = i16::MAX - 1;
        for _move in moves.iter() {
            let alpha_candidate = -_move
                .board
                .search_for_best_move_impl(globals, 1, max_depth, -beta, -alpha);
            if alpha_candidate > alpha {
                alpha = alpha_candidate;
                best_move = Some(*_move);
            }
        }

        globals.moves_vec.push_moves(moves);

        best_move
    }

    pub fn search_for_best_move_impl(
        &self,
        globals: &mut Globals,
        depth: usize,
        max_depth: usize,
        alpha: i16,
        beta: i16,
    ) -> i16 {
        if depth == max_depth {
            return self.evaluate_position();
        }

        let moves = self.generate_legal_moves(globals);
        if moves.is_empty() {
            globals.moves_vec.push_moves(moves);
            return self.evaluate_position();
        }

        let mut alpha = alpha;
        for _move in moves.iter() {
            let alpha_candidate = -_move.board.search_for_best_move_impl(
                globals,
                depth + 1,
                max_depth,
                -beta,
                -alpha,
            );
            alpha = max(alpha, alpha_candidate);
        }

        globals.moves_vec.push_moves(moves);

        alpha
    }
}
