use crate::ai::{/*minimax::minimax,*/ heuristic::Heuristic/*, transposition::TranspositionTable*/};
use crate::core::state::GameState;
use crate::core::board::Player;
// use std::time::{Duration, Instant};

// Simple minimax without transposition table
fn simple_minimax(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
) -> i32 {
    if depth == 0 || state.is_terminal() {
        return Heuristic::evaluate(state);
    }

    let moves = state.get_possible_moves();
    if moves.is_empty() {
        return Heuristic::evaluate(state);
    }

    if maximizing_player {
        let mut max_eval = i32::MIN;
        for mv in moves {
            state.make_move(mv);
            let eval = simple_minimax(state, depth - 1, alpha, beta, false);
            state.undo_move(mv);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break; // Alpha-beta pruning
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for mv in moves {
            state.make_move(mv);
            let eval = simple_minimax(state, depth - 1, alpha, beta, true);
            state.undo_move(mv);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break; // Alpha-beta pruning
            }
        }
        min_eval
    }
}

pub fn iterative_deepening_search(
    state: &mut GameState,
    max_depth: i32,
) -> Option<(usize, usize)> {
    let mut best_move = None;
    let mut depth = 1;

    while depth <= max_depth {
        if let Some(move_) = search_at_depth(state, depth) {
            best_move = Some(move_);
            depth += 1;
        } else {
            println!("No valid moves found at depth {}", depth);
            break;
        }
    }

    best_move
}

fn search_at_depth(
    state: &mut GameState,
    depth: i32,
) -> Option<(usize, usize)> {
    let mut best_move = None;
    let mut best_score = if state.current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };

    let mut moves = state.get_possible_moves();
    crate::ai::heuristic::Heuristic::order_moves(state, &mut moves);

    for mv in moves {
        state.make_move(mv);
        let score = simple_minimax(
            state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            state.current_player == Player::Min,
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