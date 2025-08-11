use crate::ai::{minimax::iterative_deepening_search, transposition::TranspositionTable};
use crate::core::state::GameState;
use std::time::Duration;

pub fn find_best_move(state: &mut GameState, depth: i32, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    find_best_move_iterative(state, depth, tt)
}

pub fn find_best_move_iterative(
    state: &mut GameState, 
    max_depth: i32, 
    tt: &mut TranspositionTable
) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, max_depth, None, tt);
    
    #[cfg(debug_assertions)]
    {
        let nps = if result.time_elapsed.as_millis() > 0 {
            (result.nodes_searched as f64 / result.time_elapsed.as_millis() as f64 * 1000.0) as u64
        } else {
            result.nodes_searched
        };
        println!(
            "🧠 Iterative deepening: depth={}, score={}, nodes={}, time={:.1}ms, nps={}",
            result.depth_reached, result.score, result.nodes_searched, 
            result.time_elapsed.as_millis(), nps
        );
    }
    
    result.best_move
}

pub fn find_best_move_timed(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Duration,
    tt: &mut TranspositionTable,
) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, max_depth, Some(time_limit), tt);
    
    #[cfg(debug_assertions)]
    {
        let nps = if result.time_elapsed.as_millis() > 0 {
            (result.nodes_searched as f64 / result.time_elapsed.as_millis() as f64 * 1000.0) as u64
        } else {
            result.nodes_searched
        };
        let efficiency = (result.depth_reached as f64 / max_depth as f64 * 100.0) as u32;
        println!(
            "⏱️  Timed search: depth={}/{} ({}%), score={}, nodes={}, time={:.1}ms, nps={}",
            result.depth_reached, max_depth, efficiency, result.score, 
            result.nodes_searched, result.time_elapsed.as_millis(), nps
        );
    }
    
    result.best_move
}
