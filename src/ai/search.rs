use crate::core::state::GameState;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering}};
use rayon::prelude::*;

use super::{minimax::minimax, move_ordering::MoveOrdering, transposition::TranspositionTable};

#[derive(Debug)]
pub struct SearchResult {
    pub best_move: Option<(usize, usize)>,
    pub score: i32,
    pub depth_reached: i32,
    pub nodes_searched: u64,
    pub time_elapsed: Duration,
}

// Shared state for lazy SMP coordination
struct LazySMPState {
    best_move: Mutex<Option<(usize, usize)>>,
    best_score: AtomicI32,
    should_stop: AtomicBool,
    nodes_searched: AtomicU64,
    max_depth_reached: AtomicI32,
}

pub fn find_best_move(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &TranspositionTable,
) -> SearchResult {
    let start_time = Instant::now();
    let is_maximizing = state.current_player == crate::core::board::Player::Max;
    
    tt.advance_age();

    let initial_moves = state.get_possible_moves();
    if initial_moves.is_empty() {
        return SearchResult {
            best_move: None,
            score: 0,
            depth_reached: 0,
            nodes_searched: 0,
            time_elapsed: start_time.elapsed(),
        };
    }

    // Shared state for lazy SMP coordination
    let shared_state = Arc::new(LazySMPState {
        best_move: Mutex::new(None),
        best_score: AtomicI32::new(if is_maximizing { -1_000_000 } else { 1_000_000 }),
        should_stop: AtomicBool::new(false),
        nodes_searched: AtomicU64::new(0),
        max_depth_reached: AtomicI32::new(0),
    });

    // Determine number of threads - maximize for deep search
    let num_threads = rayon::current_num_threads().min(12).max(8); // Use 8-12 threads for maximum depth coverage
    
    // Launch parallel search threads with different parameters
    let results: Vec<_> = (0..num_threads).into_par_iter().map(|thread_id| {
        lazy_smp_thread_search(
            state, 
            max_depth, 
            thread_id, 
            num_threads,
            &shared_state, 
            &start_time, 
            time_limit, 
            tt,
            is_maximizing
        )
    }).collect();

    // Collect results from all threads
    let total_nodes: u64 = results.iter().map(|r| r.nodes_searched).sum();
    let max_depth_reached = results.iter().map(|r| r.depth_reached).max().unwrap_or(0);
    
    let final_best_move = shared_state.best_move.lock().unwrap().clone();
    let final_score = shared_state.best_score.load(Ordering::Relaxed);

    SearchResult {
        best_move: final_best_move,
        score: final_score,
        depth_reached: max_depth_reached,
        nodes_searched: total_nodes,
        time_elapsed: start_time.elapsed(),
    }
}

