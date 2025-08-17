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
    result.best_move
}

pub fn find_best_move_timed(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Duration,
    tt: &mut TranspositionTable,
) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, max_depth, Some(time_limit), tt);
    result.best_move
}
