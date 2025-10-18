//! Candidate move generation heuristics.
//!
//! Move generation is a key performance lever in board-game AI. This module
//! implements a prioritized candidate move generator for Gomoku. Instead of
//! returning every empty square, it prefers:
//! 1. Immediate winning moves.
//! 2. Forced blocks / must-block moves.
//! 3. Threat moves (moves that create threats or block opponent threats).
//! 4. A local zone around existing stones for general play.
//!
//! This reduces branching factor and lets the search focus on promising
//! continuations. The generator also filters moves that create illegal
//! double-threes according to usual Gomoku rules.

use crate::core::board::{Board, Player};
use crate::core::patterns::{DIRECTIONS, PatternAnalyzer};
use crate::core::rules::DoubleThreeDetection;
use crate::ai::pattern_utils;
use crate::ai::heuristic::CAPTURE_BONUS_MULTIPLIER;
use std::collections::HashSet;

/// Maximum bonus points for center position in move generation.
/// Decreases linearly with distance from center (max 10 points for center, 0 for edges).
const CENTER_POSITION_BONUS: i32 = 10;

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
        let player_bits = board.get_player_bits(player);
        let mut result = None;

        board.iterate_bits(player_bits, |row, col| {
            if result.is_some() {
                return;
            }
            for &(dx, dy) in &DIRECTIONS {
                if let Some(win_pos) = Self::find_win_in_direction(board, row, col, dx, dy, player)
                {
                    result = Some(win_pos);
                    return;
                }
            }
        });

        result
    }

    fn find_win_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> Option<(usize, usize)> {
        const PRIORITY_OFFSETS: &[isize] = &[-1, 1, -2, 2, -3, 3, -4, 4, -5, 5];

        for &offset in PRIORITY_OFFSETS {
            let check_row = row as isize + dx * offset;
            let check_col = col as isize + dy * offset;

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
            let total = PatternAnalyzer::count_consecutive_bidirectional(board, pos.0, pos.1, dx, dy, player);
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

        // Check for gapped threats that need blocking
        let gapped_threats = Self::find_gapped_threats(board, opponent);
        if !gapped_threats.is_empty() {
            return Some(gapped_threats);
        }

        None
    }

    fn find_open_four_threats(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = board.get_player_bits(player);

        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let backward =
                    PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
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
        });

        threats.into_iter().collect()
    }

    /// Simple function to find gapped threats that need immediate blocking
    fn find_gapped_threats(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = board.get_player_bits(player);

        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                // Look for patterns like X.X.X or XX.X that could become winning
                let mut stones_found = vec![(row, col)];
                
                // Check forward direction for more stones with gaps
                for dist in 2..=6 {
                    let check_row = row as isize + dx * dist;
                    let check_col = col as isize + dy * dist;
                    
                    if PatternAnalyzer::is_in_bounds(board, check_row, check_col) {
                        let idx = board.index(check_row as usize, check_col as usize);
                        if Board::is_bit_set(&player_bits, idx) {
                            stones_found.push((check_row as usize, check_col as usize));
                        } else if Board::is_bit_set(&board.occupied, idx) {
                            // Hit opponent stone, stop looking
                            break;
                        }
                    } else {
                        break;
                    }
                }
                
                // If we found 3+ stones in a line with gaps, check if it's actually dangerous
                if stones_found.len() >= 3 {
                    let first = stones_found.first().unwrap();
                    let last = stones_found.last().unwrap();
                    
                    let start_row = first.0 as isize;
                    let start_col = first.1 as isize;
                    let end_row = last.0 as isize;
                    let end_col = last.1 as isize;
                    
                    let total_span = ((end_row - start_row).abs() + (end_col - start_col).abs()) + 1;
                    
                    // Only consider it a threat if:
                    // 1. The pattern spans 5 or fewer positions (could become 5-in-a-row)
                    // 2. There are actual empty gaps that could be filled
                    if total_span <= 5 {
                        let steps = ((end_row - start_row) / dx.max(1)).max((end_col - start_col) / dy.max(1));
                        let mut empty_gaps = 0;
                        let mut threat_positions = Vec::new();
                        
                        for step in 1..steps {
                            let gap_row = start_row + dx * step;
                            let gap_col = start_col + dy * step;
                            
                            if PatternAnalyzer::is_valid_empty(board, gap_row, gap_col) {
                                empty_gaps += 1;
                                threat_positions.push((gap_row as usize, gap_col as usize));
                            }
                        }
                        
                        // Only add as threats if there are few enough gaps to be completable
                        // and the pattern could actually form 5 in a row
                        if empty_gaps > 0 && stones_found.len() + empty_gaps >= 5 && empty_gaps <= 2 {
                            for pos in threat_positions {
                                threats.insert(pos);
                            }
                        }
                    }
                }
            }
        });

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
            .filter(|&(row, col)| !DoubleThreeDetection::creates_double_three(board, row, col, player))
            .collect();

        let mut prioritized_moves: Vec<((usize, usize), i32)> = filtered_moves
            .into_iter()
            .map(|mv| {
                let priority = Self::calculate_threat_priority(board, mv, player);
                (mv, priority)
            })
            .collect();

        prioritized_moves.sort_by_key(|(_, priority)| -priority);

        if prioritized_moves.len() > 25 {
            prioritized_moves.truncate(25);
        }

        prioritized_moves.into_iter().map(|(mv, _)| mv).collect()
    }

    fn calculate_threat_priority(board: &Board, mv: (usize, usize), player: Player) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        // Use heuristic-style pattern analysis for more accurate threat assessment
        for &check_player in &[player, player.opponent()] {
            let player_priority = Self::calculate_player_threat_value(board, row, col, check_player);
            
            if check_player == player {
                priority += player_priority * 2; // Offensive moves weighted higher
            } else {
                priority += player_priority; // Defensive moves (blocking opponent)
            }
        }

        // Capture bonus: significantly boost moves that create captures
        let capture_bonus = Self::calculate_capture_bonus(board, row, col, player);
        priority += capture_bonus;

        // Small positional bonus for center play
        let center = board.size / 2;
        let distance = Self::manhattan_distance(row, col, center, center) as i32;
        priority += CENTER_POSITION_BONUS - distance.min(CENTER_POSITION_BONUS);

        priority
    }

    fn calculate_player_threat_value(board: &Board, row: usize, col: usize, player: Player) -> i32 {
        let mut max_value = 0;

        // Analyze each direction for patterns
        for &(dx, dy) in &DIRECTIONS {
            let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
            let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, player);
            let total_stones = backward + forward + 1;

            if total_stones < 2 {
                continue;
            }

            // Check if this pattern has sufficient space to win (reuse heuristic function)
            let pattern_start_row = row as isize - dx * backward as isize;
            let pattern_start_col = col as isize - dy * backward as isize;
            let total_space = pattern_utils::count_total_space(
                board,
                pattern_start_row as usize,
                pattern_start_col as usize,
                dx,
                dy,
                total_stones,
            );
            if total_space < 5 { // Assuming win condition of 5
                continue;
            }

            // Use heuristic's existing pattern freedom analysis
            let pattern_start_row = row as isize - dx * backward as isize;
            let pattern_start_col = col as isize - dy * backward as isize;
            let freedom = pattern_utils::analyze_pattern_freedom(
                board,
                pattern_start_row as usize,
                pattern_start_col as usize,
                dx,
                dy,
                total_stones,
            );
            
            // Use shared pattern scoring function - single source of truth!
            let pattern_value = pattern_utils::get_pattern_score(total_stones, freedom);

            max_value = max_value.max(pattern_value);
        }

        max_value
    }

    fn calculate_capture_bonus(board: &Board, row: usize, col: usize, player: Player) -> i32 {
        let mut bonus = 0;
        let opponent = player.opponent();

        // Check all directions for capture opportunities
        for &(dx, dy) in &DIRECTIONS {
            let adj_row = row as isize + dx;
            let adj_col = col as isize + dy;
            
            if PatternAnalyzer::is_in_bounds(board, adj_row, adj_col) {
                let adj_row = adj_row as usize;
                let adj_col = adj_col as usize;
                
                // Check if adjacent position has opponent stone
                if let Some(piece_player) = board.get_player(adj_row, adj_col) {
                    if piece_player == opponent {
                        // Check for capture pattern: our_stone -> opponent -> opponent -> our_stone
                        let far_row = adj_row as isize + dx;
                        let far_col = adj_col as isize + dy;
                        
                        if PatternAnalyzer::is_in_bounds(board, far_row, far_col) {
                            let far_row = far_row as usize;
                            let far_col = far_col as usize;
                            
                            if let Some(piece_player) = board.get_player(far_row, far_col) {
                                if piece_player == opponent {
                                    // Check if there's our stone after the opponent pair
                                    let end_row = far_row as isize + dx;
                                    let end_col = far_col as isize + dy;
                                    
                                    if PatternAnalyzer::is_in_bounds(board, end_row, end_col) {
                                        let end_row = end_row as usize;
                                        let end_col = end_col as usize;
                                        
                                        if let Some(piece_player) = board.get_player(end_row, end_col) {
                                            if piece_player == player {
                                                // This creates a capture! Use scaled heuristic constant for consistency
                                                bonus += CAPTURE_BONUS_MULTIPLIER / 50; // 15000/50 = 300 points
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        bonus
    }

    fn find_threat_creating_moves(board: &Board, player: Player) -> HashSet<(usize, usize)> {
        let mut moves = HashSet::new();
        let player_bits = board.get_player_bits(player);

        board.iterate_bits(player_bits, |row, col| {
            for &(dx, dy) in &DIRECTIONS {
                let backward =
                    PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, player);
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
        });

        moves
    }



    fn get_zone_based_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut candidates = HashSet::new();
        let stone_count = board.count_stones();

        let zone_radius = if stone_count < 10 { 3 } else { 2 };
        let max_zone_moves = if stone_count < 10 { 25 } else { 20 };

        board.iterate_bits(&board.occupied, |row, col| {
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
        });

        let mut filtered_moves: Vec<(usize, usize)> = candidates
            .into_iter()
            .filter(|&(row, col)| !DoubleThreeDetection::creates_double_three(board, row, col, player))
            .collect();

        filtered_moves.sort_by_key(|&mv| -Self::calculate_threat_priority(board, mv, player));
        filtered_moves.truncate(max_zone_moves);
        filtered_moves
    }



    #[inline]
    pub fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }
}
