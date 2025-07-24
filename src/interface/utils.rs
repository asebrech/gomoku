use crate::ai::{minimax::parallel_iterative_deepening_search_with_tt, transposition::SharedTranspositionTable};
use crate::core::state::GameState;
use std::time::Duration;

// Primary AI search function using parallel implementation
pub fn find_best_move(state: &mut GameState, depth: i32, time_limit: Option<Duration>, shared_tt: &SharedTranspositionTable) -> Option<(usize, usize)> {
    let result = parallel_iterative_deepening_search_with_tt(state, depth, time_limit, shared_tt);
    
    println!(
        "🧵 Parallel search completed: depth={}, score={}, nodes={}, time={:?}",
        result.depth_reached, result.score, result.nodes_searched, result.time_elapsed
    );
    
    result.best_move
}
