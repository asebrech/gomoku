use crate::ai::{minimax::minimax, transposition::TranspositionTable};
use crate::core::state::GameState;
use crate::core::board::Player;
use std::time::{Duration, Instant};

pub fn iterative_deepening_search(
    state: &mut GameState,
    max_time: Duration,
    max_depth: i32,
) -> Option<(usize, usize)> {
    let start_time = Instant::now();
    let mut tt = TranspositionTable::new();
    let mut best_move = None;
    let mut depth = 1;

    while depth <= max_depth && start_time.elapsed() < max_time {
        let remaining_time = max_time - start_time.elapsed();
        
        if let Some(move_) = search_with_timeout(state, depth, remaining_time, &mut tt) {
            best_move = Some(move_);
            depth += 1;
        } else {
			println!("No valid moves found at depth {}", depth);
            break; // Time exceeded
        }
    }

    best_move
}

fn search_with_timeout(
    state: &mut GameState,
    depth: i32,
    timeout: Duration,
    tt: &mut TranspositionTable,
) -> Option<(usize, usize)> {
    let start_time = Instant::now();
    let mut best_move = None;
    let mut best_score = if state.current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };

    let mut moves = state.get_possible_moves();
    crate::ai::heuristic::Heuristic::order_moves(state, &mut moves);

    for mv in moves {
        if start_time.elapsed() >= timeout {
            break;
        }

        state.make_move(mv);
        let score = minimax(
            state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            state.current_player == Player::Min,
            tt,
        );
        state.undo_move(mv);

        if (state.current_player == Player::Max && score > best_score) ||
           (state.current_player == Player::Min && score < best_score) {
            best_score = score;
            best_move = Some(mv);
        }
    }

    best_move
}