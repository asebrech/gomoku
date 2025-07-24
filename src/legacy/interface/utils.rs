use crate::legacy::ai::minimax::iterative_deepening_search;
use crate::ai::transposition::TranspositionTable;
use crate::core::state::GameState;
use std::time::Duration;

pub fn find_best_move(state: &mut GameState, depth: i32, time_limit: Option<Duration>, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, depth, time_limit, tt);
    
    println!(
        "🧠 Sequential search completed: depth={}, score={}, nodes={}, time={:?}",
        result.depth_reached, result.score, result.nodes_searched, result.time_elapsed
    );
    
    result.best_move
}
