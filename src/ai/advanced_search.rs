use crate::core::state::GameState;
use crate::core::board::Player;
use std::cmp::{max, min};
use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

use super::{heuristic::Heuristic, transposition::{TranspositionTable, BoundType}};

// Atomic counters for profiling (thread-safe)
static MINIMAX_CALLS: AtomicUsize = AtomicUsize::new(0);
static TT_HITS: AtomicUsize = AtomicUsize::new(0);
static HEURISTIC_CALLS: AtomicUsize = AtomicUsize::new(0);
static MOVE_GENERATION_TIME: AtomicU64 = AtomicU64::new(0);
static HEURISTIC_TIME: AtomicU64 = AtomicU64::new(0);
static TT_TIME: AtomicU64 = AtomicU64::new(0);
static KILLER_HITS: AtomicUsize = AtomicUsize::new(0);
static HISTORY_HITS: AtomicUsize = AtomicUsize::new(0);

// Maximum search depth for arrays
const MAX_DEPTH: usize = 32;

// Killer moves: store two best moves at each depth that caused cutoffs
#[derive(Debug, Clone, Copy)]
pub struct KillerMoves {
    primary: Option<(usize, usize)>,
    secondary: Option<(usize, usize)>,
}

impl KillerMoves {
    pub fn new() -> Self {
        Self {
            primary: None,
            secondary: None,
        }
    }

    pub fn store(&mut self, move_: (usize, usize)) {
        if Some(move_) != self.primary {
            self.secondary = self.primary;
            self.primary = Some(move_);
        }
    }

    pub fn contains(&self, move_: (usize, usize)) -> bool {
        Some(move_) == self.primary || Some(move_) == self.secondary
    }

    pub fn get_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        if let Some(mv) = self.primary {
            moves.push(mv);
        }
        if let Some(mv) = self.secondary {
            moves.push(mv);
        }
        moves
    }
}

// History heuristic: track move success across the entire search
#[derive(Debug)]
pub struct HistoryTable {
    // [from_player][row][col] -> score
    table: [[[i32; 19]; 19]; 2],
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            table: [[[0; 19]; 19]; 2],
        }
    }

    pub fn record_cutoff(&mut self, move_: (usize, usize), player: Player, depth: i32) {
        let player_idx = if player == Player::Max { 0 } else { 1 };
        let (row, col) = move_;
        if row < 19 && col < 19 {
            // Square of depth gives higher weight to deeper cutoffs
            self.table[player_idx][row][col] += depth * depth;
        }
    }

    pub fn get_score(&self, move_: (usize, usize), player: Player) -> i32 {
        let player_idx = if player == Player::Max { 0 } else { 1 };
        let (row, col) = move_;
        if row < 19 && col < 19 {
            self.table[player_idx][row][col]
        } else {
            0
        }
    }

    pub fn clear(&mut self) {
        self.table = [[[0; 19]; 19]; 2];
    }

    pub fn age(&mut self) {
        // Reduce all scores by half to favor recent moves
        for player in 0..2 {
            for row in 0..19 {
                for col in 0..19 {
                    self.table[player][row][col] /= 2;
                }
            }
        }
    }
}

// Advanced search context with all heuristics
pub struct SearchContext {
    killer_moves: [KillerMoves; MAX_DEPTH],
    history_table: HistoryTable,
    start_time: Instant,
    time_limit: Option<Duration>,
    nodes_searched: usize,
}

impl SearchContext {
    pub fn new() -> Self {
        Self {
            killer_moves: [KillerMoves::new(); MAX_DEPTH],
            history_table: HistoryTable::new(),
            start_time: Instant::now(),
            time_limit: None,
            nodes_searched: 0,
        }
    }

    pub fn new_search(&mut self, time_limit: Option<Duration>) {
        // Age history table but don't clear it completely
        self.history_table.age();
        self.start_time = Instant::now();
        self.time_limit = time_limit;
        self.nodes_searched = 0;
    }

    pub fn is_time_up(&self) -> bool {
        if let Some(limit) = self.time_limit {
            self.start_time.elapsed() >= limit
        } else {
            false
        }
    }

