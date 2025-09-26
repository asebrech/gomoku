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

struct LazySMPState {
    best_move: Mutex<Option<(usize, usize)>>,
    best_score: AtomicI32,
    should_stop: AtomicBool,
    nodes_searched: AtomicU64,
    max_depth_reached: AtomicI32,
}

pub fn find_best_move(
    state: &mut GameState,
    tt: &TranspositionTable,
) -> SearchResult {
    let start_time = Instant::now();
    let is_maximizing = state.current_player == crate::core::board::Player::Max;
    
    // Hardcoded time limit - 500ms is mandatory for consistent AI performance
    let time_limit = Some(Duration::from_millis(500));
    
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

    let fallback_move = initial_moves.first().copied();
    
    let shared_state = Arc::new(LazySMPState {
        best_move: Mutex::new(fallback_move),
        best_score: AtomicI32::new(if is_maximizing { -1_000_000 } else { 1_000_000 }),
        should_stop: AtomicBool::new(false),
        nodes_searched: AtomicU64::new(0),
        max_depth_reached: AtomicI32::new(0),
    });

    let num_threads = match rayon::current_num_threads() {
        1..=2 => 2,
        3..=4 => 3,
        5..=6 => 4,
        7..=8 => 6,
        9..=10 => 10,
        11..=12 => 12,
        13..=16 => 14,
        _ => 16,
    };
    
    let results: Vec<_> = (0..num_threads).into_par_iter().map(|thread_id| {
        lazy_smp_thread_search(
            state, 
            thread_id,
            &shared_state, 
            &start_time, 
            time_limit, 
            tt,
            is_maximizing
        )
    }).collect();

    let total_nodes: u64 = results.iter().map(|r| r.nodes_searched).sum();
    let max_depth_reached = shared_state.max_depth_reached.load(Ordering::Relaxed).max(
        results.iter().map(|r| r.depth_reached).max().unwrap_or(0)
    );
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
    thread_id: usize,
    shared_state: &Arc<LazySMPState>,
    start_time: &Instant,
    time_limit: Option<Duration>,
    tt: &TranspositionTable,
    is_maximizing: bool,
) -> SearchResult {
    let mut local_state = state.clone();
    let mut nodes_searched = 0u64;
    let mut depth_reached = 0;
    
    let depth_offset = match thread_id {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 0,
        4 => 3,
        5 => 1,
        _ => (thread_id % 4) as i32,
    };
    
    let effective_max_depth = 25i32;
    let starting_depth = 1 + depth_offset;
    
    for depth in starting_depth..=effective_max_depth {
        if shared_state.should_stop.load(Ordering::Relaxed) {
            break;
        }
        
        if let Some(limit) = time_limit {
            let elapsed = start_time.elapsed();
            let should_stop = if depth_offset == 0 {
                elapsed >= Duration::from_millis(400)
            } else if depth_offset <= 1 {
                elapsed >= Duration::from_millis(450)
            } else {
                elapsed >= limit
            };
            
            if should_stop {
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

        MoveOrdering::order_moves(&local_state, &mut moves);
        
        if thread_id > 0 && moves.len() > 4 {
            let is_tactical = local_state.get_possible_moves().len() < 20;
            if !is_tactical {
                let rotation = thread_id % (moves.len() - 2).max(1);
                if moves.len() > 2 && rotation > 0 {
                    let rest = &mut moves[2..];
                    rest.rotate_left(rotation.min(rest.len()));
                }
            }
        }

        if let Ok(shared_best) = shared_state.best_move.lock() {
            if let Some(prev_best) = *shared_best {
                if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                    moves.swap(0, pos);
                }
            }
        }

        let mut best_score_this_depth = if is_maximizing { -1_000_000 } else { 1_000_000 };

        for &mv in &moves {
            if shared_state.should_stop.load(Ordering::Relaxed) {
                break;
            }

            local_state.make_move(mv);
            
            let (alpha, beta) = if thread_id == 0 {
                (i32::MIN, i32::MAX)
            } else {
                let current_best = shared_state.best_score.load(Ordering::Relaxed);
                let base_window = if depth_offset >= 3 { 100 } else { 50 };
                let window = base_window + thread_id as i32 * 15;
                (current_best.saturating_sub(window), current_best.saturating_add(window))
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

            let is_better = if is_maximizing {
                score > best_score_this_depth
            } else {
                score < best_score_this_depth
            };

            if is_better {
                best_score_this_depth = score;
                
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
                    
                    if score.abs() >= 1_000_000 {
                        shared_state.should_stop.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        }

        depth_reached = depth;
        shared_state.max_depth_reached.store(depth, Ordering::Relaxed);

        if best_score_this_depth.abs() >= 1_000_000 {
            if thread_id == 0 || depth_offset <= 2 {
                shared_state.should_stop.store(true, Ordering::Relaxed);
                break;
            } else if shared_state.should_stop.load(Ordering::Relaxed) {
                break;
            }
        }
    }

    shared_state.nodes_searched.fetch_add(nodes_searched, Ordering::Relaxed);

    SearchResult {
        best_move: None,
        score: 0,
        depth_reached,
        nodes_searched,
        time_elapsed: start_time.elapsed(),
    }
}
