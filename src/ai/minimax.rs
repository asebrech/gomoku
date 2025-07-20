use crate::core::state::GameState;
use std::cmp::{max, min};
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

use super::{heuristic::Heuristic, transposition::{TranspositionTable, BoundType}};

// Atomic counters for profiling (thread-safe)
static MINIMAX_CALLS: AtomicUsize = AtomicUsize::new(0);
static TT_HITS: AtomicUsize = AtomicUsize::new(0);
static HEURISTIC_CALLS: AtomicUsize = AtomicUsize::new(0);
static MOVE_GENERATION_TIME: AtomicU64 = AtomicU64::new(0);
static HEURISTIC_TIME: AtomicU64 = AtomicU64::new(0);
static TT_TIME: AtomicU64 = AtomicU64::new(0);

pub fn reset_profiling() {
    MINIMAX_CALLS.store(0, Ordering::Relaxed);
    TT_HITS.store(0, Ordering::Relaxed);
    HEURISTIC_CALLS.store(0, Ordering::Relaxed);
    MOVE_GENERATION_TIME.store(0, Ordering::Relaxed);
    HEURISTIC_TIME.store(0, Ordering::Relaxed);
    TT_TIME.store(0, Ordering::Relaxed);
}

pub fn print_profiling() {
    let minimax_calls = MINIMAX_CALLS.load(Ordering::Relaxed);
    let tt_hits = TT_HITS.load(Ordering::Relaxed);
    let heuristic_calls = HEURISTIC_CALLS.load(Ordering::Relaxed);
    let move_gen_time = MOVE_GENERATION_TIME.load(Ordering::Relaxed);
    let heur_time = HEURISTIC_TIME.load(Ordering::Relaxed);
    let tt_time = TT_TIME.load(Ordering::Relaxed);
    
    println!("=== MINIMAX PROFILING ===");
    println!("Minimax calls: {}", minimax_calls);
    println!("Transposition table hits: {} ({}%)", tt_hits, if minimax_calls > 0 { tt_hits * 100 / minimax_calls } else { 0 });
    println!("Heuristic evaluations: {}", heuristic_calls);
    println!("Move generation time: {}μs", move_gen_time);
    println!("Heuristic evaluation time: {}μs", heur_time);
    println!("Transposition table time: {}μs", tt_time);
    println!("========================");
}

pub fn minimax(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &mut TranspositionTable,
) -> i32 {
    MINIMAX_CALLS.fetch_add(1, Ordering::Relaxed);
    
    // Check transposition table first with enhanced lookup
    let tt_start = Instant::now();
    let cached_result = tt.lookup_enhanced(state.zobrist_hash, depth, alpha, beta);
    TT_TIME.fetch_add(tt_start.elapsed().as_micros() as u64, Ordering::Relaxed);
    
    if let Some((cached_value, _best_move)) = cached_result {
        TT_HITS.fetch_add(1, Ordering::Relaxed);
        return cached_value;
    }

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

    // Get values we need before borrowing
    let center = state.board.size() / 2;
    let center_move = (center, center);
    
    // Collect moves to avoid borrow checker issues (this is the minimal allocation we need)
    let move_gen_start = Instant::now();
    let mut moves = Vec::with_capacity(64);
    state.for_each_possible_move(|mv| moves.push(mv));
    MOVE_GENERATION_TIME.fetch_add(move_gen_start.elapsed().as_micros() as u64, Ordering::Relaxed);
    
    // Check if we have a best move from TT for move ordering
    if let Some((_score, Some(tt_best_move))) = cached_result {
        if moves.contains(&tt_best_move) {
            // Move the TT best move to front for better pruning
            moves.retain(|&mv| mv != tt_best_move);
            moves.insert(0, tt_best_move);
        }
    }
    
    let original_alpha = alpha;
    let original_beta = beta;
    let mut best_move = None;
    
    let eval = if maximizing_player {
        let mut value = i32::MIN;
        
        // Try center move first if available and not already prioritized by TT
        if moves.contains(&center_move) && moves[0] != center_move {
            state.make_move(center_move);
            let move_value = minimax(state, depth - 1, alpha, beta, false, tt);
            state.undo_move(center_move);
            if move_value > value {
                value = move_value;
                best_move = Some(center_move);
            }
            if value >= beta {
                let tt_store_start = Instant::now();
                tt.store_enhanced(state.zobrist_hash, value, depth, BoundType::LowerBound, best_move);
                TT_TIME.fetch_add(tt_store_start.elapsed().as_micros() as u64, Ordering::Relaxed);
                return value;
            }
            alpha = max(alpha, value);
        }
        
        // Try all other moves
        for &move_ in &moves {
            if move_ != center_move { // Skip center as we already tried it
                state.make_move(move_);
                let move_value = minimax(state, depth - 1, alpha, beta, false, tt);
                state.undo_move(move_);
                if move_value > value {
                    value = move_value;
                    best_move = Some(move_);
                }
                if value >= beta {
                    break; // Alpha-beta pruning
                }
                alpha = max(alpha, value);
            }
        }
        value
    } else {
        let mut value = i32::MAX;
        
        // Try center move first if available and not already prioritized by TT
        if moves.contains(&center_move) && moves[0] != center_move {
            state.make_move(center_move);
            let move_value = minimax(state, depth - 1, alpha, beta, true, tt);
            state.undo_move(center_move);
            if move_value < value {
                value = move_value;
                best_move = Some(center_move);
            }
            if value <= alpha {
                let tt_store_start = Instant::now();
                tt.store_enhanced(state.zobrist_hash, value, depth, BoundType::UpperBound, best_move);
                TT_TIME.fetch_add(tt_store_start.elapsed().as_micros() as u64, Ordering::Relaxed);
                return value;
            }
            beta = min(beta, value);
        }
        
        // Try all other moves
        for &move_ in &moves {
            if move_ != center_move { // Skip center as we already tried it
                state.make_move(move_);
                let move_value = minimax(state, depth - 1, alpha, beta, true, tt);
                state.undo_move(move_);
                if move_value < value {
                    value = move_value;
                    best_move = Some(move_);
                }
                if value <= alpha {
                    break; // Alpha-beta pruning
                }
                beta = min(beta, value);
            }
        }
        value
    };
    
    // Determine bound type based on how search ended
    let bound_type = if eval <= original_alpha {
        BoundType::UpperBound  // Failed low
    } else if eval >= original_beta {
        BoundType::LowerBound  // Failed high  
    } else {
        BoundType::Exact       // Exact value
    };
    
    let tt_store_start = Instant::now();
    tt.store_enhanced(state.zobrist_hash, eval, depth, bound_type, best_move);
    TT_TIME.fetch_add(tt_store_start.elapsed().as_micros() as u64, Ordering::Relaxed);
    eval
}
