use crate::core::board::{Board, Player};
use crate::core::state::GameState;

pub struct Heuristic;

impl Heuristic {
    pub fn evaluate(state: &GameState) -> i32 {
        // Check for terminal states first
        if let Some(winner) = state.check_winner() {
            return match winner {
                Player::Max => 1000000,
                Player::Min => -1000000,
            };
        }

        // Check for capture win conditions
        if state.max_captures >= 5 {
            return 1_000_000;
        }

        if state.min_captures >= 5 {
            return -1_000_000;
        }

        // If the board is full and no winner, it's a draw
        if state.board.cells.iter().all(|row| row.iter().all(|&cell| cell.is_some())) {
            return 0;
        }

        let max_score = Self::evaluate_player(&state.board, Player::Max, state.win_condition);
        let min_score = Self::evaluate_player(&state.board, Player::Min, state.win_condition);

        // Add capture bonus
        let capture_bonus = (state.max_captures as i32 * 200) - (state.min_captures as i32 * 200);

        max_score - min_score //+ capture_bonus
    }

    fn evaluate_player(board: &Board, player: Player, win_condition: usize) -> i32 {
        let mut score = 0;
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        // Count patterns for this player
        let mut five_in_row = 0;
        let mut live_four = 0;
        let mut live_three = 0;
        let mut dead_four = 0;
        let mut dead_three = 0;
        let mut live_two = 0;

        for row in 0..board.size {
            for col in 0..board.size {
                if board.get_player(row, col) == Some(player) {
                    for &(dx, dy) in &directions {
                        let pattern =
                            Self::analyze_line(board, row, col, dx, dy, player, win_condition);

                        match pattern {
                            5 => five_in_row += 1,
                            4 => {
                                if Self::is_live_pattern(board, row, col, dx, dy, player, 4) {
                                    live_four += 1;
                                } else {
                                    dead_four += 1;
                                }
                            }
                            3 => {
                                if Self::is_live_pattern(board, row, col, dx, dy, player, 3) {
                                    live_three += 1;
                                } else {
                                    dead_three += 1;
                                }
                            }
                            2 => {
                                if Self::is_live_pattern(board, row, col, dx, dy, player, 2) {
                                    live_two += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Avoid double counting by dividing by pattern length
        five_in_row /= 5;
        live_four /= 4;
        dead_four /= 4;
        live_three /= 3;
        dead_three /= 3;
        live_two /= 2;

        // Apply scoring rules from pseudocode
        if five_in_row > 0 {
            score += 100000;
        }

        if live_four == 1 {
            score += 15000;
        } else if live_four > 1 {
            score += 20000; // Multiple live fours is even better
        }

        if live_three >= 2 || dead_four >= 2 || (dead_four >= 1 && live_three >= 1) {
            score += 10000;
        }

        if dead_four > 0 {
            score += 1000 * dead_four;
        }

        // Additional scoring for other patterns
        score += live_three * 500;
        score += dead_three * 100;
        score += live_two * 50;

        score
    }

    fn analyze_line(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        win_condition: usize,
    ) -> usize {
        let mut count = 0;
        let mut r = row as isize;
        let mut c = col as isize;

        // Count consecutive pieces in positive direction
        while r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            if board.get_player(r as usize, c as usize) == Some(player) {
                count += 1;
                r += dx;
                c += dy;
            } else {
                break;
            }
        }

        // Count consecutive pieces in negative direction
        r = row as isize - dx;
        c = col as isize - dy;
        while r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            if board.get_player(r as usize, c as usize) == Some(player) {
                count += 1;
                r -= dx;
                c -= dy;
            } else {
                break;
            }
        }

        count.min(win_condition)
    }

    fn is_live_pattern(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
        pattern_length: usize,
    ) -> bool {
        // Check if pattern has open ends
        let mut start_r = row as isize;
        let mut start_c = col as isize;

        // Find start of pattern
        while start_r >= 0
            && start_r < board.size as isize
            && start_c >= 0
            && start_c < board.size as isize
            && board.get_player(start_r as usize, start_c as usize) == Some(player)
        {
            start_r -= dx;
            start_c -= dy;
        }

        // Find end of pattern
        let mut end_r = row as isize;
        let mut end_c = col as isize;
        while end_r >= 0
            && end_r < board.size as isize
            && end_c >= 0
            && end_c < board.size as isize
            && board.get_player(end_r as usize, end_c as usize) == Some(player)
        {
            end_r += dx;
            end_c += dy;
        }

        // Check if both ends are empty (live pattern)
        let start_empty = start_r >= 0
            && start_r < board.size as isize
            && start_c >= 0
            && start_c < board.size as isize
            && board
                .get_player(start_r as usize, start_c as usize)
                .is_none();

        let end_empty = end_r >= 0
            && end_r < board.size as isize
            && end_c >= 0
            && end_c < board.size as isize
            && board.get_player(end_r as usize, end_c as usize).is_none();

        start_empty && end_empty
    }

    pub fn order_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
        // Simple move ordering: prioritize center moves and moves near existing pieces
        let center = state.board.size / 2;

        moves.sort_by(|&a, &b| {
            let dist_a = Self::move_priority(state, a, center);
            let dist_b = Self::move_priority(state, b, center);
            dist_b.cmp(&dist_a) // Higher priority first
        });
    }

    fn move_priority(state: &GameState, mv: (usize, usize), center: usize) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        // Prefer moves closer to center
        let center_dist = ((row as isize - center as isize).abs()
            + (col as isize - center as isize).abs()) as i32;
        priority += 100 - center_dist;

        // Prefer moves near existing pieces
        let directions = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        for &(dx, dy) in &directions {
            let new_row = row as isize + dx;
            let new_col = col as isize + dy;

            if new_row >= 0
                && new_row < state.board.size as isize
                && new_col >= 0
                && new_col < state.board.size as isize
            {
                if state
                    .board
                    .get_player(new_row as usize, new_col as usize)
                    .is_some()
                {
                    priority += 50;
                }
            }
        }

        priority
    }
}
