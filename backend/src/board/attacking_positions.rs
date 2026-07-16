use crate::*;

impl Board {
    pub fn is_position_attacked(&self, globals: &Globals, pos: u8) -> bool {
        let (rank, file) = from_pos(pos);

        let player_occupied_squares = self.player.occupied_squares();
        let opponent_occupied_squares = self.opponent.occupied_squares();
        let occupied_squares = player_occupied_squares | opponent_occupied_squares;

        // Pawns
        if rank < 7 {
            if file > 0 {
                let current_pos = to_pos(rank + 1, file - 1);
                if self.opponent.pawns.get(current_pos) {
                    return true;
                }
            }
            if file < 7 {
                let current_pos = to_pos(rank + 1, file + 1);
                if self.opponent.pawns.get(current_pos) {
                    return true;
                }
            }
        }

        // Rooks, bishops, queens
        let mg_cardinals = &globals.rook_magic_bitboards[pos as usize];
        let attack_bitboard_cardinals = mg_cardinals.get_attack_bitboard(occupied_squares);
        let mg_diagonals = &globals.bishop_magic_bitboards[pos as usize];
        let attack_bitboard_diagonals = mg_diagonals.get_attack_bitboard(occupied_squares);
        if (attack_bitboard_cardinals & self.opponent.rooks).board != 0
            || (attack_bitboard_cardinals & self.opponent.queens).board != 0
        {
            return true;
        }
        if (attack_bitboard_diagonals & self.opponent.bishops).board != 0
            || (attack_bitboard_diagonals & self.opponent.queens).board != 0
        {
            return true;
        }

        // Knights
        if (globals.knight_bitboards[pos as usize] & self.opponent.knights).board != 0 {
            return true;
        }

        false
    }
}