    pub fn get_nodes_searched(&self) -> usize {
        self.nodes_searched
    }

    pub fn order_moves(&mut self, moves: &mut Vec<(usize, usize)>, depth: usize, current_player: Player, tt_best: Option<(usize, usize)>) {
        let depth_idx = depth.min(MAX_DEPTH - 1);
        
        // Create a vector of (move, priority) pairs
        let mut move_priorities: Vec<((usize, usize), i32)> = moves.iter().map(|&mv| {
            let mut priority = 0;

            // 1. TT best move gets highest priority
            if Some(mv) == tt_best {
                priority += 1000000;
            }

            // 2. URGENCY PRIORITY: Check for immediate threats (for shallow depths)
            if depth <= 2 {
                // This is expensive but critical for shallow search accuracy
                let urgency = self.evaluate_move_urgency_quick(mv, current_player);
                priority += urgency;
            }

            // 3. Killer moves get high priority
            if self.killer_moves[depth_idx].contains(mv) {
                priority += 50000;
                KILLER_HITS.fetch_add(1, Ordering::Relaxed);
            }

            // 4. History heuristic
            let history_score = self.history_table.get_score(mv, current_player);
            if history_score > 0 {
                priority += history_score;
                HISTORY_HITS.fetch_add(1, Ordering::Relaxed);
            }

            // 5. Basic positional scoring (center preference)
            let (row, col) = mv;
            let center = 9; // Assuming 19x19 board
            let distance_from_center = ((row as i32 - center).abs() + (col as i32 - center).abs()) as i32;
            priority += 100 - distance_from_center;

            (mv, priority)
        }).collect();

        // Sort by priority (highest first)
        move_priorities.sort_unstable_by_key(|(_, priority)| -priority);

        // Extract the ordered moves
        *moves = move_priorities.into_iter().map(|(mv, _)| mv).collect();
    }

    // Quick urgency evaluation for move ordering (simplified version)
    fn evaluate_move_urgency_quick(&self, mv: (usize, usize), player: Player) -> i32 {
        let (row, col) = mv;
        
        // Simple pattern check - count adjacent pieces in all directions
        let mut max_urgency = 0;
        
        for &(dx, dy) in &[(1,0), (0,1), (1,1), (1,-1)] {
            let mut adjacent_count = 0;
            
            // Count in both directions
            for direction in [-1, 1] {
                let mut r = row as isize + dx * direction;
                let mut c = col as isize + dy * direction;
                let mut count = 0;
                
                while r >= 0 && r < 19 && c >= 0 && c < 19 && count < 4 {
                    // This is a simplified check - in real code we'd need access to the board
                    // For now, we'll just use a basic heuristic
                    r += dx * direction;
                    c += dy * direction;
                    count += 1;
                }
                adjacent_count += count;
            }
            
            // Simple urgency scoring based on potential
            let urgency = match adjacent_count {
                4.. => 100000,  // Potentially critical
                3 => 50000,     // High priority
                2 => 10000,     // Medium priority
                1 => 1000,      // Low priority
                _ => 0,
            };
            
            max_urgency = max_urgency.max(urgency);
        }
        
        max_urgency
    }

    pub fn record_killer(&mut self, move_: (usize, usize), depth: usize) {
        let depth_idx = depth.min(MAX_DEPTH - 1);
        self.killer_moves[depth_idx].store(move_);
    }

    pub fn record_history_cutoff(&mut self, move_: (usize, usize), player: Player, depth: i32) {
        self.history_table.record_cutoff(move_, player, depth);
    }
}

pub fn reset_profiling() {
    MINIMAX_CALLS.store(0, Ordering::Relaxed);
    TT_HITS.store(0, Ordering::Relaxed);
    HEURISTIC_CALLS.store(0, Ordering::Relaxed);
    MOVE_GENERATION_TIME.store(0, Ordering::Relaxed);
    HEURISTIC_TIME.store(0, Ordering::Relaxed);
    TT_TIME.store(0, Ordering::Relaxed);
    KILLER_HITS.store(0, Ordering::Relaxed);
    HISTORY_HITS.store(0, Ordering::Relaxed);
}

