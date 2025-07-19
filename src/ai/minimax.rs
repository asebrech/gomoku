use crate::core::state::GameState;
use std::cmp::{max, min};
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

use super::{heuristic::Heuristic, transposition::TranspositionTable};

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
    
    // Check transposition table first
    let tt_start = Instant::now();
    let cached_result = tt.lookup_with_hash(state.zobrist_hash);
    TT_TIME.fetch_add(tt_start.elapsed().as_micros() as u64, Ordering::Relaxed);
    
    if let Some(cached_value) = cached_result {
        TT_HITS.fetch_add(1, Ordering::Relaxed);
        return cached_value;
    }

    if depth == 0 || state.is_terminal() {
        let heur_start = Instant::now();
        let eval = Heuristic::evaluate(state, depth);
        HEURISTIC_TIME.fetch_add(heur_start.elapsed().as_micros() as u64, Ordering::Relaxed);
        HEURISTIC_CALLS.fetch_add(1, Ordering::Relaxed);
        
        let tt_store_start = Instant::now();
        tt.store_with_hash(state.zobrist_hash, eval);
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
    
    let eval = if maximizing_player {
        let mut value = i32::MIN;
        
        // Try center move first if available (good heuristic for opening)
        if moves.contains(&center_move) {
            state.make_move(center_move);
            value = max(value, minimax(state, depth - 1, alpha, beta, false, tt));
            state.undo_move(center_move);
            if value >= beta {
                tt.store_with_hash(state.zobrist_hash, value);
                return value;
            }
            alpha = max(alpha, value);
        }
        
        // Try all other moves without expensive ordering
        for &move_ in &moves {
            if move_ != center_move { // Skip center as we already tried it
                state.make_move(move_);
                value = max(value, minimax(state, depth - 1, alpha, beta, false, tt));
                state.undo_move(move_);
                if value >= beta {
                    break; // Alpha-beta pruning
                }
                alpha = max(alpha, value);
            }
        }
        value
    } else {
        let mut value = i32::MAX;
        
        // Try center move first if available
        if moves.contains(&center_move) {
            state.make_move(center_move);
            value = min(value, minimax(state, depth - 1, alpha, beta, true, tt));
            state.undo_move(center_move);
            if value <= alpha {
                tt.store_with_hash(state.zobrist_hash, value);
                return value;
            }
            beta = min(beta, value);
        }
        
        // Try all other moves without expensive ordering
        for &move_ in &moves {
            if move_ != center_move { // Skip center as we already tried it
                state.make_move(move_);
                value = min(value, minimax(state, depth - 1, alpha, beta, true, tt));
                state.undo_move(move_);
                if value <= alpha {
                    break; // Alpha-beta pruning
                }
                beta = min(beta, value);
            }
        }
        value
    };
    
    let tt_store_start = Instant::now();
    tt.store_with_hash(state.zobrist_hash, eval);
    TT_TIME.fetch_add(tt_store_start.elapsed().as_micros() as u64, Ordering::Relaxed);
    eval
}
