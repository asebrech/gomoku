use crate::core::board::{Board, Player};
use crate::core::patterns::{PatternAnalyzer, DIRECTIONS};
use crate::core::rules::GameRules;
use std::collections::HashSet;

pub struct MoveGenerator;

impl MoveGenerator {
    pub fn get_candidate_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        if board.is_empty() {
            return vec![board.center()];
        }

        if let Some(winning_move) = Self::find_winning_move(board, player) {
            return vec![winning_move];
        }

        if let Some(block_moves) = Self::find_must_block_moves(board, player.opponent()) {
            if !block_moves.is_empty() {
                return block_moves;
            }
        }

        let threat_moves = Self::find_threat_moves(board, player);
        if !threat_moves.is_empty() {
            return threat_moves;
        }

        Self::get_zone_based_moves(board, player)
    }

    fn find_winning_move(board: &Board, player: Player) -> Option<(usize, usize)> {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        for array_idx in 0..board.u64_count {
            let mut bits = player_bits[array_idx];
            while bits != 0 {
                let bit_pos = bits.trailing_zeros() as usize;
                let global_idx = array_idx * 64 + bit_pos;
                if global_idx < board.total_cells {
                    let row = global_idx / board.size;
                    let col = global_idx % board.size;

                    for &(dx, dy) in &DIRECTIONS {
                        if let Some(win_pos) = Self::find_win_in_direction(board, row, col, dx, dy, player) {
                            return Some(win_pos);
                        }
                    }
                }
                bits &= bits - 1;
            }
        }
        None
    }

    /// Find a winning move in a specific direction from a given stone.
    /// 
    /// This function handles three cases:
    /// 1. Solid pattern (XXXX_): Check endpoints
    /// 2. Pattern with one gap (XXX_X): Check the gap
    /// 3. Pattern with multiple gaps: Check each gap
    /// 
    /// Returns the position that completes five-in-a-row if found.
    fn find_win_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> Option<(usize, usize)> {
        let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
        let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
        let total = backward + forward + 1;

        // Need at least 4 stones to potentially create 5
        if total < 4 {
            return None;
        }

        // Case 1: Check endpoint positions (handles solid patterns like XXXX_)
        let back_row = row as isize - dx * (backward as isize + 1);
        let back_col = col as isize - dy * (backward as isize + 1);
        if PatternAnalyzer::is_valid_empty(board, back_row, back_col) {
            if Self::creates_five_in_row(board, (back_row as usize, back_col as usize), player) {
                return Some((back_row as usize, back_col as usize));
            }
        }

        let fwd_row = row as isize + dx * (forward as isize + 1);
        let fwd_col = col as isize + dy * (forward as isize + 1);
        if PatternAnalyzer::is_valid_empty(board, fwd_row, fwd_col) {
            if Self::creates_five_in_row(board, (fwd_row as usize, fwd_col as usize), player) {
                return Some((fwd_row as usize, fwd_col as usize));
            }
        }

        // Case 2 & 3: Check for gaps within the pattern (handles patterns like XXX_X or XX_XX)
        // This is necessary because the consecutive count breaks at gaps
        for i in 1..=backward {
            let check_row = row as isize - dx * i as isize;
            let check_col = col as isize - dy * i as isize;
            if PatternAnalyzer::is_valid_empty(board, check_row, check_col) {
                let pos = (check_row as usize, check_col as usize);
                if Self::creates_five_in_row(board, pos, player) {
                    return Some(pos);
                }
            }
        }

        for i in 1..=forward {
            let check_row = row as isize + dx * i as isize;
            let check_col = col as isize + dy * i as isize;
            if PatternAnalyzer::is_valid_empty(board, check_row, check_col) {
                let pos = (check_row as usize, check_col as usize);
                if Self::creates_five_in_row(board, pos, player) {
                    return Some(pos);
                }
            }
        }

        None
    }

    fn creates_five_in_row(board: &Board, pos: (usize, usize), player: Player) -> bool {
        for &(dx, dy) in &DIRECTIONS {
            // Count stones in both directions from this empty position
            // Note: pos is empty, so we count around it, not including it
            let backward = PatternAnalyzer::count_consecutive(board, pos.0, pos.1, -dx, -dy, player);
            let forward = PatternAnalyzer::count_consecutive(board, pos.0, pos.1, dx, dy, player);
            // Add 1 for the stone we would place at pos
            let total = backward + forward + 1;
            if total >= 5 {
                return true;
            }
        }
        false
    }

    fn find_must_block_moves(board: &Board, opponent: Player) -> Option<Vec<(usize, usize)>> {
        if let Some(opp_win) = Self::find_winning_move(board, opponent) {
            return Some(vec![opp_win]);
        }

        let open_fours = Self::find_open_four_threats(board, opponent);
        if !open_fours.is_empty() {
            return Some(open_fours);
        }

        None
    }

    fn find_open_four_threats(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        for array_idx in 0..board.u64_count {
            let mut bits = player_bits[array_idx];
            while bits != 0 {
                let bit_pos = bits.trailing_zeros() as usize;
                let global_idx = array_idx * 64 + bit_pos;
                if global_idx < board.total_cells {
                    let row = global_idx / board.size;
                    let col = global_idx % board.size;

                    for &(dx, dy) in &DIRECTIONS {
                        let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
                        let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
                        
                        if backward + forward + 1 == 4 {
                            let back_row = row as isize - dx * (backward as isize + 1);
                            let back_col = col as isize - dy * (backward as isize + 1);
                            let fwd_row = row as isize + dx * (forward as isize + 1);
                            let fwd_col = col as isize + dy * (forward as isize + 1);

                            if PatternAnalyzer::is_valid_empty(board, back_row, back_col) {
                                threats.insert((back_row as usize, back_col as usize));
                            }
                            if PatternAnalyzer::is_valid_empty(board, fwd_row, fwd_col) {
                                threats.insert((fwd_row as usize, fwd_col as usize));
                            }
                        }
                    }
                }
                bits &= bits - 1;
            }
        }

        threats.into_iter().collect()
    }

    fn find_threat_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut moves = HashSet::new();

        let our_threats = Self::find_threat_creating_moves(board, player);
        moves.extend(our_threats);

        let opp_threats = Self::find_threat_creating_moves(board, player.opponent());
        moves.extend(opp_threats);

        let filtered_moves: Vec<(usize, usize)> = moves
            .into_iter()
            .filter(|&(row, col)| !GameRules::creates_double_three(board, row, col, player))
            .collect();

        // If too many threat moves, prioritize and limit them instead of abandoning
        if filtered_moves.len() > 30 {
            let mut prioritized_moves: Vec<((usize, usize), i32)> = filtered_moves
                .into_iter()
                .map(|mv| {
                    let priority = Self::calculate_threat_priority(board, mv, player);
                    (mv, priority)
                })
                .collect();
            
            // Sort by priority (descending)
            prioritized_moves.sort_by_key(|(_, priority)| -priority);
            
            // Take top 30 moves
            prioritized_moves.truncate(30);
            prioritized_moves.into_iter().map(|(mv, _)| mv).collect()
        } else {
            filtered_moves
        }
    }

    /// Calculate the tactical priority of a threat move
    fn calculate_threat_priority(board: &Board, mv: (usize, usize), player: Player) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        // Check both our threats and opponent's threats at this position
        for &check_player in &[player, player.opponent()] {
            for &(dx, dy) in &DIRECTIONS {
                let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, check_player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, check_player);
                let total = backward + forward + 1;

                // Prioritize based on pattern strength
                let pattern_value = match total {
                    5 => 10000,  // Creates five - winning move
                    4 => {
                        // Check if it's open four
                        let back_row = row as isize - dx * (backward as isize + 1);
                        let back_col = col as isize - dy * (backward as isize + 1);
                        let fwd_row = row as isize + dx * (forward as isize + 1);
                        let fwd_col = col as isize + dy * (forward as isize + 1);
                        
                        let back_open = PatternAnalyzer::is_valid_empty(board, back_row, back_col);
                        let fwd_open = PatternAnalyzer::is_valid_empty(board, fwd_row, fwd_col);
                        
                        if back_open && fwd_open {
                            1000  // Open four - very strong
                        } else {
                            500   // Half-open four
                        }
                    }
                    3 => 200,  // Three in a row
                    2 => 50,   // Two in a row
                    _ => 0,
                };

                // Double the value if it's our threat (offensive), keep normal if opponent's (defensive)
                if check_player == player {
                    priority += pattern_value * 2;
                } else {
                    priority += pattern_value;
                }
            }
        }

        // Bonus for center proximity
        let center = board.size / 2;
        let distance = ((row as isize - center as isize).abs() + (col as isize - center as isize).abs()) as i32;
        priority += 10 - distance.min(10);

        priority
    }

    fn find_threat_creating_moves(board: &Board, player: Player) -> HashSet<(usize, usize)> {
        let mut moves = HashSet::new();
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        for array_idx in 0..board.u64_count {
            let mut bits = player_bits[array_idx];
            while bits != 0 {
                let bit_pos = bits.trailing_zeros() as usize;
                let global_idx = array_idx * 64 + bit_pos;
                if global_idx < board.total_cells {
                    let row = global_idx / board.size;
                    let col = global_idx % board.size;

                    for &(dx, dy) in &DIRECTIONS {
                        let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
                        let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
                        let total = backward + forward + 1;

                        if total >= 2 && total <= 4 {
                            for offset in -(backward as isize + 1)..=(forward as isize + 1) {
                                let r = row as isize + dx * offset;
                                let c = col as isize + dy * offset;
                                if PatternAnalyzer::is_valid_empty(board, r, c) {
                                    moves.insert((r as usize, c as usize));
                                }
                            }
                        }
                    }
                }
                bits &= bits - 1;
            }
        }

        moves
    }

    fn get_zone_based_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut candidates = HashSet::new();
        let stone_count = board.count_stones();
        
        let zone_radius = if stone_count < 10 { 2 } else { 1 };

        for array_idx in 0..board.u64_count {
            let mut bits = board.occupied[array_idx];
            while bits != 0 {
                let bit_pos = bits.trailing_zeros() as usize;
                let global_idx = array_idx * 64 + bit_pos;
                if global_idx < board.total_cells {
                    let row = global_idx / board.size;
                    let col = global_idx % board.size;

                    for dr in -(zone_radius as isize)..=(zone_radius as isize) {
                        for dc in -(zone_radius as isize)..=(zone_radius as isize) {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            let nr = row as isize + dr;
                            let nc = col as isize + dc;
                            
                            if PatternAnalyzer::is_valid_empty(board, nr, nc) {
                                candidates.insert((nr as usize, nc as usize));
                            }
                        }
                    }
                }
                bits &= bits - 1;
            }
        }

        candidates
            .into_iter()
            .filter(|&(row, col)| !GameRules::creates_double_three(board, row, col, player))
            .collect()
    }
}
