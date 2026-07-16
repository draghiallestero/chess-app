pub mod _move;
pub mod bitboard;
mod board;
mod en_passant;
pub mod move_sets;
mod piece;
mod piece_square_tables;
pub use _move::Move;
pub use bitboard::BitBoard;
pub use board::Board;
use piece_square_tables::MIDGAME_PAWNS;
use std::cmp::max;

use bit_iter::BitIter;

use crate::{
    en_passant::EnPassant,
    move_sets::{from_chars, to_chars},
    piece::Piece,
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