pub fn print_profiling() {
    let minimax_calls = MINIMAX_CALLS.load(Ordering::Relaxed);
    let tt_hits = TT_HITS.load(Ordering::Relaxed);
    let heuristic_calls = HEURISTIC_CALLS.load(Ordering::Relaxed);
    let move_gen_time = MOVE_GENERATION_TIME.load(Ordering::Relaxed);
    let heur_time = HEURISTIC_TIME.load(Ordering::Relaxed);
    let tt_time = TT_TIME.load(Ordering::Relaxed);
    let killer_hits = KILLER_HITS.load(Ordering::Relaxed);
    let history_hits = HISTORY_HITS.load(Ordering::Relaxed);
    
    println!("=== ADVANCED SEARCH PROFILING ===");
    println!("Minimax calls: {}", minimax_calls);
    println!("Transposition table hits: {} ({}%)", tt_hits, if minimax_calls > 0 { tt_hits * 100 / minimax_calls } else { 0 });
    println!("Killer move hits: {}", killer_hits);
    println!("History heuristic hits: {}", history_hits);
    println!("Heuristic evaluations: {}", heuristic_calls);
    println!("Move generation time: {}μs", move_gen_time);
    println!("Heuristic evaluation time: {}μs", heur_time);
    println!("Transposition table time: {}μs", tt_time);
    println!("==================================");
}

// Advanced minimax with all optimizations
pub fn minimax_advanced(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &mut TranspositionTable,
    ctx: &mut SearchContext,
) -> i32 {
    MINIMAX_CALLS.fetch_add(1, Ordering::Relaxed);
    ctx.nodes_searched += 1;

    // Check time limit periodically
    if ctx.nodes_searched % 1000 == 0 && ctx.is_time_up() {
        return 0; // Emergency exit
    }
    
    // Check transposition table first with enhanced lookup
    let tt_start = Instant::now();
    let cached_result = tt.lookup_enhanced(state.zobrist_hash, depth, alpha, beta);
    TT_TIME.fetch_add(tt_start.elapsed().as_micros() as u64, Ordering::Relaxed);
    
    let tt_best_move = if let Some((_cached_value, best_move)) = cached_result {
        if let Some((cached_value, _)) = cached_result {
            TT_HITS.fetch_add(1, Ordering::Relaxed);
            return cached_value;
        }
        best_move
    } else {
        None
    };

    if depth == 0 || state.is_terminal() {
        let heur_start = Instant::now();
        let eval = Heuristic::evaluate(state, depth);
        HEURISTIC_TIME.fetch_add(heur_start.elapsed().as_micros() as u64, Ordering::Relaxed);
        HEURISTIC_CALLS.fetch_add(1, Ordering::Relaxed);
        
        let tt_store_start = Instant::now();
        tt.store_enhanced(state.zobrist_hash, eval, depth, BoundType::Exact, None);
        TT_TIME.fetch_add(tt_store_start.elapsed().as_micros() as u64, Ordering::Relaxed);
        return eval;
    }

    // Collect and order moves using all heuristics
    let move_gen_start = Instant::now();
    let mut moves = Vec::with_capacity(64);
    state.for_each_possible_move(|mv| moves.push(mv));
    MOVE_GENERATION_TIME.fetch_add(move_gen_start.elapsed().as_micros() as u64, Ordering::Relaxed);
    
    if moves.is_empty() {
        return Heuristic::evaluate(state, depth);
    }

    // Advanced move ordering with all heuristics
    let current_player = state.current_player;
    ctx.order_moves(&mut moves, depth as usize, current_player, tt_best_move);
    
    let original_alpha = alpha;
    let original_beta = beta;
    let mut best_move = None;
    let mut best_score = if maximizing_player { i32::MIN } else { i32::MAX };
    
    // Principal Variation Search (PVS) style
    let mut first_move = true;
    
    for &move_ in &moves {
        state.make_move(move_);
        
        let score = if first_move {
            // Search first move with full window
            minimax_advanced(state, depth - 1, alpha, beta, !maximizing_player, tt, ctx)
        } else {
            // Search remaining moves with null window first
            let null_score = minimax_advanced(state, depth - 1, 
                if maximizing_player { alpha } else { beta - 1 }, 
                if maximizing_player { alpha + 1 } else { beta }, 
                !maximizing_player, tt, ctx);
            
            // If null window search indicates this might be better, re-search with full window
            if maximizing_player && null_score > alpha || !maximizing_player && null_score < beta {
                minimax_advanced(state, depth - 1, alpha, beta, !maximizing_player, tt, ctx)
            } else {
                null_score
            }
        };
        
        state.undo_move(move_);
        first_move = false;

        if maximizing_player {
            if score > best_score {
                best_score = score;
                best_move = Some(move_);
            }
            alpha = max(alpha, score);
            if beta <= alpha {
                // Alpha-beta cutoff - record this move in killer and history tables
                ctx.record_killer(move_, depth as usize);
                ctx.record_history_cutoff(move_, current_player, depth);
                break;
            }
        } else {
            if score < best_score {
                best_score = score;
                best_move = Some(move_);
            }
            beta = min(beta, score);
            if beta <= alpha {
                // Alpha-beta cutoff - record this move in killer and history tables
                ctx.record_killer(move_, depth as usize);
                ctx.record_history_cutoff(move_, current_player, depth);
                break;
            }
        }
    }
    
    // Determine bound type based on how search ended
    let bound_type = if best_score <= original_alpha {
        BoundType::UpperBound  // Failed low
    } else if best_score >= original_beta {
        BoundType::LowerBound  // Failed high  
    } else {
        BoundType::Exact       // Exact value
    };
    
    let tt_store_start = Instant::now();
    tt.store_enhanced(state.zobrist_hash, best_score, depth, bound_type, best_move);
    TT_TIME.fetch_add(tt_store_start.elapsed().as_micros() as u64, Ordering::Relaxed);
    
    best_score
}

