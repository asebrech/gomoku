use crate::core::state::GameState;
use crate::core::board::Player;

pub struct Heuristic;

impl Heuristic {
    pub fn evaluate(state: &GameState) -> i32 {
        if let Some(winner) = state.winner {
            return match winner {
                Player::Max => 1_000_000,
                Player::Min => -1_000_000,
            };
        }

        if let Some(winner) = state.check_capture_win() {
            return match winner {
                Player::Max => 1_000_000,
                Player::Min => -1_000_000,
            };
        }

        if state.get_possible_moves().is_empty() {
            return 0;
        }

        let mut total_score = 0;
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        let capture_score = (state.max_captures as i32 - state.min_captures as i32) * 1000;
        total_score += capture_score;

        for i in 0..state.board.size {
            for j in 0..state.board.size {
                if let Some(player) = state.board.get_player(i, j) {
                    for &(dx, dy) in directions.iter() {
                        let prev_i = i as isize - dx as isize;
                        let prev_j = j as isize - dy as isize;

                        if prev_i >= 0
                            && prev_j >= 0
                            && prev_i < state.board.size as isize
                            && prev_j < state.board.size as isize
                        {
                            if let Some(prev_player) = state.board.get_player(prev_i as usize, prev_j as usize)
                            {
                                if prev_player == player {
                                    continue;
                                }
                            }
                        }

                        let mut count = 1;
                        let mut cur_i = i as isize + dx as isize;
                        let mut cur_j = j as isize + dy as isize;

                        while cur_i >= 0
                            && cur_j >= 0
                            && cur_i < state.board.size as isize
                            && cur_j < state.board.size as isize
                        {
                            match state.board.get_player(cur_i as usize, cur_j as usize) {
                                Some(cell_player) if cell_player == player => {
                                    count += 1;
                                    cur_i += dx as isize;
                                    cur_j += dy as isize;
                                }
                                _ => break,
                            }
                        }

                        if count < 2 {
                            continue;
                        }

                        if count >= state.win_condition {
                            continue;
                        }

                        let start_open = if prev_i >= 0
                            && prev_j >= 0
                            && prev_i < state.board.size as isize
                            && prev_j < state.board.size as isize
                        {
                            state.board.get_player(prev_i as usize, prev_j as usize).is_none()
                        } else {
                            false
                        };

                        let end_open = if cur_i >= 0
                            && cur_j >= 0
                            && cur_i < state.board.size as isize
                            && cur_j < state.board.size as isize
                        {
                            state.board.get_player(cur_i as usize, cur_j as usize).is_none()
                        } else {
                            false
                        };

                        let score = match (start_open, end_open) {
                            (true, true) => 2 * 10i32.pow(count as u32 - 1),
                            (true, false) | (false, true) => 10i32.pow(count as u32 - 1),
                            _ => 0,
                        };

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
