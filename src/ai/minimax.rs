use crate::core::state::GameState;
use std::cmp::{max, min};
use std::time::{Duration, Instant};

use super::{heuristic::Heuristic, move_ordering::MoveOrdering, transposition::{TranspositionTable, EntryType}};

pub fn minimax(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &mut TranspositionTable,
) -> (i32, u64) {
    let original_alpha = alpha;
    let hash_key = state.hash();
    let mut nodes_visited = 1u64; // Count this node
    
    let tt_result = tt.probe(hash_key, depth, alpha, beta);
    if tt_result.cutoff {
        return (tt_result.value.unwrap(), nodes_visited);
    }

    if depth == 0 || state.is_terminal() {
        let eval = Heuristic::evaluate(state, depth);
        tt.store(hash_key, eval, depth, EntryType::Exact, None);
        return (eval, nodes_visited);
    }

    let mut moves = state.get_possible_moves();
    MoveOrdering::order_moves(state, &mut moves);
    
    if let Some(best_move) = tt_result.best_move {
        if let Some(pos) = moves.iter().position(|&m| m == best_move) {
            moves.swap(0, pos);
        }
    }

    let mut best_move = None;
    let mut value;

    if maximizing_player {
        value = i32::MIN;
        for move_ in moves {
            state.make_move(move_);
            let (eval, child_nodes) = minimax(state, depth - 1, alpha, beta, false, tt);
            state.undo_move(move_);
            nodes_visited += child_nodes;
            
            if eval > value {
                value = eval;
                best_move = Some(move_);
            }
            
            if value >= beta {
                break; // Beta cutoff
            }
            alpha = max(alpha, value);
        }
    } else {
        value = i32::MAX;
        for move_ in moves {
            state.make_move(move_);
            let (eval, child_nodes) = minimax(state, depth - 1, alpha, beta, true, tt);
            state.undo_move(move_);
            nodes_visited += child_nodes;
            
            if eval < value {
                value = eval;
                best_move = Some(move_);
            }
            
            if value <= alpha {
                break; // Alpha cutoff
            }
            beta = min(beta, value);
        }
    }

    let entry_type = if value <= original_alpha {
        EntryType::UpperBound
    } else if value >= beta {
        EntryType::LowerBound
    } else {
        EntryType::Exact
    };
    
    tt.store(hash_key, value, depth, entry_type, best_move);
    (value, nodes_visited)
}

#[derive(Debug)]
pub struct SearchResult {
    pub best_move: Option<(usize, usize)>,
    pub score: i32,
    pub depth_reached: i32,
    pub nodes_searched: u64,
    pub time_elapsed: Duration,
}

pub fn iterative_deepening_search(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    tt: &mut TranspositionTable,
) -> SearchResult {
    let start_time = Instant::now();
    let mut best_move = None;
    let mut best_score = if state.current_player == crate::core::board::Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut nodes_searched = 0u64;
    let mut depth_reached = 0;

    // Advance transposition table age for this search
    tt.advance_age();

    // Check if there are any possible moves at all
    let initial_moves = state.get_possible_moves();
    if initial_moves.is_empty() {
        return SearchResult {
            best_move: None,
            score: 0, // Draw/no moves available
            depth_reached: 0,
            nodes_searched: 0,
            time_elapsed: start_time.elapsed(),
        };
    }

    for depth in 1..=max_depth {
        let depth_start_time = Instant::now();
        
        // Check time limit before starting new depth iteration
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                break;
            }
        }

        let mut iteration_best_move = None;
        let mut iteration_best_score = if state.current_player == crate::core::board::Player::Max {
            i32::MIN
        } else {
            i32::MAX
        };

        let mut moves = state.get_possible_moves();
        MoveOrdering::order_moves(state, &mut moves);
        
        // Use best move from previous iteration for move ordering
        if let Some(prev_best) = best_move {
            if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                moves.swap(0, pos);
            }
        }

        let mut all_moves_searched = true;
        for mv in moves {
            // Check time limit during move iteration
            if let Some(limit) = time_limit {
                if start_time.elapsed() >= limit {
                    all_moves_searched = false;
                    break;
                }
            }

            state.make_move(mv);
            let (score, child_nodes) = minimax(
                state,
                depth - 1,
                i32::MIN,
                i32::MAX,
                state.current_player == crate::core::board::Player::Max,
                tt,
            );
            state.undo_move(mv);
            nodes_searched += child_nodes;

            let is_better = if state.current_player == crate::core::board::Player::Max {
                score > iteration_best_score
            } else {
                score < iteration_best_score
            };

            if is_better {
                iteration_best_score = score;
                iteration_best_move = Some(mv);
            }
        }

        // Only update best result if we completed the full depth search
        if all_moves_searched {
            best_move = iteration_best_move;
            best_score = iteration_best_score;
            depth_reached = depth;
            
            // Check for immediate win/loss - no need to search deeper
            if best_score.abs() >= 1_000_000 {
                break;
            }
        } else {
            // If we didn't complete this depth, don't use its results
            break;
        }

        let depth_time = depth_start_time.elapsed();
        // Optional debug output - can be enabled with a feature flag
        #[cfg(debug_assertions)]
        println!(
            "Depth {} completed in {:?}: best_move={:?}, score={}, nodes={}",
            depth, depth_time, best_move, best_score, nodes_searched
        );
    }

    SearchResult {
        best_move,
        score: best_score,
        depth_reached,
        nodes_searched,
        time_elapsed: start_time.elapsed(),
    }
}
