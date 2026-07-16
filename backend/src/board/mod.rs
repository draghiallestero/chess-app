mod attacking_positions;
mod fen;
mod generate_legal_moves;

use super::*;

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

    fn is_king_in_check(&self) -> bool {
        let pos = BitIter::from(self.player.kings.board).next().unwrap() as u8;

        return self.is_position_attacked(pos);
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
}
