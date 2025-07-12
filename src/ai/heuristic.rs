use crate::core::board::Player;
use crate::core::state::GameState;

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
                            if let Some(prev_player) =
                                state.board.get_player(prev_i as usize, prev_j as usize)
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
                            state
                                .board
                                .get_player(prev_i as usize, prev_j as usize)
                                .is_none()
                        } else {
                            false
                        };

                        let end_open = if cur_i >= 0
                            && cur_j >= 0
                            && cur_i < state.board.size as isize
                            && cur_j < state.board.size as isize
                        {
                            state
                                .board
                                .get_player(cur_i as usize, cur_j as usize)
                                .is_none()
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

    pub fn order_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
        let center = state.board.center();
        let board_size = state.board.size;
        let my_player = state.current_player;
        let opp_player = my_player.opponent();
        let win_condition = state.win_condition;
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        fn manhattan(a: (usize, usize), b: (usize, usize)) -> usize {
            (a.0 as isize - b.0 as isize).abs() as usize
                + (a.1 as isize - b.1 as isize).abs() as usize
        }

        // For each candidate move, assign a score
        let mut scored_moves: Vec<((usize, usize), i32)> = moves
            .iter()
            .map(|&mv| {
                let mut score = 0i32;
                // Center bonus: closer to center, higher score
                let dist = manhattan(mv, center);
                score += 100 - (dist as i32);
                // Adjacency: more adjacent stones, higher score
                let mut adj_my = 0;
                let mut adj_opp = 0;
                for dr in -1..=1 {
                    for dc in -1..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let newr = mv.0 as isize + dr;
                        let newc = mv.1 as isize + dc;
                        if newr >= 0
                            && newc >= 0
                            && newr < board_size as isize
                            && newc < board_size as isize
                        {
                            match state.board.get_player(newr as usize, newc as usize) {
                                Some(p) if p == my_player => adj_my += 1,
                                Some(p) if p == opp_player => adj_opp += 1,
                                _ => {}
                            }
                        }
                    }
                }
                score += adj_my * 20;
                score += adj_opp * 8;
                // Check if move creates a capture
                let captures = crate::core::captures::CaptureHandler::detect_captures(
                    &state.board,
                    mv.0,
                    mv.1,
                    my_player,
                );
                if !captures.is_empty() {
                    score += 1000 + (captures.len() as i32) * 20;
                }
                // Check if move blocks opponent's immediate win or creates own win
                for &(dx, dy) in &directions {
                    for &player in &[my_player, opp_player] {
                        let mut count = 1;
                        let mut blocked = false;
                        for dir in &[-1, 1] {
                            let (mut x, mut y) = (mv.0 as isize, mv.1 as isize);
                            loop {
                                x += dx as isize * dir;
                                y += dy as isize * dir;
                                if x < 0
                                    || y < 0
                                    || x >= board_size as isize
                                    || y >= board_size as isize
                                {
                                    break;
                                }
                                match state.board.get_player(x as usize, y as usize) {
                                    Some(p) if p == player => count += 1,
                                    Some(_) => {
                                        blocked = true;
                                        break;
                                    }
                                    None => break,
                                }
                            }
                        }
                        if count >= win_condition {
                            // Win for self or block opponent win
                            if player == my_player {
                                score += 100_000;
                            } else {
                                score += 80_000;
                            }
                        } else if count == win_condition - 1 {
                            // Threats (lines of length 4)
                            if player == my_player {
                                score += 2000;
                            } else {
                                score += 1500;
                            }
                        }
                    }
                }
                ((mv.0, mv.1), score)
            })
            .collect();

        // Sort by descending score
        scored_moves.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        // Rewrite moves in new order
        *moves = scored_moves.into_iter().map(|(mv, _)| mv).collect();
    }
}
