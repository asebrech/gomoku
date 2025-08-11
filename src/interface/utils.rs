use crate::ai::{minimax::{minimax, iterative_deepening_search}, transposition::TranspositionTable};
use crate::core::state::GameState;
use crate::core::board::Player;
use std::time::Duration;

pub fn find_best_move(state: &mut GameState, depth: i32, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    // Use iterative deepening by default for better performance
    find_best_move_iterative(state, depth, tt)
}

pub fn find_best_move_legacy(state: &mut GameState, depth: i32, tt: &mut TranspositionTable) -> Option<(usize, usize)> {
    let mut best_move = None;
    let current_player = state.current_player;
    let mut best_score = if current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };

    let mut moves = state.get_possible_moves();
    // Use the same sophisticated move ordering as iterative deepening
    crate::ai::move_ordering::MoveOrdering::order_moves(state, &mut moves);

    for mv in moves {
        state.make_move(mv);
        let (score, _) = minimax(
            state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            current_player == Player::Min,
            tt,
        );
        state.undo_move(mv);

        if (current_player == Player::Max && score > best_score)
            || (current_player == Player::Min && score < best_score)
        {
            best_score = score;
            best_move = Some(mv);
        }
    }

    best_move
}

pub fn find_best_move_iterative(
    state: &mut GameState, 
    max_depth: i32, 
    tt: &mut TranspositionTable
) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, max_depth, None, tt);
    
    #[cfg(debug_assertions)]
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
    
    #[cfg(debug_assertions)]
    println!(
        "⏱️  Timed search completed: depth={}/{}, score={}, nodes={}, time={:?}",
        result.depth_reached, max_depth, result.score, result.nodes_searched, result.time_elapsed
    );
    
    result.best_move
}
