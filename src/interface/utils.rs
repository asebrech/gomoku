use crate::ai::{minimax::{iterative_deepening_search, parallel_iterative_deepening_search_with_tt}, transposition::{TranspositionTable, SharedTranspositionTable}};
use crate::core::state::GameState;
use std::time::Duration;

pub fn find_best_move(state: &mut GameState, depth: i32, time_limit: Option<Duration>, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, depth, time_limit, tt);
    
    println!(
        "🧠 Iterative deepening completed: depth={}, score={}, nodes={}, time={:?}",
        result.depth_reached, result.score, result.nodes_searched, result.time_elapsed
    );
    
    result.best_move
}

// Multi-threaded AI search - automatically chooses between sequential and parallel based on depth
pub fn find_best_move_parallel(state: &mut GameState, depth: i32, time_limit: Option<Duration>, shared_tt: &SharedTranspositionTable) -> Option<(usize, usize)> {
    // For very shallow depths, use sequential search to avoid threading overhead
    let result = if depth <= 4 {
        let mut tt = TranspositionTable::new_default();
        iterative_deepening_search(state, depth, time_limit, &mut tt)
    } else {
        // Create a local copy of the shared transposition table for the parallel search
        // This will preserve existing entries while allowing thread-safe access
        parallel_iterative_deepening_search_with_tt(state, depth, time_limit, shared_tt)
    };
    
    println!(
        "🧵 Parallel search completed: depth={}, score={}, nodes={}, time={:?}",
        result.depth_reached, result.score, result.nodes_searched, result.time_elapsed
    );
    
    result.best_move
}
