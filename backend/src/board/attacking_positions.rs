use crate::{
    Board,
    move_sets::{KNIGHT_TARGET_POS_LISTS_2D, from_pos, to_pos},
};

impl Board {
    pub fn get_attacking_positions<const RETURN_EARLY: bool>(&self, pos: u8) -> (Vec<u8>, bool) {
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
}
