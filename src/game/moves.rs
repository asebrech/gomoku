use crate::game::board::{Board, Player};

pub struct MoveHandler;

impl MoveHandler {
    pub fn get_possible_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        
        for i in 0..board.size {
            for j in 0..board.size {
                if board.is_empty_position(i, j) {
                    if board.is_empty() {
                        let center = board.center();
                        if (i, j) == center {
                            moves.push((i, j));
                        }
                    } else if board.is_adjacent_to_stone(i, j) {
                        if !RuleValidator::creates_double_three(board, i, j, player) {
                            moves.push((i, j));
                        }
                    }
                }
            }
        }
        
        moves
    }

}

pub struct RuleValidator;

impl RuleValidator {
    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let mut free_three_count = 0;

        for &direction in &directions {
            if Self::is_free_three(board, row, col, player, direction) {
                free_three_count += 1;
                if free_three_count >= 2 {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_free_three(board: &Board, row: usize, col: usize, player: Player, direction: (isize, isize)) -> bool {
        let (dx, dy) = direction;

        let mut line = Vec::new();
        for i in -3..=3 {
            let new_row = row as isize + i * dx;
            let new_col = col as isize + i * dy;

            if new_row >= 0 && new_row < board.size as isize && 
               new_col >= 0 && new_col < board.size as isize {
                if new_row as usize == row && new_col as usize == col {
                    line.push(Some(player));
                } else {
                    line.push(board.get_player(new_row as usize, new_col as usize));
                }
            } else {
                line.push(Some(Player::Max));
            }
        }

        Self::contains_free_three_pattern(&line, player)
    }

    fn contains_free_three_pattern(line: &[Option<Player>], player: Player) -> bool {
        if line.len() < 6 {
            return false;
        }

        for start in 0..=(line.len() - 6) {
            let segment = &line[start..start + 6];
            if Self::can_form_open_four(segment, player) {
                return true;
            }
        }

        false
    }

    fn can_form_open_four(segment: &[Option<Player>], player: Player) -> bool {
        if segment.len() != 6 {
            return false;
        }

        let mut player_count = 0;

        if segment[0].is_some() || segment[5].is_some() {
            return false;
        }

        for i in 1..5 {
            match segment[i] {
                Some(p) if p == player => {
                    player_count += 1;
                }
                Some(_) => return false,
                None => {}
            }
        }

        if player_count == 3 {
            let empty_positions: Vec<usize> = (1..5).filter(|&i| segment[i].is_none()).collect();

            if empty_positions.len() == 1 {
                return true;
            }
        }

        false
    }
}

