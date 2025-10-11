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
        let player_bits = board.get_player_bits(player);
        let mut result = None;
        
        board.iterate_bits(player_bits, |row, col| {
            if result.is_some() {
                return;
            }
            for &(dx, dy) in &DIRECTIONS {
                if let Some(win_pos) = Self::find_win_in_direction(board, row, col, dx, dy, player) {
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

        const MAX_SEARCH_DISTANCE: isize = 5;

        for offset in -MAX_SEARCH_DISTANCE..=MAX_SEARCH_DISTANCE {
            let check_row = row as isize + dx * offset;
            let check_col = col as isize + dy * offset;

            if offset == 0 {
                continue;
            }

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

        None
    }

    fn find_open_four_threats(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut threats = HashSet::new();
        let player_bits = board.get_player_bits(player);

        board.iterate_bits(player_bits, |row, col| {
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
            .filter(|&(row, col)| !GameRules::creates_double_three(board, row, col, player))
            .collect();

        if filtered_moves.len() > 30 {
            let mut prioritized_moves: Vec<((usize, usize), i32)> = filtered_moves
                .into_iter()
                .map(|mv| {
                    let priority = Self::calculate_threat_priority(board, mv, player);
                    (mv, priority)
                })
                .collect();

            prioritized_moves.sort_by_key(|(_, priority)| -priority);

            prioritized_moves.truncate(30);
            prioritized_moves.into_iter().map(|(mv, _)| mv).collect()
        } else {
            filtered_moves
        }
    }

    fn calculate_threat_priority(board: &Board, mv: (usize, usize), player: Player) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        for &check_player in &[player, player.opponent()] {
            for &(dx, dy) in &DIRECTIONS {
                let backward = PatternAnalyzer::count_consecutive(board, row, col, -dx, -dy, check_player);
                let forward = PatternAnalyzer::count_consecutive(board, row, col, dx, dy, check_player);
                let total = backward + forward + 1;

                let pattern_value = match total {
                    5 => 10000,
                    4 => {

                        let back_row = row as isize - dx * (backward as isize + 1);
                        let back_col = col as isize - dy * (backward as isize + 1);
                        let fwd_row = row as isize + dx * (forward as isize + 1);
                        let fwd_col = col as isize + dy * (forward as isize + 1);
                        
                        let back_open = PatternAnalyzer::is_valid_empty(board, back_row, back_col);
                        let fwd_open = PatternAnalyzer::is_valid_empty(board, fwd_row, fwd_col);
                        
                        if back_open && fwd_open {
                            1000
                        } else {
                            500
                        }
                    }
                    3 => 200,
                    2 => 50,
                    _ => 0,
                };

                if check_player == player {
                    priority += pattern_value * 2;
                } else {
                    priority += pattern_value;
                }
            }
        }

        let center = board.size / 2;
        let distance = PatternAnalyzer::manhattan_distance(row, col, center, center) as i32;
        priority += 10 - distance.min(10);

        priority
    }

    fn find_threat_creating_moves(board: &Board, player: Player) -> HashSet<(usize, usize)> {
        let mut moves = HashSet::new();
        let player_bits = board.get_player_bits(player);

        board.iterate_bits(player_bits, |row, col| {
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
        });

        moves
    }

    fn get_zone_based_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut candidates = HashSet::new();
        let stone_count = board.count_stones();
        
        let zone_radius = if stone_count < 10 { 2 } else { 1 };

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

        candidates
            .into_iter()
            .filter(|&(row, col)| !GameRules::creates_double_three(board, row, col, player))
            .collect()
    }
}