// Iterative deepening search - returns (best_move, depth_reached)
pub fn iterative_deepening_search(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &mut TranspositionTable,
    ctx: &mut SearchContext,
) -> (Option<(usize, usize)>, i32) {
    ctx.new_search(time_limit);
    let mut best_move = None;
    let mut best_score = if state.current_player == Player::Max { i32::MIN } else { i32::MAX };
    let mut depth_reached = 0;
    
    println!("Starting iterative deepening search (max depth: {})", max_depth);
    
    for depth in 1..=max_depth {
        if ctx.is_time_up() {
            println!("Time limit reached at depth {}", depth - 1);
            break;
        }
        
        let depth_start = Instant::now();
        let mut moves = Vec::with_capacity(64);
        state.for_each_possible_move(|mv| moves.push(mv));
        
        if moves.is_empty() {
            break;
        }
        
        // Order moves based on previous iteration results
        ctx.order_moves(&mut moves, 0, state.current_player, best_move);
        
        let mut depth_best_move = None;
        let mut depth_best_score = if state.current_player == Player::Max { i32::MIN } else { i32::MAX };
        
        let mut moves_evaluated = 0;
        for mv in moves {
            if ctx.is_time_up() {
                break;
            }
            
            moves_evaluated += 1;
            state.make_move(mv);
            
            let score = minimax_advanced(
                state,
                depth - 1,
                i32::MIN,
                i32::MAX,
                state.current_player == Player::Min, // Opposite because we made a move
                tt,
                ctx,
            );
            
            state.undo_move(mv);
            
            if (state.current_player == Player::Max && score > depth_best_score) ||
               (state.current_player == Player::Min && score < depth_best_score) {
                depth_best_score = score;
                depth_best_move = Some(mv);
            }
        }
        
        // Only update best move if we completed the depth
        if !ctx.is_time_up() || moves_evaluated > 0 {
            best_move = depth_best_move;
            best_score = depth_best_score;
            depth_reached = depth;
        }
        
        let depth_time = depth_start.elapsed();
        println!("Depth {} completed: best_move={:?}, score={}, time={:?}", 
                 depth, best_move, best_score, depth_time);
        
        // STOP EARLY based on minimax scoring - let the heuristic decide when to stop
        if best_score.abs() >= 2_500_000 { // IMMEDIATE_THREAT_SCORE - found critical position
            println!("Critical position found (score: {}), stopping search at depth {}", best_score, depth);
            depth_reached = depth;
            break;
        }
        
        // Also stop for definitive wins
        if (state.current_player == Player::Max && best_score > 5_000_000) ||
           (state.current_player == Player::Min && best_score < -5_000_000) {
            println!("Winning move found, stopping search");
            depth_reached = depth;
            break;
        }
    }
    
    (best_move, depth_reached)
}

