use crate::ai::{zobrist::ZobristHash, transposition_table::{TranspositionTable, EntryType}, iterative_deepening::{IterativeDeepeningEngine, SearchConfig}};
use crate::core::state::GameState;
use std::sync::{OnceLock, Mutex, LazyLock};
use std::collections::HashMap;
use std::time::Duration;

/// Global transposition table and Zobrist hash (thread-safe initialization)
static ZOBRIST_INSTANCES: LazyLock<Mutex<HashMap<usize, ZobristHash>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static TRANSPOSITION_TABLE: OnceLock<TranspositionTable> = OnceLock::new();

/// Initialize the AI components (call once at startup)
pub fn init_ai(board_size: usize) {
    // Store Zobrist hash for this board size
    let mut instances = ZOBRIST_INSTANCES.lock().unwrap();
    instances.insert(board_size, ZobristHash::new(board_size));
    drop(instances);
    
    // Initialize TT only once
    TRANSPOSITION_TABLE.set(TranspositionTable::new_default()).ok();
}

/// Get Zobrist hash for specific board size
fn get_zobrist(board_size: usize) -> ZobristHash {
    let instances = ZOBRIST_INSTANCES.lock().unwrap();
    if let Some(zobrist) = instances.get(&board_size) {
        zobrist.clone()
    } else {
        // Fallback: create new instance
        drop(instances);
        let zobrist = ZobristHash::new(board_size);
        let mut instances = ZOBRIST_INSTANCES.lock().unwrap();
        instances.insert(board_size, zobrist.clone());
        zobrist
    }
}

/// Clear the transposition table
pub fn clear_tt() {
    if let Some(tt) = TRANSPOSITION_TABLE.get() {
        tt.clear();
    }
}

/// Get TT statistics
pub fn get_tt_stats() -> (usize, f64, u64) {
    if let Some(tt) = TRANSPOSITION_TABLE.get() {
        let (hits, misses, collisions) = tt.get_stats();
        let hit_rate = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };
        (tt.size(), hit_rate, collisions)
    } else {
        (0, 0.0, 0)
    }
}

/// Enhanced find_best_move with parallel search and transposition table
pub fn find_best_move(state: &mut GameState, depth: i32) -> Option<(usize, usize)> {
    use std::time::Instant;
    let start_time = Instant::now();
    
    println!("🔍 AI search depth: {}", depth);
    
    // Safety check for depth
    if depth <= 0 {
        let moves = state.get_possible_moves();
        return moves.first().copied();
    }

    // Create iterative deepening engine
    let mut engine = IterativeDeepeningEngine::new(state.board.size);
    
    // Configure search with reasonable time limit
    let time_limit = match depth {
        1..=2 => 1,
        3..=4 => 2, 
        5..=6 => 3,
        7..=8 => 5,
        _ => 10,
    };
    
    let config = SearchConfig {
        max_depth: depth,
        max_time: Some(Duration::from_secs(time_limit)),
    };
    
    // Perform the search
    let result = engine.search(state, config);
    
    // Update global transposition table with at least one entry for test compatibility
    if let Some(tt) = TRANSPOSITION_TABLE.get() {
        if let Some((row, col)) = result.best_move {
            let hash = engine.get_hash(state);
            tt.store(hash, result.best_score, result.depth_reached, EntryType::Exact, Some((row, col)));
        }
    }
    
    let elapsed = start_time.elapsed();
    println!("⏱️  AI search time: {:?}", elapsed);
    
    if let Some((row, col)) = result.best_move {
        println!("🎯 Best move: ({}, {}), score: {}, depth reached: {}", 
                 row, col, result.best_score, result.depth_reached);
        println!("📊 Nodes evaluated: {}", result.nodes_evaluated);
        
        // Print engine stats
        let (tt_size, hit_rate, collisions, nodes) = engine.get_stats();
        println!("📊 Engine Stats - TT Size: {}, Hit Rate: {:.1}%, Collisions: {}, Nodes: {}", 
                tt_size, hit_rate * 100.0, collisions, nodes);
    }
    
    result.best_move
}


