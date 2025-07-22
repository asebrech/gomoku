use crate::ai::{minimax::iterative_deepening_search, transposition::TranspositionTable};
use crate::core::state::GameState;
use std::time::Duration;

pub fn find_best_move(state: &mut GameState, depth: i32, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, depth, None, tt);
    
    println!(
        "🧠 Iterative deepening completed: depth={}, score={}, nodes={}, time={:?}",
        result.depth_reached, result.score, result.nodes_searched, result.time_elapsed
    );
    
    result.best_move
}

pub fn find_best_move_timed(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Duration,
    tt: &mut TranspositionTable,
) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, max_depth, Some(time_limit), tt);
    
    println!(
        "⏱️  Timed search completed: depth={}/{}, score={}, nodes={}, time={:?}",
        result.depth_reached, max_depth, result.score, result.nodes_searched, result.time_elapsed
    );
    
    result.best_move
}

