use crate::core::board::{Board, Player};
use std::collections::HashSet;

const DIRECTIONS: [(isize, isize); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];
const FREE_THREE_LENGTH: usize = 3;
const MAX_SEARCH_DISTANCE: isize = 4;

pub struct MoveHandler;

impl MoveHandler {
    /// Smart move generation with threat detection and zone-based filtering
    pub fn get_possible_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        if board.is_empty() {
            return vec![board.center()];
        }

        // STEP 1: Check for immediate winning moves
        if let Some(winning_move) = Self::find_winning_move(board, player) {
            return vec![winning_move];
        }

        // STEP 2: Check if opponent has winning threat - must block
        if let Some(block_moves) = Self::find_must_block_moves(board, player.opponent()) {
            if !block_moves.is_empty() {
                return block_moves;
            }
        }

        // STEP 3: Check for tactical positions (threats)
        let threat_moves = Self::find_threat_moves(board, player);
        if !threat_moves.is_empty() {
            return threat_moves;
        }

        // STEP 4: Normal position - zone-based generation
        Self::get_zone_based_moves(board, player)
    }

    /// Find immediate winning move (5-in-a-row)
    fn find_winning_move(board: &Board, player: Player) -> Option<(usize, usize)> {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        // Check each occupied stone for potential winning moves
        for array_idx in 0..board.u64_count {
            let mut bits = player_bits[array_idx];
            while bits != 0 {
                let bit_pos = bits.trailing_zeros() as usize;
                let global_idx = array_idx * 64 + bit_pos;
                if global_idx < board.total_cells {
                    let row = global_idx / board.size;
                    let col = global_idx % board.size;

                    // Check all 4 directions for winning move
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

    /// Find winning position in a specific direction
    fn find_win_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> Option<(usize, usize)> {
        // Count consecutive stones in both directions
        let backward = Self::count_consecutive_dir(board, row, col, -dx, -dy, player);
        let forward = Self::count_consecutive_dir(board, row, col, dx, dy, player);
        let total = backward + forward + 1;

        if total >= 4 {
            // Check for empty spot to complete 5-in-a-row
            // Check backward
            let back_row = row as isize - dx * (backward as isize + 1);
            let back_col = col as isize - dy * (backward as isize + 1);
            if Self::is_valid_empty(board, back_row, back_col) {
                return Some((back_row as usize, back_col as usize));
            }

            // Check forward
            let fwd_row = row as isize + dx * (forward as isize + 1);
            let fwd_col = col as isize + dy * (forward as isize + 1);
            if Self::is_valid_empty(board, fwd_row, fwd_col) {
                return Some((fwd_row as usize, fwd_col as usize));
            }

            // Check gaps within the sequence
            for i in 1..=backward {
                let check_row = row as isize - dx * i as isize;
                let check_col = col as isize - dy * i as isize;
                if Self::is_valid_empty(board, check_row, check_col) {
                    // Check if filling this creates 5-in-row
                    let pos = (check_row as usize, check_col as usize);
                    if Self::creates_five_in_row(board, pos, player) {
                        return Some(pos);
                    }
                }
            }

            for i in 1..=forward {
                let check_row = row as isize + dx * i as isize;
                let check_col = col as isize + dy * i as isize;
                if Self::is_valid_empty(board, check_row, check_col) {
                    let pos = (check_row as usize, check_col as usize);
                    if Self::creates_five_in_row(board, pos, player) {
                        return Some(pos);
                    }
                }
            }
        }

        None
    }

    /// Check if placing stone creates 5-in-a-row
    fn creates_five_in_row(board: &Board, pos: (usize, usize), player: Player) -> bool {
        for &(dx, dy) in &DIRECTIONS {
            let backward = Self::count_consecutive_dir(board, pos.0, pos.1, -dx, -dy, player);
            let forward = Self::count_consecutive_dir(board, pos.0, pos.1, dx, dy, player);
            if backward + forward + 1 >= 5 {
                return true;
            }
        }
        false
    }

    /// Count consecutive stones in one direction
    fn count_consecutive_dir(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        let mut count = 0;
        let mut r = row as isize + dx;
        let mut c = col as isize + dy;

        while r >= 0 && r < board.size as isize && c >= 0 && c < board.size as isize {
            let idx = board.index(r as usize, c as usize);
            if Board::is_bit_set(player_bits, idx) {
                count += 1;
                r += dx;
                c += dy;
            } else {
                break;
            }
        }
        count
    }

    /// Find moves that must block opponent's winning threats
    fn find_must_block_moves(board: &Board, opponent: Player) -> Option<Vec<(usize, usize)>> {
        // Check if opponent has 4-in-a-row or winning threat
        if let Some(opp_win) = Self::find_winning_move(board, opponent) {
            return Some(vec![opp_win]);
        }

        // Check for open-4 threats (must block)
        let open_fours = Self::find_open_four_threats(board, opponent);
        if !open_fours.is_empty() {
            return Some(open_fours);
        }

        None
    }

    /// Find open-four threats (4-in-a-row with both ends open)
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
                        let backward = Self::count_consecutive_dir(board, row, col, -dx, -dy, player);
                        let forward = Self::count_consecutive_dir(board, row, col, dx, dy, player);
                        
                        if backward + forward + 1 == 4 {
                            // Check if both ends are open
                            let back_row = row as isize - dx * (backward as isize + 1);
                            let back_col = col as isize - dy * (backward as isize + 1);
                            let fwd_row = row as isize + dx * (forward as isize + 1);
                            let fwd_col = col as isize + dy * (forward as isize + 1);

                            if Self::is_valid_empty(board, back_row, back_col) {
                                threats.insert((back_row as usize, back_col as usize));
                            }
                            if Self::is_valid_empty(board, fwd_row, fwd_col) {
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

    /// Find threat-creating and threat-blocking moves
    fn find_threat_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut moves = HashSet::new();

        // Find our threat-creating moves (3-in-a-row, 4-in-a-row)
        let our_threats = Self::find_threat_creating_moves(board, player);
        moves.extend(our_threats);

        // Find opponent threat-blocking moves
        let opp_threats = Self::find_threat_creating_moves(board, player.opponent());
        moves.extend(opp_threats);

        if moves.len() > 30 {
            // Too many moves, fall back to zone generation
            return Vec::new();
        }

        moves.into_iter().collect()
    }

    /// Find moves that create threats (3-in-row or 4-in-row)
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
                        let backward = Self::count_consecutive_dir(board, row, col, -dx, -dy, player);
                        let forward = Self::count_consecutive_dir(board, row, col, dx, dy, player);
                        let total = backward + forward + 1;

                        // Look for 3-in-row or 4-in-row patterns
                        if total >= 2 && total <= 4 {
                            // Add empty positions around this pattern
                            for offset in -(backward as isize + 1)..=(forward as isize + 1) {
                                let r = row as isize + dx * offset;
                                let c = col as isize + dy * offset;
                                if Self::is_valid_empty(board, r, c) {
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

    /// Zone-based move generation (only near existing stones)
    fn get_zone_based_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut candidates = HashSet::new();
        let stone_count = board.count_stones();
        
        // Adaptive zone radius
        let zone_radius = if stone_count < 10 {
            2  // Early game: wider radius
        } else if stone_count < 30 {
            1  // Mid game: tighter radius
        } else {
            1  // Late game: tight radius
        };

        // Generate moves around each occupied stone
        for array_idx in 0..board.u64_count {
            let mut bits = board.occupied[array_idx];
            while bits != 0 {
                let bit_pos = bits.trailing_zeros() as usize;
                let global_idx = array_idx * 64 + bit_pos;
                if global_idx < board.total_cells {
                    let row = global_idx / board.size;
                    let col = global_idx % board.size;

                    // Add all cells within zone radius
                    for dr in -(zone_radius as isize)..=(zone_radius as isize) {
                        for dc in -(zone_radius as isize)..=(zone_radius as isize) {
                            if dr == 0 && dc == 0 {
                                continue;
                            }
                            let nr = row as isize + dr;
                            let nc = col as isize + dc;
                            
                            if Self::is_valid_empty(board, nr, nc) {
                                candidates.insert((nr as usize, nc as usize));
                            }
                        }
                    }
                }
                bits &= bits - 1;
            }
        }

        // Filter out double-three violations
        candidates
            .into_iter()
            .filter(|&(row, col)| !RuleValidator::creates_double_three(board, row, col, player))
            .collect()
    }

    /// Check if position is valid and empty
    fn is_valid_empty(board: &Board, row: isize, col: isize) -> bool {
        if row < 0 || col < 0 || row >= board.size as isize || col >= board.size as isize {
            return false;
        }
        let idx = board.index(row as usize, col as usize);
        !Board::is_bit_set(&board.occupied, idx)
    }
}

pub struct RuleValidator;

impl RuleValidator {
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
        let player_bits = match player {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };
        let opponent_bits = match player.opponent() {
            Player::Max => &board.max_bits,
            Player::Min => &board.min_bits,
        };

        let mut stones = 0;
        let mut empty_found = false;
        let mut is_open = false;

        for i in 1..=MAX_SEARCH_DISTANCE {
            let new_row = row as isize + dr * i;
            let new_col = col as isize + dc * i;

            if !Self::is_valid_pos(board, new_row, new_col) {
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

    fn can_form_open_four(left_open: bool, right_open: bool) -> bool {
        left_open || right_open
    }

    fn is_valid_pos(board: &Board, row: isize, col: isize) -> bool {
        (0..board.size as isize).contains(&row) && (0..board.size as isize).contains(&col)
    }
}
