use crate::ai::{minimax::minimax, transposition::TranspositionTable};
use crate::core::state::GameState;
use crate::core::board::{Player, initialize_zobrist};
// use std::time::{Duration, Instant};

pub fn iterative_deepening_search(
    state: &mut GameState,
    max_depth: i32,
) -> Option<(usize, usize)> {
    // Initialize Zobrist hashing if not already done
    initialize_zobrist();
    
    let mut best_move = None;
    let mut depth = 1;
    let mut tt = TranspositionTable::new();

    while depth <= max_depth {
        if let Some(move_) = search_at_depth(state, depth, &mut tt) {
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
    tt: &mut TranspositionTable,
) -> Option<(usize, usize)> {
    let mut best_move = None;
    let mut best_score = if state.current_player == Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };

    let mut moves = state.get_possible_moves();
    crate::ai::heuristic::Heuristic::order_moves(state, &mut moves);

	println!("Searching at depth {} with {} possible moves", depth, moves.len());
    for mv in moves {
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