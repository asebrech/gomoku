use crate::core::state::GameState;
use crate::ai::minimax_tt::MinimaxWithTT;
use std::time::{Duration, Instant};

/// Result of an iterative deepening search
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<(usize, usize)>,
    pub best_score: i32,
    pub depth_reached: i32,
    pub nodes_evaluated: u64,
    pub time_elapsed: Duration,
}

/// Configuration for iterative deepening search
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub max_depth: i32,
    pub max_time: Option<Duration>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_depth: 6,
            max_time: Some(Duration::from_secs(5)),
        }
    }
}

/// Simple Iterative Deepening Engine
/// 
/// Based on the classic iterative deepening algorithm:
/// 1. Start with depth 1
/// 2. Increment depth and search again
/// 3. Always keep the last completed search result as fallback
/// 4. Stop when time is exhausted or max depth reached
pub struct IterativeDeepeningEngine {
    minimax_engine: MinimaxWithTT,
}

impl IterativeDeepeningEngine {
    /// Create a new iterative deepening engine
    pub fn new(board_size: usize) -> Self {
        Self {
            minimax_engine: MinimaxWithTT::new(board_size),
        }
    }
    
    /// Create a new iterative deepening engine with custom table size
    pub fn new_with_table_size(board_size: usize, table_size: usize) -> Self {
        Self {
            minimax_engine: MinimaxWithTT::new_with_table_size(board_size, table_size),
        }
    }
    
    /// Main iterative deepening search function
    /// 
    /// This implements the classic iterative deepening algorithm:
    /// - Start with depth 1 and increment
    /// - Always keep the result from the last completed iteration
    /// - Use existing minimax function for each depth search
    pub fn search(&mut self, state: &GameState, config: SearchConfig) -> SearchResult {
        let start_time = Instant::now();
        
        let moves = state.get_possible_moves();
        if moves.is_empty() {
            return SearchResult {
                best_move: None,
                best_score: 0,
                depth_reached: 0,
                nodes_evaluated: 0,
                time_elapsed: start_time.elapsed(),
            };
        }
        
        // Initialize with a fallback move (first available move)
        let mut best_move = moves[0];
        let mut best_score = 0;
        let mut depth_reached = 0;
        let mut total_nodes = 0;
        
        println!("🔄 Starting iterative deepening search (max depth: {})", config.max_depth);
        
        // Iterative deepening loop - the core algorithm
        for depth in 1..=config.max_depth {
            // Check time limit before starting new iteration
            if let Some(max_time) = config.max_time {
                if start_time.elapsed() >= max_time {
                    println!("⏰ Time limit reached before depth {}", depth);
                    break;
                }
            }
            
            println!("🔍 Searching at depth {}...", depth);
            let iteration_start = Instant::now();
            
            // Use minimax engine directly for this depth
            let mut state_copy = state.clone();
            let move_result = self.minimax_engine.find_best_move(&mut state_copy, depth);
            
            // Always update our result if we got a valid move
            if let Some(mv) = move_result {
                best_move = mv;
                // For now, we don't have access to the score from find_best_move
                // This is a limitation of the simple approach, but it still works
                best_score = 0; // Placeholder score
                depth_reached = depth;
                
                let iteration_time = iteration_start.elapsed();
                println!("✅ Depth {} completed: move ({}, {}), time {:?}", 
                         depth, mv.0, mv.1, iteration_time);
            } else {
                println!("⚠️ No valid move found at depth {}", depth);
                break;
            }
            
            // Estimate nodes (rough approximation)
            total_nodes += (moves.len() as u64).pow(depth as u32);
        }
        
        println!("🏁 Iterative deepening completed: depth {}, best move ({}, {}), score {}", 
                 depth_reached, best_move.0, best_move.1, best_score);
        
        SearchResult {
            best_move: Some(best_move),
            best_score,
            depth_reached,
            nodes_evaluated: total_nodes,
            time_elapsed: start_time.elapsed(),
        }
    }
    
    /// Get engine statistics (simplified for new implementation)
    pub fn get_stats(&self) -> (usize, f64, u64, u64) {
        // Return stats from the minimax engine
        let (tt_size, hit_rate, collisions) = self.minimax_engine.get_tt_stats();
        (tt_size, hit_rate, collisions, self.minimax_engine.nodes_searched)
    }
    
    /// Clear the transposition table
    pub fn clear_tt(&mut self) {
        self.minimax_engine.clear_table();
    }
    
    /// Get the zobrist hash for a position
    pub fn get_hash(&self, state: &GameState) -> u64 {
        self.minimax_engine.get_hash(state)
    }
}
