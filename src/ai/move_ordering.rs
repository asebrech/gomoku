use crate::core::board::{Board, Player};
use crate::core::state::GameState;
use std::collections::HashMap;

pub struct MoveOrdering;

// Killer move table for tracking good moves at each depth
#[derive(Debug, Clone)]
pub struct KillerTable {
    killers: HashMap<i32, [Option<(usize, usize)>; 2]>, // depth -> [killer1, killer2]
}

// History heuristic table for tracking move success rates
#[derive(Debug, Clone)]
pub struct HistoryTable {
    history: HashMap<(usize, usize), u32>, // move -> success count
    max_history: u32,
}

const DIRECTIONS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
const ALL_DIRECTIONS: [(isize, isize); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

impl KillerTable {
    pub fn new() -> Self {
        Self {
            killers: HashMap::new(),
        }
    }
    
    pub fn store_killer(&mut self, depth: i32, mv: (usize, usize)) {
        let entry = self.killers.entry(depth).or_insert([None, None]);
        
        // Don't store if already present
        if entry[0] == Some(mv) || entry[1] == Some(mv) {
            return;
        }
        
        // Shift killers: move killer2 to killer1, new move to killer2
        entry[0] = entry[1];
        entry[1] = Some(mv);
    }
    
    pub fn get_killers(&self, depth: i32) -> &[Option<(usize, usize)>; 2] {
        static EMPTY: [Option<(usize, usize)>; 2] = [None, None];
        self.killers.get(&depth).unwrap_or(&EMPTY)
    }
    
    pub fn clear(&mut self) {
        self.killers.clear();
    }
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            max_history: 10000,
        }
    }
    
    pub fn record_cutoff(&mut self, mv: (usize, usize), depth: i32) {
        let score = self.history.entry(mv).or_insert(0);
        *score = (*score + depth as u32).min(self.max_history);
    }
    
    pub fn get_history_score(&self, mv: (usize, usize)) -> u32 {
        *self.history.get(&mv).unwrap_or(&0)
    }
    
    pub fn clear(&mut self) {
        self.history.clear();
    }
    
    // Age history scores to prevent old data from dominating
    pub fn age_history(&mut self) {
        for score in self.history.values_mut() {
            *score /= 2;
        }
        // Remove entries that became too small
        self.history.retain(|_, &mut score| score > 0);
    }
}