// Check for strategic threats (2-move lookahead to prevent opponent setup)
fn check_strategic_threats(state: &GameState) -> Option<(usize, usize)> {
    let opponent = match state.current_player {
        Player::Max => Player::Min,
        Player::Min => Player::Max,
    };
    
    // Get all possible moves for the opponent
    let mut state_copy = state.clone();
    let mut possible_moves = Vec::new();
    state_copy.for_each_possible_move(|mv| possible_moves.push(mv));
    
    for &mv in &possible_moves {
        // Simulate opponent move
        let mut test_state = state.clone();
        test_state.current_player = opponent;
        test_state.make_move(mv);
        
        // Check if this creates a situation where opponent can force multiple wins next move
        let mut follow_up_state = test_state.clone();
        let mut follow_up_moves = Vec::new();
        follow_up_state.for_each_possible_move(|next_mv| follow_up_moves.push(next_mv));
        
        let mut forcing_threats = 0;
        
        for &next_mv in &follow_up_moves {
            // Simulate the follow-up move
            let mut final_test = test_state.clone();
            final_test.make_move(next_mv);
            
            // Check if this creates a win
            if final_test.is_terminal() && final_test.check_winner() == Some(opponent) {
                forcing_threats += 1;
            } else {
                // Check if this creates a very strong threat using our heuristic
                let threat_value = Heuristic::evaluate(&final_test, 1);
                
                // If the position becomes very favorable for opponent, count as forcing threat
                let adjusted_value = if opponent == Player::Max { threat_value } else { -threat_value };
                if adjusted_value >= 25000 { // Very high threat threshold
                    forcing_threats += 1;
                }
            }
        }
        
        // If opponent can create 3+ forcing threats (not just 2), we must block the setup move
        if forcing_threats >= 3 {
            println!("Strategic threat: Opponent can create {} forcing threats after ({}, {})", forcing_threats, mv.0, mv.1);
            // Block the setup move
            return Some(mv);
        }
    }
    
    None
}

