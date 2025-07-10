use super::game_state::{GameState, Player};

pub struct Heuristic;

impl Heuristic {
    pub fn evaluate(state: &GameState) -> i32 {
        // Terminal state evaluation: check for wins or draws
        if let Some(winner) = state.winner {
            return match winner {
                Player::Max => 1_000_000,
                Player::Min => -1_000_000,
            };
        }

        if state.get_possible_moves().is_empty() {
            return 0; // Draw
        }

        let mut total_score = 0;
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for i in 0..state.board_size {
            for j in 0..state.board_size {
                if let Some(player) = state.board[i][j] {
                    for &(dx, dy) in directions.iter() {
                        // Check if current cell is start of a run
                        let prev_i = i as isize - dx as isize;
                        let prev_j = j as isize - dy as isize;

                        // Skip if previous cell in run exists
                        if prev_i >= 0
                            && prev_j >= 0
                            && prev_i < state.board_size as isize
                            && prev_j < state.board_size as isize
                        {
                            if let Some(prev_player) = state.board[prev_i as usize][prev_j as usize]
                            {
                                if prev_player == player {
                                    continue;
                                }
                            }
                        }

                        // Count consecutive stones in direction
                        let mut count = 1;
                        let mut cur_i = i as isize + dx as isize;
                        let mut cur_j = j as isize + dy as isize;

                        while cur_i >= 0
                            && cur_j >= 0
                            && cur_i < state.board_size as isize
                            && cur_j < state.board_size as isize
                        {
                            match state.board[cur_i as usize][cur_j as usize] {
                                Some(cell_player) if cell_player == player => {
                                    count += 1;
                                    cur_i += dx as isize;
                                    cur_j += dy as isize;
                                }
                                _ => break,
                            }
                        }

                        // Only consider runs of length 2 or more
                        if count < 2 {
                            continue;
                        }

                        // Skip runs that meet/exceed win condition (should be terminal)
                        if count >= state.win_condition {
                            continue;
                        }

                        // Check start end (behind the run)
                        let start_open = if prev_i >= 0
                            && prev_j >= 0
                            && prev_i < state.board_size as isize
                            && prev_j < state.board_size as isize
                        {
                            state.board[prev_i as usize][prev_j as usize].is_none()
                        } else {
                            false // Blocked by board edge
                        };

                        // Check end end (after the run)
                        let end_open = if cur_i >= 0
                            && cur_j >= 0
                            && cur_i < state.board_size as isize
                            && cur_j < state.board_size as isize
                        {
                            state.board[cur_i as usize][cur_j as usize].is_none()
                        } else {
                            false // Blocked by board edge
                        };

                        // Calculate run score based on type
                        let score = match (start_open, end_open) {
                            (true, true) => 2 * 10i32.pow(count as u32 - 1), // Open run
                            (true, false) | (false, true) => 10i32.pow(count as u32 - 1), // Semi-open
                            _ => 0, // Closed run (no score)
                        };

                        // Add score for Max, subtract for Min
                        match player {
                            Player::Max => total_score += score,
                            Player::Min => total_score -= score,
                        }
                    }
                }
            }
        }

        total_score
    }
}