impl MoveOrdering {
    // Basic move ordering - use this when you don't have killer/history tables
    pub fn order_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
        let center = state.board.size / 2;
        moves.sort_unstable_by_key(|&mv| -Self::calculate_move_priority(state, mv, center));
    }
    
    // Enhanced move ordering with killer moves and history heuristic
    pub fn order_moves_enhanced(
        state: &GameState, 
        moves: &mut Vec<(usize, usize)>, 
        tt_move: Option<(usize, usize)>,
        killer_table: &KillerTable,
        history_table: &HistoryTable,
        depth: i32
    ) {
        let center = state.board.size / 2;
        let killers = killer_table.get_killers(depth);
        
        // First, do a quick sort with basic criteria
        moves.sort_by_cached_key(|&mv| {
            let mut score = 0i32;
            
            // TT move gets highest priority
            if Some(mv) == tt_move {
                return -100000;
            }
            
            // Killer moves get high priority
            if killers[0] == Some(mv) {
                return -50000;
            }
            if killers[1] == Some(mv) {
                return -40000;
            }
            
            // History heuristic
            let history_score = history_table.get_history_score(mv) as i32;
            score -= history_score * 10;
            
            // Quick tactical evaluation (much faster than before)
            if depth >= 4 { // Only do tactical evaluation for deeper searches
                let tactical_score = Self::quick_tactical_score(state, mv);
                score -= tactical_score;
            }
            
            // Basic positional scoring
            score -= Self::calculate_move_priority(state, mv, center);
            
            score
        });
        
        // For very deep searches, do more detailed evaluation on the top moves only
        if depth >= 8 && moves.len() > 10 {
            let top_moves = moves.iter().take(10).cloned().collect::<Vec<_>>();
            let mut scored_moves: Vec<_> = top_moves.iter().map(|&mv| {
                let tactical_score = Self::evaluate_move_tactics(state, mv);
                (mv, -tactical_score) // Negative because we want higher scores first
            }).collect();
            
            scored_moves.sort_by_key(|&(_, score)| score);
            
            // Replace the top 10 moves with the newly sorted ones
            for (i, &(mv, _)) in scored_moves.iter().enumerate() {
                moves[i] = mv;
            }
        }
    }
    
    // Quick tactical scoring that doesn't require expensive operations
    fn quick_tactical_score(state: &GameState, mv: (usize, usize)) -> i32 {
        let mut score = 0;
        
        // Check immediate threat creation
        score += Self::evaluate_threats_created(&state.board, mv, state.current_player) * 10;
        
        // Check if this blocks opponent threats
        score += Self::evaluate_threats_created(&state.board, mv, state.current_player.opponent()) * 8;
        
        // Local pattern bonus
        score += Self::evaluate_local_patterns(&state.board, mv, state.current_player);
        
        score
    }
    
    // Evaluate tactical importance of a move (threats, captures, blocks)
    fn evaluate_move_tactics(state: &GameState, mv: (usize, usize)) -> i32 {
        let mut score = 0;
        let (row, col) = mv;
        
        // Quick evaluation without creating full game state clone
        // This is much faster than the previous approach
        
        // Evaluate immediate threats created by this move
        score += Self::evaluate_threats_created(&state.board, mv, state.current_player) * 100;
        
        // Quick check for opponent blocking - just check if this move would block
        // any immediate winning threats from the opponent
        let opponent = state.current_player.opponent();
        score += Self::evaluate_threats_created(&state.board, mv, opponent) * 80;
        
        // Evaluate local patterns around this move
        score += Self::evaluate_local_patterns(&state.board, mv, state.current_player) * 50;
        
        score
    }
    
    // Evaluate threats created by placing a stone at this position
    fn evaluate_threats_created(board: &Board, mv: (usize, usize), player: Player) -> i32 {
        let (row, col) = mv;
        let mut max_threat = 0;
        
        for &(dx, dy) in &DIRECTIONS {
            let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);
            let threat_level = match consecutive {
                5 => 1000,  // Win
                4 => {
                    // Check if it's an open four or closed four
                    if Self::is_open_threat(board, row, col, dx, dy, player, 4) {
                        800  // Open four - very dangerous
                    } else {
                        400  // Closed four - still threatening
                    }
                }
                3 => {
                    if Self::is_open_threat(board, row, col, dx, dy, player, 3) {
                        200  // Open three
                    } else {
                        50   // Closed three
                    }
                }
                2 => {
                    if Self::is_open_threat(board, row, col, dx, dy, player, 2) {
                        20   // Open two
                    } else {
                        5    // Closed two
                    }
                }
                _ => 0,
            };
            max_threat = max_threat.max(threat_level);
        }
        
        max_threat
    }
    
    // Check if a threat is "open" (has empty spaces on both ends)
    fn is_open_threat(board: &Board, row: usize, col: usize, dx: isize, dy: isize, player: Player, length: usize) -> bool {
        let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
        let forwards = Self::count_direction(board, row, col, dx, dy, player);
        let total_length = backwards + forwards + 1;
        
        if total_length < length {
            return false;
        }
        
        // Check if both ends are open
        let back_row = row as isize - (backwards as isize + 1) * dx;
        let back_col = col as isize - (backwards as isize + 1) * dy;
        let front_row = row as isize + (forwards as isize + 1) * dx;
        let front_col = col as isize + (forwards as isize + 1) * dy;
        
        let back_open = Self::is_position_empty_and_valid(board, back_row, back_col);
        let front_open = Self::is_position_empty_and_valid(board, front_row, front_col);
        
        back_open && front_open
    }
    
    fn is_position_empty_and_valid(board: &Board, row: isize, col: isize) -> bool {
        if row < 0 || col < 0 || row >= board.size as isize || col >= board.size as isize {
            return false;
        }
        
        let idx = board.index(row as usize, col as usize);
        !Board::is_bit_set(&board.occupied, idx)
    }
    
    // Evaluate special patterns like double threats
    fn evaluate_patterns(board: &Board, mv: (usize, usize), player: Player) -> i32 {
        let mut pattern_score = 0;
        let (row, col) = mv;
        
        // Count the number of different directions that create threats
        let mut threat_directions = 0;
        
        for &(dx, dy) in &DIRECTIONS {
            let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);
            if consecutive >= 3 {
                threat_directions += 1;
            }
        }
        
        // Double threat (threats in multiple directions) is very powerful
        if threat_directions >= 2 {
            pattern_score += 300;
        }
        
        // Check for fork patterns (creating multiple open threes)
        let mut open_threes = 0;
        for &(dx, dy) in &DIRECTIONS {
            let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);
            if consecutive == 3 && Self::is_open_threat(board, row, col, dx, dy, player, 3) {
                open_threes += 1;
            }
        }
        
        if open_threes >= 2 {
            pattern_score += 500; // Fork with multiple open threes
        }
        
        pattern_score
    }
    
    // Fast local pattern evaluation without expensive cloning
    fn evaluate_local_patterns(board: &Board, mv: (usize, usize), player: Player) -> i32 {
        let (row, col) = mv;
        let mut score = 0;
        
        // Check for creating multiple threats in one move
        let mut consecutive_counts = Vec::new();
        for &(dx, dy) in &DIRECTIONS {
            let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);
            consecutive_counts.push(consecutive);
        }
        
        // Bonus for creating multiple medium-length sequences
        let threes_and_fours = consecutive_counts.iter().filter(|&&count| count >= 3).count();
        if threes_and_fours >= 2 {
            score += 200; // Creating multiple threats
        }
        
        // Bonus for creating very long sequences
        let max_consecutive = consecutive_counts.iter().max().unwrap_or(&0);
        score += match max_consecutive {
            5 => 1000, // Win
            4 => 400,  // Four in a row
            3 => 100,  // Three in a row
            _ => 0,
        };
        
        score
    }

    fn calculate_move_priority(state: &GameState, mv: (usize, usize), center: usize) -> i32 {
        let (row, col) = mv;
        let mut priority = 0;

        let center_distance = Self::manhattan_distance(row, col, center, center);
        priority += 100 - center_distance as i32;

        priority += Self::calculate_threat_priority(&state.board, row, col);
        priority += Self::calculate_adjacency_bonus(&state.board, row, col);

        priority
    }

    fn calculate_threat_priority(board: &Board, row: usize, col: usize) -> i32 {
        let mut threat_score = 0;

        for &player in &[Player::Max, Player::Min] {
            for &(dx, dy) in &DIRECTIONS {
                let consecutive = Self::simulate_move_consecutive(board, row, col, dx, dy, player);
                let multiplier = if player == Player::Max { 1 } else { -1 };

                threat_score += multiplier
                    * match consecutive {
                        5 => 10000,
                        4 => 5000,
                        3 => 1000,
                        _ => 0,
                    };
            }
        }
        threat_score
    }

    fn simulate_move_consecutive(
        board: &Board,
        row: usize,
        col: usize,
        dx: isize,
        dy: isize,
        player: Player,
    ) -> usize {
        let backwards = Self::count_direction(board, row, col, -dx, -dy, player);
        let forwards = Self::count_direction(board, row, col, dx, dy, player);
        backwards + forwards + 1
    }

    fn count_direction(
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
        let mut current_row = row as isize + dx;
        let mut current_col = col as isize + dy;

        while current_row >= 0
            && current_row < board.size as isize
            && current_col >= 0
            && current_col < board.size as isize
        {
            let idx = board.index(current_row as usize, current_col as usize);
            if Board::is_bit_set(player_bits, idx) {
                count += 1;
                current_row += dx;
                current_col += dy;
            } else {
                break;
            }
        }
        count
    }

    fn calculate_adjacency_bonus(board: &Board, row: usize, col: usize) -> i32 {
        let mut neighbor_mask = vec![0u64; board.u64_count];
        let mut num_adjacent = 0;

        for &(dx, dy) in &ALL_DIRECTIONS {
            let nr = row as isize + dx;
            let nc = col as isize + dy;
            if nr >= 0 && nc >= 0 && nr < board.size as isize && nc < board.size as isize {
                let idx = board.index(nr as usize, nc as usize);
                Board::set_bit(&mut neighbor_mask, idx);
            }
        }

        for (&o, &m) in board.occupied.iter().zip(&neighbor_mask) {
            num_adjacent += (o & m).count_ones() as i32;
        }

        num_adjacent * 50
    }

    fn manhattan_distance(row1: usize, col1: usize, row2: usize, col2: usize) -> usize {
        ((row1 as isize - row2 as isize).abs() + (col1 as isize - col2 as isize).abs()) as usize
    }
}