// Individual thread search with diversification
fn lazy_smp_thread_search(
    state: &GameState,
    max_depth: i32,
    thread_id: usize,
    _num_threads: usize,
    shared_state: &Arc<LazySMPState>,
    start_time: &Instant,
    time_limit: Option<Duration>,
    tt: &TranspositionTable,
    is_maximizing: bool,
) -> SearchResult {
    let mut local_state = state.clone();
    let mut nodes_searched = 0u64;
    let mut depth_reached = 0;
    
    // Thread diversification strategies for maximum depth
    let depth_offset = match thread_id {
        0 => 0,                    // Main thread uses requested depth
        1 => 2,                    // Thread 1 goes 2 deeper
        2 => 4,                    // Thread 2 goes 4 deeper  
        3 => 1,                    // Thread 3 goes 1 deeper
        4 => 3,                    // Thread 4 goes 3 deeper
        5 => 6,                    // Thread 5 goes 6 deeper
        6 => 8,                    // Thread 6 goes 8 deeper
        7 => 10,                   // Thread 7 goes 10 deeper
        _ => (thread_id % 8) as i32 + 1, // Other threads cycle with offsets
    };
    let effective_max_depth = max_depth + depth_offset; // No minimum limit - go as deep as possible
    
    for depth in 1..=effective_max_depth {
        // Check if we should stop (time limit or another thread found solution)
        if shared_state.should_stop.load(Ordering::Relaxed) {
            break;
        }
        
        if let Some(limit) = time_limit {
            // Allow deeper search threads more time - they might find better solutions
            let time_extension = if depth_offset > 4 { 
                // Threads searching much deeper get significantly more time
                Duration::from_millis((depth_offset as u64 * 200).min(2000)) 
            } else if depth_offset > 0 {
                // Threads going moderately deeper get some extra time
                Duration::from_millis(depth_offset as u64 * 100)
            } else {
                Duration::from_millis(0) // Main thread sticks to original limit
            };
            
            if start_time.elapsed() >= limit + time_extension {
                // Only main thread can signal global stop for time limit
                if thread_id == 0 {
                    shared_state.should_stop.store(true, Ordering::Relaxed);
                }
                break;
            }
        }

        let mut moves = local_state.get_possible_moves();
        if moves.is_empty() {
            break;
        }

        // Diversify move ordering per thread
        MoveOrdering::order_moves(&local_state, &mut moves);
        
        // Thread 0 uses standard ordering, others use variations
        // Only diversify if we have enough moves to make it worthwhile
        if thread_id > 0 && moves.len() > 4 {
            // Rotate moves based on thread_id to explore different parts first
            let rotation = thread_id % moves.len();
            moves.rotate_left(rotation);
        }

        // Try to get previous best move from shared state for move ordering
        if let Ok(shared_best) = shared_state.best_move.lock() {
            if let Some(prev_best) = *shared_best {
                if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                    moves.swap(0, pos);
                }
            }
        }

        let mut _best_move_this_depth = None;
        let mut best_score_this_depth = if is_maximizing { -1_000_000 } else { 1_000_000 };

        for &mv in &moves {
            if shared_state.should_stop.load(Ordering::Relaxed) {
                break;
            }

            local_state.make_move(mv);
            
            // Optimized aspiration windows for deep search threads
            let (alpha, beta) = if thread_id == 0 {
                (i32::MIN, i32::MAX) // Full window for main thread
            } else {
                let current_best = shared_state.best_score.load(Ordering::Relaxed);
                
                // Deeper search threads get wider windows to avoid re-searches
                let base_window = if depth_offset >= 6 {
                    200 // Very wide windows for deepest threads (6+ deeper)
                } else if depth_offset >= 3 {
                    100 // Wide windows for moderately deep threads (3-5 deeper) 
                } else {
                    50  // Standard windows for shallow helper threads
                };
                
                let window = base_window + thread_id as i32 * 15;
                let alpha = current_best.saturating_sub(window);
                let beta = current_best.saturating_add(window);
                (alpha, beta)
            };
            
            let (score, child_nodes) = minimax(
                &mut local_state,
                depth - 1,
                alpha,
                beta,
                !is_maximizing,
                tt,
                start_time,
                time_limit,
            );
            
            local_state.undo_move(mv);
            nodes_searched += child_nodes;

            // Always ensure we have at least one move stored (first valid move for this thread)
            if thread_id == 0 && _best_move_this_depth.is_none() {
                _best_move_this_depth = Some(mv);
                best_score_this_depth = score;
                if let Ok(mut best_move) = shared_state.best_move.lock() {
                    if best_move.is_none() {
                        *best_move = Some(mv);
                        shared_state.best_score.store(score, Ordering::Relaxed);
                    }
                }
            }

            // Update shared state if this is a better move
            let is_better = if is_maximizing {
                score > best_score_this_depth
            } else {
                score < best_score_this_depth
            };

            if is_better {
                best_score_this_depth = score;
                _best_move_this_depth = Some(mv);
                
                // Update global best if this is better
                let current_best = shared_state.best_score.load(Ordering::Relaxed);
                let should_update = if is_maximizing {
                    score > current_best
                } else {
                    score < current_best
                };
                
                if should_update {
                    shared_state.best_score.store(score, Ordering::Relaxed);
                    if let Ok(mut best_move) = shared_state.best_move.lock() {
                        *best_move = Some(mv);
                    }
                    
                    // If we found a winning/losing position, signal other threads to stop
                    if score.abs() >= 1_000_000 {
                        shared_state.should_stop.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        }

        depth_reached = depth;
        shared_state.max_depth_reached.store(depth, Ordering::Relaxed);

        // Early termination for definitive results - but allow deeper threads to continue
        if best_score_this_depth.abs() >= 1_000_000 {
            // Only stop immediately if this is a shallow thread or main thread
            if thread_id == 0 || depth_offset <= 2 {
                shared_state.should_stop.store(true, Ordering::Relaxed);
                break;
            } else {
                // Deep search threads continue briefly to potentially find even better solutions
                // But if multiple threads have found wins, then stop
                let existing_stop = shared_state.should_stop.load(Ordering::Relaxed);
                if existing_stop {
                    break; // Another thread already signaled to stop
                }
                // Continue searching for a bit more to verify the solution depth
            }
        }
    }

    shared_state.nodes_searched.fetch_add(nodes_searched, Ordering::Relaxed);

    SearchResult {
        best_move: None, // Individual thread doesn't return best move
        score: 0,
        depth_reached,
        nodes_searched,
        time_elapsed: start_time.elapsed(),
    }
}
