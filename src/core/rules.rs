//! Game rules and validation utilities.
//!
//! Contains functions to detect wins around a move, capture-win conditions
//! and forbidden patterns such as double-three. The implementation focuses
//! on correctness and readability.
//!
use crate::core::board::{Board, Player};
use crate::core::patterns::{PatternAnalyzer, DIRECTIONS};

const FREE_THREE_LENGTH: usize = 3;
const MAX_SEARCH_DISTANCE: isize = 4;

pub struct GameRules;

impl GameRules {
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize) -> bool {
        if row >= board.size || col >= board.size {
            return false;
        }

        let idx = board.index(row, col);
        if !Board::is_bit_set(&board.occupied, idx) {
            return false;
        }

        let player = if Board::is_bit_set(&board.max_bits, idx) {
            Player::Max
        } else {
            Player::Min
        };

        for &(dx, dy) in &DIRECTIONS {
            let mut count = 1;
            count += PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
            count += PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);

            if count >= win_condition {
                return true;
            }
        }

        false
    }

    pub fn check_win_and_breakable(board: &Board, row: usize, col: usize, win_condition: usize) -> (bool, bool) {
        if row >= board.size || col >= board.size {
            return (false, false);
        }

        let idx = board.index(row, col);
        if !Board::is_bit_set(&board.occupied, idx) {
            return (false, false);
        }

        let player = if Board::is_bit_set(&board.max_bits, idx) {
            Player::Max
        } else {
            Player::Min
        };

        for &(dx, dy) in &DIRECTIONS {
            let mut count = 1;
            count += PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
            count += PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);

            if count >= win_condition {
                let is_breakable = Self::can_break_five_by_capture(board, row, col, player);
                return (true, is_breakable);
            }
        }

        (false, false)
    }

    pub fn check_capture_win(max_captures: usize, min_captures: usize, capture_to_win: usize) -> Option<Player> {
        if max_captures >= capture_to_win {
            Some(Player::Max)
        } else if min_captures >= capture_to_win {
            Some(Player::Min)
        } else {
            None
        }
    }

    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
        DIRECTIONS
            .iter()
            .filter(|&&dir| Self::is_free_three_in_direction(board, row, col, player, dir))
            .count()
            >= 2
    }

    fn is_free_three_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        (dr, dc): (isize, isize),
    ) -> bool {
        let (stones, left_open, right_open) = Self::analyze_line(board, row, col, player, dr, dc);
        stones == FREE_THREE_LENGTH && Self::can_form_open_four(left_open, right_open)
    }

    fn analyze_line(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
    ) -> (usize, bool, bool) {
        let left_info = Self::scan_direction(board, row, col, player, -dr, -dc);
        let right_info = Self::scan_direction(board, row, col, player, dr, dc);

        let total_stones = 1 + left_info.0 + right_info.0;
        let left_open = left_info.1;
        let right_open = right_info.1;

        (total_stones, left_open, right_open)
    }

    fn scan_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
    ) -> (usize, bool) {
        let player_bits = board.get_player_bits(player);
        let opponent_bits = board.get_player_bits(player.opponent());

        let mut stones = 0;
        let mut empty_found = false;
        let mut is_open = false;

        for i in 1..=MAX_SEARCH_DISTANCE {
            let new_row = row as isize + dr * i;
            let new_col = col as isize + dc * i;

            if !PatternAnalyzer::is_in_bounds(board, new_row, new_col) {
                break;
            }
            let idx = board.index(new_row as usize, new_col as usize);

            if Board::is_bit_set(player_bits, idx) {
                if empty_found {
                    break;
                }
                stones += 1;
            } else if !Board::is_bit_set(&board.occupied, idx) {
                if !empty_found && stones > 0 {
                    is_open = true;
                }
                empty_found = true;
                if stones > 0 {
                    break;
                }
            } else if Board::is_bit_set(opponent_bits, idx) {
                break;
            }
        }

        (stones, is_open)
    }

    #[inline]
    fn can_form_open_four(left_open: bool, right_open: bool) -> bool {
        left_open || right_open
    }

    pub fn can_break_five_by_capture(board: &Board, row: usize, col: usize, player: Player) -> bool {
        let opponent = player.opponent();
        for &(dx, dy) in &DIRECTIONS {
            let mut count = 1;
            count += PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
            count += PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);

            if count >= 5 {
                let mut line_positions = vec![(row, col)];
                for i in 1..=4 {
                    let r = row as isize - dx * i;
                    let c = col as isize - dy * i;
                    if PatternAnalyzer::is_in_bounds(board, r, c) {
                        let idx = board.index(r as usize, c as usize);
                        let player_bits = board.get_player_bits(player);
                        if Board::is_bit_set(player_bits, idx) {
                            line_positions.push((r as usize, c as usize));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                for i in 1..=4 {
                    let r = row as isize + dx * i;
                    let c = col as isize + dy * i;
                    if PatternAnalyzer::is_in_bounds(board, r, c) {
                        let idx = board.index(r as usize, c as usize);
                        let player_bits = board.get_player_bits(player);
                        if Board::is_bit_set(player_bits, idx) {
                            line_positions.push((r as usize, c as usize));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                for &(stone_row, stone_col) in &line_positions {
                    if Self::can_opponent_capture_stone(board, stone_row, stone_col, opponent) {
                        return true;
                    }
                }
            }
        }
        
        false
    }

    pub fn get_breaking_capture_moves(board: &Board, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
        let opponent = player.opponent();
        let mut breaking_moves = Vec::new();
        
        for &(dx, dy) in &DIRECTIONS {
            let mut count = 1;
            count += PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
            count += PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);

            if count >= 5 {
                let mut line_positions = vec![(row, col)];
                
                for i in 1..=4 {
                    let r = row as isize - dx * i;
                    let c = col as isize - dy * i;
                    if PatternAnalyzer::is_in_bounds(board, r, c) {
                        let idx = board.index(r as usize, c as usize);
                        let player_bits = board.get_player_bits(player);
                        if Board::is_bit_set(player_bits, idx) {
                            line_positions.push((r as usize, c as usize));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                for i in 1..=4 {
                    let r = row as isize + dx * i;
                    let c = col as isize + dy * i;
                    if PatternAnalyzer::is_in_bounds(board, r, c) {
                        let idx = board.index(r as usize, c as usize);
                        let player_bits = board.get_player_bits(player);
                        if Board::is_bit_set(player_bits, idx) {
                            line_positions.push((r as usize, c as usize));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                for &(stone_row, stone_col) in &line_positions {
                    let capture_moves = Self::get_moves_that_capture_stone(board, stone_row, stone_col, opponent);
                    breaking_moves.extend(capture_moves);
                }
            }
        }
        
        breaking_moves.sort();
        breaking_moves.dedup();
        breaking_moves
    }

    fn can_opponent_capture_stone(board: &Board, row: usize, col: usize, opponent: Player) -> bool {
        for &(dx, dy) in &DIRECTIONS {
            for &multiplier in &[1, -1] {
                let actual_dx = dx * multiplier;
                let actual_dy = dy * multiplier;
                
                let cap_row = row as isize - actual_dx;
                let cap_col = col as isize - actual_dy;
                
                if !PatternAnalyzer::is_in_bounds(board, cap_row, cap_col) {
                    continue;
                }
                
                let cap_idx = board.index(cap_row as usize, cap_col as usize);
                
                let opponent_bits = board.get_player_bits(opponent);
                if !Board::is_bit_set(opponent_bits, cap_idx) {
                    continue;
                }
                
                let next_row = row as isize + actual_dx;
                let next_col = col as isize + actual_dy;
                
                if !PatternAnalyzer::is_in_bounds(board, next_row, next_col) {
                    continue;
                }
                
                let next_idx = board.index(next_row as usize, next_col as usize);
                let player_bits = board.get_player_bits(opponent.opponent());
                
                if !Board::is_bit_set(player_bits, next_idx) {
                    continue;
                }
                
                let place_row = next_row + actual_dx;
                let place_col = next_col + actual_dy;
                
                if PatternAnalyzer::is_valid_empty(board, place_row, place_col) {
                    return true;
                }
            }
        }
        
        false
    }

    fn get_moves_that_capture_stone(board: &Board, row: usize, col: usize, opponent: Player) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        
        for &(dx, dy) in &DIRECTIONS {
            for &multiplier in &[1, -1] {
                let actual_dx = dx * multiplier;
                let actual_dy = dy * multiplier;
                
                let next_row = row as isize + actual_dx;
                let next_col = col as isize + actual_dy;
                
                if !PatternAnalyzer::is_in_bounds(board, next_row, next_col) {
                    continue;
                }
                
                let next_idx = board.index(next_row as usize, next_col as usize);
                let player_bits = board.get_player_bits(opponent.opponent());
                
                if !Board::is_bit_set(player_bits, next_idx) {
                    continue;
                }
                
                let place_row = next_row + actual_dx;
                let place_col = next_col + actual_dy;
                
                if !PatternAnalyzer::is_valid_empty(board, place_row, place_col) {
                    continue;
                }
                
                let opp_row = row as isize - actual_dx;
                let opp_col = col as isize - actual_dy;
                
                if !PatternAnalyzer::is_in_bounds(board, opp_row, opp_col) {
                    continue;
                }
                
                let opp_idx = board.index(opp_row as usize, opp_col as usize);
                let opponent_bits = board.get_player_bits(opponent);
                
                if Board::is_bit_set(opponent_bits, opp_idx) {
                    moves.push((place_row as usize, place_col as usize));
                }
            }
        }
        
        moves
    }
}
