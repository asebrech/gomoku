use crate::ai::{minimax::{iterative_deepening_search, SearchResult}, transposition::TranspositionTable};
use crate::core::state::GameState;
use std::time::Duration;

pub fn find_best_move(
    state: &mut GameState, 
    max_depth: i32, 
    time_limit: Option<Duration>,
    tt: &mut TranspositionTable
) -> SearchResult {
    iterative_deepening_search(state, max_depth, time_limit, tt)
}
