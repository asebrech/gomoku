use crate::core::state::GameState;
use std::time::{Duration, Instant};
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

pub fn find_best_move(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &mut TranspositionTable,
) -> SearchResult {
    let start_time = Instant::now();
    let mut best_move = None;
    let is_maximizing = state.current_player == crate::core::board::Player::Max;
    let mut best_score = if is_maximizing { i32::MIN } else { i32::MAX };
    let mut nodes_searched = 0u64;
    let mut depth_reached = 0;

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

    for depth in 1..=max_depth {
        if let Some(limit) = time_limit {
            let elapsed = start_time.elapsed();
            if elapsed >= limit {
                break;
            }
        }

        let mut iteration_best_move = None;
        let mut iteration_best_score = if is_maximizing { i32::MIN } else { i32::MAX };

        let mut moves = state.get_possible_moves();
        MoveOrdering::order_moves(state, &mut moves);
        
        if let Some(prev_best) = best_move {
            if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                moves.swap(0, pos);
            }
        }

        // Check time limit before processing moves
        if let Some(limit) = time_limit {
            let elapsed = start_time.elapsed();
            if elapsed >= limit {
                break;
            }
        }
        
        let move_results: Vec<_> = moves.par_iter().map(|&mv| {
            let mut state_clone = state.clone();
            let mut tt_local = TranspositionTable::new(tt.size().min(5_000));
            
            state_clone.make_move(mv);
            let (score, child_nodes) = minimax(
                &mut state_clone,
                depth - 1,
                i32::MIN,
                i32::MAX,
                !is_maximizing,
                &mut tt_local,
                &start_time,
                time_limit,
            );
            
            let (local_hits, local_misses) = tt_local.get_stats();
            (mv, score, child_nodes, local_hits, local_misses)
        }).collect();
        
        for (mv, score, child_nodes, local_hits, local_misses) in move_results {
            nodes_searched += child_nodes;
            
            // Add local TT stats to main TT stats
            tt.add_stats(local_hits, local_misses);

            let is_better = if is_maximizing {
                score > iteration_best_score
            } else {
                score < iteration_best_score
            };

            if is_better {
                iteration_best_score = score;
                iteration_best_move = Some(mv);
            }
        }
        
        best_move = iteration_best_move;
        best_score = iteration_best_score;
        depth_reached = depth;
        
        if best_score.abs() >= 1_000_000 {
            break;
        }
    }

    SearchResult {
        best_move,
        score: best_score,
        depth_reached,
        nodes_searched,
        time_elapsed: start_time.elapsed(),
    }
}
