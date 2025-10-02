use crate::core::state::GameState;
use std::time::{Duration, Instant};

use super::{minimax::mtdf, transposition::TranspositionTable};

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

    // Iterative deepening with MTD(f)
    for depth in 1..=max_depth {
        if let Some(limit) = time_limit {
            let elapsed = start_time.elapsed();
            // Stop if we've used 80% of time or exceeded limit
            if elapsed >= limit || elapsed > limit * 4 / 5 {
                break;
            }
        }

        // Use MTD(f) with the previous iteration's score as first guess
        let (score, iteration_nodes, iteration_best_move) = mtdf(
            state,
            best_score,
            depth,
            tt,
            &start_time,
            time_limit,
        );
        
        nodes_searched += iteration_nodes;
        
        // Check if time ran out during search
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                break;
            }
        }
        
        // Update best results
        if iteration_best_move.is_some() {
            best_move = iteration_best_move;
            best_score = score;
            depth_reached = depth;
            
            // If we found a winning or losing position, we can stop
            if best_score.abs() >= 1_000_000 {
                break;
            }
        } else {
            // If MTD(f) couldn't find a best move, stop iterating
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