// Check for immediate threats that must be blocked (1-2 move lookahead)
fn check_immediate_threats(state: &GameState) -> Option<(usize, usize)> {
    let opponent = match state.current_player {
        Player::Max => Player::Min,
        Player::Min => Player::Max,
    };
    
    // Get all possible moves by cloning state temporarily
    let mut state_copy = state.clone();
    let mut possible_moves = Vec::new();
    state_copy.for_each_possible_move(|mv| possible_moves.push(mv));
    
    // PHASE 1: Check for immediate winning moves for ourselves (highest priority)
    for &mv in &possible_moves {
        let mut test_state = state.clone();
        test_state.make_move(mv);
        
        // Check if we win with this move
        if test_state.is_terminal() && test_state.check_winner() == Some(state.current_player) {
            return Some(mv); // Take the win immediately
        }
    }
    
    // PHASE 2: Check for immediate opponent wins that must be blocked
    let mut blocking_moves = Vec::new();
    for &mv in &possible_moves {
        // Simulate opponent playing this move
        let mut test_state = state.clone();
        test_state.current_player = opponent; // Switch to opponent
        test_state.make_move(mv);
        
        // Check if opponent wins with this move
        if test_state.is_terminal() && test_state.check_winner() == Some(opponent) {
            blocking_moves.push(mv);
        }
    }
    
    // If there are multiple winning threats, AI is already lost (opponent has unstoppable fork)
    // But still try to block one
    if !blocking_moves.is_empty() {
        return Some(blocking_moves[0]);
    }
    
    // PHASE 3: Check for opponent creating multiple threats (fork detection)
    let mut critical_blocks = Vec::new();
    for &mv in &possible_moves {
        let mut test_state = state.clone();
        test_state.current_player = opponent;
        test_state.make_move(mv);
        
        // Count how many ways opponent can win after this move
        let mut opponent_winning_moves = 0;
        let mut test_copy = test_state.clone();
        let mut next_moves = Vec::new();
        test_copy.for_each_possible_move(|next_mv| next_moves.push(next_mv));
        
        for &next_mv in &next_moves {
            let mut final_test = test_state.clone();
            final_test.make_move(next_mv);
            if final_test.is_terminal() && final_test.check_winner() == Some(opponent) {
                opponent_winning_moves += 1;
            }
        }
        
        // If opponent can create 2+ winning threats, this move must be blocked
        if opponent_winning_moves >= 2 {
            critical_blocks.push((mv, opponent_winning_moves));
        }
    }
    
    // Block the move that creates the most threats
    if !critical_blocks.is_empty() {
        critical_blocks.sort_by_key(|(_, threats)| -*threats);
        return Some(critical_blocks[0].0);
    }
    
    // PHASE 4: Emergency block ONLY immediate 4-in-a-row formation (not 3s)
    for &mv in &possible_moves {
        let mut test_state = state.clone();
        test_state.current_player = opponent;
        test_state.make_move(mv);
        
        // ONLY block if opponent forms exactly 4 in a row (deadly threat)
        if forms_line_of_length(&test_state, mv, opponent, 4) {
            println!("EMERGENCY: Blocking 4-in-a-row at ({}, {})", mv.0, mv.1);
            return Some(mv);
        }
    }
    
    // PHASE 5: Only block VERY high scoring threats (not every 3-in-a-row)
    for &mv in &possible_moves {
        let mut test_state = state.clone();
        test_state.current_player = opponent;
        test_state.make_move(mv);
        
        // Use heuristic but with MUCH higher threshold - only critical threats
        let threat_level = evaluate_threat_level(&test_state, opponent);
        if threat_level > 40000 { // Much higher threshold - only truly dangerous moves
            println!("Critical threat detected at ({}, {}) with level {}", mv.0, mv.1, threat_level);
            return Some(mv);
        }
    }
    
    None
}

// Evaluate how threatening a position is for a given player
fn evaluate_threat_level(state: &GameState, player: Player) -> i32 {
    // Count open 4s, open 3s, etc. that player can create
    let mut threat_score = 0;
    
    // Check all positions for patterns
    for row in 0..state.board.size() {
        for col in 0..state.board.size() {
            if state.board.is_empty_position(row, col) {
                // Simulate playing here
                let mut test_state = state.clone();
                test_state.current_player = player;
                test_state.make_move((row, col));
                
                // Count patterns formed
                threat_score += count_dangerous_patterns(&test_state, player, row, col);
            }
        }
    }
    
    threat_score
}

// Count dangerous patterns formed by a move
fn count_dangerous_patterns(state: &GameState, player: Player, row: usize, col: usize) -> i32 {
    let mut pattern_score = 0;
    
    // Check all 4 directions for patterns
    for &(dx, dy) in &[(1,0), (0,1), (1,1), (1,-1)] {
        let line_score = analyze_line_pattern(state, player, row, col, dx, dy);
        pattern_score += line_score;
    }
    
    pattern_score
}

