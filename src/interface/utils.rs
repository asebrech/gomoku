use crate::ai::{zobrist::ZobristHash, transposition_table::{TranspositionTable, EntryType}};
use crate::core::state::GameState;
use crate::core::board::Player;
use std::cmp::{max, min};
use std::sync::{OnceLock, Mutex, LazyLock};
use std::collections::HashMap;
use rayon::prelude::*;

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
    
    // Get the correct Zobrist hash for this board size
    let zobrist = get_zobrist(state.board.size);
    
    let tt = match TRANSPOSITION_TABLE.get() {
        Some(t) => t,
        None => {
            println!("⚠️  TT not initialized, falling back to sequential");
            let result = find_best_move_sequential(state, depth);
            let elapsed = start_time.elapsed();
            println!("⏱️  AI search time: {:?}", elapsed);
            return result;
        }
    };
    
    let moves = state.get_possible_moves();
    if moves.is_empty() {
        return None;
    }
    
    println!("🧠 Using REAL minimax with TT and {} moves", moves.len());
    
    // Check thread count
    let thread_count = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    println!("💻 Available CPU cores: {}", thread_count);
    
    // Advance age for new search
    tt.advance_age();
    
    let current_player = state.current_player;
    let initial_hash = zobrist.compute_hash(state);
    
    // Use parallel search at root level for better performance
    println!("🚀 Starting parallel root search with {} threads", 
             std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));
    
    let result = moves.into_par_iter().map(|mv| {
        let mut state_copy = state.clone();
        state_copy.make_move(mv);
        
        let new_hash = zobrist.update_hash_make_move(initial_hash, mv.0, mv.1, current_player);
        
        let score = minimax(
            &mut state_copy,
            depth - 1,
            i32::MIN + 1,
            i32::MAX - 1,
            current_player == Player::Min,
            new_hash,
            &zobrist,
            tt,
        );
        
        println!("📊 Thread {:?} evaluated move ({}, {}) = {}", 
                 std::thread::current().id(), mv.0, mv.1, score);
        
        (mv, score)
    }).max_by_key(|&(_, score)| {
        if current_player == Player::Max {
            score
        } else {
            -score // For Min player, we want the minimum score
        }
    });
    
    let best_move = result.map(|(mv, _)| mv);
    
    let elapsed = start_time.elapsed();
    println!("⏱️  AI search time: {:?}", elapsed);
    
    if let Some((row, col)) = best_move {
        println!("🎯 Best move: ({}, {})", row, col);
    }
    
    // Print TT stats
    let (tt_size, hit_rate, collisions) = get_tt_stats();
    if tt_size > 0 {
        println!("📊 TT Stats - Size: {}, Hit Rate: {:.1}%, Collisions: {}", 
                tt_size, hit_rate * 100.0, collisions);
    }
    
    best_move
}

/// Fallback sequential version of find_best_move  
fn find_best_move_sequential(state: &mut GameState, depth: i32) -> Option<(usize, usize)> {
    let moves = state.get_possible_moves();
    println!("📝 Evaluating {} possible moves", moves.len());
    
    if moves.is_empty() {
        return None;
    }
    
    let current_player = state.current_player;
    let mut best_move = None;
    let mut best_score = if current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };

    for mv in moves {
        let mut state_copy = state.clone();
        state_copy.make_move(mv);
        
        // Use simple evaluation for fallback
        let score = crate::ai::heuristic::Heuristic::evaluate(&state_copy, depth);

        if (current_player == Player::Max && score > best_score)
            || (current_player == Player::Min && score < best_score)
        {
            best_score = score;
            best_move = Some(mv);
        }
    }

    best_move
}

/// Enhanced minimax with transposition table support
fn minimax(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    hash: u64,
    zobrist: &ZobristHash,
    tt: &TranspositionTable,
) -> i32 {
    // Check transposition table first
    let tt_result = tt.probe(hash, depth, alpha, beta);
    if tt_result.cutoff {
        if let Some(value) = tt_result.value {
            return value;
        }
    }
    
    // Terminal node check
    if depth == 0 || state.is_terminal() {
        let eval = crate::ai::heuristic::Heuristic::evaluate(state, depth);
        tt.store(hash, eval, depth, EntryType::Exact, None);
        return eval;
    }
    
    // Get possible moves
    let mut moves = state.get_possible_moves();
    if moves.is_empty() {
        let eval = crate::ai::heuristic::Heuristic::evaluate(state, depth);
        tt.store(hash, eval, depth, EntryType::Exact, None);
        return eval;
    }
    
    // Move ordering with TT hint
    crate::ai::move_ordering::MoveOrdering::order_moves(state, &mut moves);
    if let Some(tt_move) = tt_result.best_move {
        if let Some(pos) = moves.iter().position(|&m| m == tt_move) {
            moves.swap(0, pos);
        }
    }
    
    let original_alpha = alpha;
    let mut best_move = None;
    let mut best_value = if maximizing_player { i32::MIN } else { i32::MAX };
    
    for mv in moves {
        // Calculate new hash using real board state
        let new_hash = zobrist.update_hash_make_move(hash, mv.0, mv.1, state.current_player);
        
        let mut state_copy = state.clone();
        state_copy.make_move(mv);
        let value = minimax(
            &mut state_copy, 
            depth - 1, 
            alpha, 
            beta, 
            !maximizing_player, 
            new_hash,
            zobrist,
            tt
        );
        
        if maximizing_player {
            if value > best_value {
                best_value = value;
                best_move = Some(mv);
            }
            alpha = max(alpha, value);
            if beta <= alpha {
                break; // Beta cutoff
            }
        } else {
            if value < best_value {
                best_value = value;
                best_move = Some(mv);
            }
            beta = min(beta, value);
            if beta <= alpha {
                break; // Alpha cutoff
            }
        }
    }
    
    // Store in transposition table
    let entry_type = if best_value <= original_alpha {
        EntryType::UpperBound
    } else if best_value >= beta {
        EntryType::LowerBound
    } else {
        EntryType::Exact
    };
    
    tt.store(hash, best_value, depth, entry_type, best_move);
    best_value
}