// Check if a move forms a line of specific length
fn forms_line_of_length(state: &GameState, mv: (usize, usize), player: Player, target_length: usize) -> bool {
    let (row, col) = mv;
    
    // Check all 4 directions for exact length patterns
    for &(dx, dy) in &[(1,0), (0,1), (1,1), (1,-1)] {
        let mut count = 1; // Count the placed piece
        
        // Count in positive direction
        let mut r = row as isize + dx;
        let mut c = col as isize + dy;
        while r >= 0 && r < state.board.size() as isize && c >= 0 && c < state.board.size() as isize {
            if let Some(p) = state.board.get_player(r as usize, c as usize) {
                if p == player {
                    count += 1;
                    r += dx;
                    c += dy;
                } else {
                    break; // Blocked by opponent
                }
            } else {
                break; // Empty space
            }
        }
        
        // Count in negative direction
        r = row as isize - dx;
        c = col as isize - dy;
        while r >= 0 && r < state.board.size() as isize && c >= 0 && c < state.board.size() as isize {
            if let Some(p) = state.board.get_player(r as usize, c as usize) {
                if p == player {
                    count += 1;
                    r -= dx;
                    c -= dy;
                } else {
                    break; // Blocked by opponent
                }
            } else {
                break; // Empty space
            }
        }
        
        // Check if we reached the target length
        if count >= target_length {
            return true;
        }
    }
    
    false
}

// Check if a move forms a dangerous line (3+ stones) for the opponent
fn forms_dangerous_line(state: &GameState, mv: (usize, usize), player: Player) -> bool {
    let (row, col) = mv;
    
    // Check all 4 directions for dangerous patterns
    for &(dx, dy) in &[(1,0), (0,1), (1,1), (1,-1)] {
        let mut count = 1; // Count the placed piece
        
        // Count in positive direction
        let mut r = row as isize + dx;
        let mut c = col as isize + dy;
        while r >= 0 && r < state.board.size() as isize && c >= 0 && c < state.board.size() as isize {
            if let Some(p) = state.board.get_player(r as usize, c as usize) {
                if p == player {
                    count += 1;
                    r += dx;
                    c += dy;
                } else {
                    break; // Blocked by opponent
                }
            } else {
                break; // Empty space
            }
        }
        
        // Count in negative direction
        r = row as isize - dx;
        c = col as isize - dy;
        while r >= 0 && r < state.board.size() as isize && c >= 0 && c < state.board.size() as isize {
            if let Some(p) = state.board.get_player(r as usize, c as usize) {
                if p == player {
                    count += 1;
                    r -= dx;
                    c -= dy;
                } else {
                    break; // Blocked by opponent
                }
            } else {
                break; // Empty space
            }
        }
        
        // If opponent can form 3+ in a row, this is dangerous
        if count >= 3 {
            return true;
        }
    }
    
    false
}

// Analyze pattern strength in one direction
fn analyze_line_pattern(state: &GameState, player: Player, row: usize, col: usize, dx: isize, dy: isize) -> i32 {
    let mut count = 1; // Count the placed piece
    let mut open_ends = 0;
    
    // Count in positive direction
    let mut r = row as isize + dx;
    let mut c = col as isize + dy;
    while r >= 0 && r < state.board.size() as isize && c >= 0 && c < state.board.size() as isize {
        if let Some(p) = state.board.get_player(r as usize, c as usize) {
            if p == player {
                count += 1;
                r += dx;
                c += dy;
            } else {
                break; // Blocked by opponent
            }
        } else {
            open_ends += 1;
            break; // Found empty space
        }
    }
    
    // Count in negative direction
    r = row as isize - dx;
    c = col as isize - dy;
    while r >= 0 && r < state.board.size() as isize && c >= 0 && c < state.board.size() as isize {
        if let Some(p) = state.board.get_player(r as usize, c as usize) {
            if p == player {
                count += 1;
                r -= dx;
                c -= dy;
            } else {
                break; // Blocked by opponent
            }
        } else {
            open_ends += 1;
            break; // Found empty space
        }
    }
    
    // Score based on count and openness
    match count {
        5 => 100000, // Five in a row
        4 => if open_ends >= 1 { 50000 } else { 5000 },  // Open 4 vs blocked 4 - HIGHER blocked 4 score
        3 => if open_ends >= 2 { 15000 } else if open_ends == 1 { 5000 } else { 1000 }, // More aggressive 3-scoring
        2 => if open_ends >= 2 { 2000 } else { 500 },      // Higher 2-in-row scoring
        _ => 0,
    }
}
