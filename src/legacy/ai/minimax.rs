use crate::core::state::GameState;
use std::cmp::{max, min};
use std::time::{Duration, Instant};

use crate::ai::{heuristic::Heuristic, move_ordering::MoveOrdering};
use crate::legacy::ai::transposition::{TranspositionTable, EntryType};

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
    let mut nodes_evaluated = 1; // Count this node
    
    let tt_result = tt.probe(hash_key, depth, alpha, beta);
    if tt_result.cutoff {
        return (tt_result.value.unwrap(), nodes_evaluated);
    }

    if depth == 0 || state.is_terminal() {
        let eval = Heuristic::evaluate(state, depth);
        tt.store_with_pv(hash_key, eval, depth, EntryType::Exact, None, false);
        return (eval, nodes_evaluated);
    }

    // Razoring: At low depths, if position looks bad, reduce depth
    if depth <= 3 && !maximizing_player {
        let static_eval = Heuristic::evaluate(state, depth);
        let razor_margin = 150 * depth;
        if static_eval.saturating_add(razor_margin) < alpha {
            let razor_depth = depth - 1;
            if razor_depth <= 0 {
                return (static_eval, nodes_evaluated);
            }
            let (razor_value, razor_nodes) = minimax(state, razor_depth, alpha, beta, maximizing_player, tt);
            nodes_evaluated += razor_nodes;
            if razor_value < alpha {
                return (razor_value, nodes_evaluated);
            }
        }
    }

    let mut moves = state.get_possible_moves();
    
    // Futility pruning: If we can't improve alpha even with best case, skip this node
    if depth <= 2 && !maximizing_player && moves.len() > 4 {
        let static_eval = Heuristic::evaluate(state, depth);
        let futility_margin = 200 * depth;
        if static_eval.saturating_add(futility_margin) < alpha {
            return (static_eval, nodes_evaluated);
        }
    }

    // Delta pruning: More aggressive pruning for deeper searches
    if depth >= 4 && moves.len() > 8 {
        let static_eval = Heuristic::evaluate(state, depth);
        let delta_margin = 300 + 50 * depth; // Larger margins for deeper searches
        if maximizing_player && static_eval.saturating_add(delta_margin) < alpha {
            return (static_eval, nodes_evaluated);
        } else if !maximizing_player && static_eval.saturating_sub(delta_margin) > beta {
            return (static_eval, nodes_evaluated);
        }
    }

    // Null move pruning: Give opponent an extra move and see if position is still good
    if depth >= 3 && !state.is_terminal() && moves.len() > 6 {
        let static_eval = Heuristic::evaluate(state, depth);
        let null_move_threshold = if maximizing_player { beta } else { alpha };
        
        // Only try null move if we're in a "good" position
        let should_try_null = if maximizing_player {
            static_eval >= null_move_threshold.saturating_add(100)
        } else {
            static_eval <= null_move_threshold.saturating_sub(100)
        };
        
        if should_try_null {
            let null_depth = depth - 3; // Aggressive reduction for null move
            if null_depth > 0 {
                let (null_score, null_nodes) = minimax(
                    state, 
                    null_depth, 
                    alpha, 
                    beta, 
                    !maximizing_player, 
                    tt
                );
                nodes_evaluated += null_nodes;
                
                // If null move causes a cutoff, prune this branch
                if maximizing_player && null_score >= beta {
                    return (null_score, nodes_evaluated);
                } else if !maximizing_player && null_score <= alpha {
                    return (null_score, nodes_evaluated);
                }
            }
        }
    }
    
    MoveOrdering::order_moves(state, &mut moves);
    
    // Move TT best move to front for better move ordering (PV move from previous iteration)
    if let Some(best_move) = tt_result.best_move {
        if let Some(pos) = moves.iter().position(|&m| m == best_move) {
            moves.swap(0, pos);
        }
    }

    let mut best_move = None;
    let mut value;
    let mut first_move = true;
    let mut moves_searched = 0;

    if maximizing_player {
        value = i32::MIN;
        for move_ in moves {
            state.make_move(move_);
            moves_searched += 1;
            
            let (eval, child_nodes) = if first_move || depth <= 2 {
                // Search first move (likely PV) and shallow searches with full window
                minimax(state, depth - 1, alpha, beta, false, tt)
            } else if moves_searched <= 3 {
                // Search first few moves with null window
                let (null_eval, null_nodes) = minimax(state, depth - 1, alpha, alpha + 1, false, tt);
                if null_eval > alpha && null_eval < beta {
                    // Re-search with full window if null window indicates this might be better
                    let (full_eval, full_nodes) = minimax(state, depth - 1, alpha, beta, false, tt);
                    (full_eval, null_nodes + full_nodes)
                } else {
                    (null_eval, null_nodes)
                }
            } else {
                // Late Move Reduction: Reduce depth for moves beyond the first few
                let reduction = if depth >= 4 && moves_searched > 6 { 2 } else { 1 };
                let reduced_depth = (depth - 1 - reduction).max(0);
                
                let (lmr_eval, lmr_nodes) = minimax(state, reduced_depth, alpha, alpha + 1, false, tt);
                
                // If LMR search fails high, re-search with full depth and window
                if lmr_eval > alpha && reduced_depth < depth - 1 {
                    let (full_eval, full_nodes) = minimax(state, depth - 1, alpha, beta, false, tt);
                    (full_eval, lmr_nodes + full_nodes)
                } else {
                    (lmr_eval, lmr_nodes)
                }
            };
            
            nodes_evaluated += child_nodes;
            state.undo_move(move_);
            first_move = false;
            
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
            moves_searched += 1;
            
            let (eval, child_nodes) = if first_move || depth <= 2 {
                // Search first move (likely PV) and shallow searches with full window
                minimax(state, depth - 1, alpha, beta, true, tt)
            } else if moves_searched <= 3 {
                // Search first few moves with null window
                let (null_eval, null_nodes) = minimax(state, depth - 1, beta - 1, beta, true, tt);
                if null_eval > alpha && null_eval < beta {
                    // Re-search with full window if null window indicates this might be better
                    let (full_eval, full_nodes) = minimax(state, depth - 1, alpha, beta, true, tt);
                    (full_eval, null_nodes + full_nodes)
                } else {
                    (null_eval, null_nodes)
                }
            } else {
                // Late Move Reduction: Reduce depth for moves beyond the first few
                let reduction = if depth >= 4 && moves_searched > 6 { 2 } else { 1 };
                let reduced_depth = (depth - 1 - reduction).max(0);
                
                let (lmr_eval, lmr_nodes) = minimax(state, reduced_depth, beta - 1, beta, true, tt);
                
                // If LMR search fails low, re-search with full depth and window
                if lmr_eval < beta && reduced_depth < depth - 1 {
                    let (full_eval, full_nodes) = minimax(state, depth - 1, alpha, beta, true, tt);
                    (full_eval, lmr_nodes + full_nodes)
                } else {
                    (lmr_eval, lmr_nodes)
                }
            };
            
            nodes_evaluated += child_nodes;
            state.undo_move(move_);
            first_move = false;
            
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
    
    // Mark as PV node if we have an exact value (no cutoff occurred)
    let is_pv = entry_type == EntryType::Exact;
    tt.store_with_pv(hash_key, value, depth, entry_type, best_move, is_pv);
    (value, nodes_evaluated)
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
        
        // Prioritize best move from previous iteration (PV move) for better move ordering
        if let Some(prev_best) = best_move {
            if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                moves.swap(0, pos);
            }
        }

        // Aggressive move pruning at root for deeper searches
        if depth >= 8 && moves.len() > 8 {
            moves.truncate(8); // Only consider top 8 moves at very deep searches  
        } else if depth >= 6 && moves.len() > 12 {
            moves.truncate(12); // Only consider top 12 moves at deep searches
        } else if depth >= 4 && moves.len() > 16 {
            moves.truncate(16); // Only consider top 16 moves at medium depths
        }

        let mut all_moves_searched = true;
        let mut first_move = true;
        
        for mv in moves {
            // Check time limit during move iteration
            if let Some(limit) = time_limit {
                if start_time.elapsed() >= limit {
                    all_moves_searched = false;
                    break;
                }
            }

            state.make_move(mv);
            
            // Use aspiration windows for the first move (expected PV move) at higher depths
            let (score, move_nodes) = if first_move && depth > 2 && best_score != i32::MIN && best_score != i32::MAX {
                // Try narrow aspiration window first for aggressive pruning
                let window = if depth >= 6 { 25 } else { 50 }; // Narrower windows for deeper searches
                let asp_alpha = best_score.saturating_sub(window);
                let asp_beta = best_score.saturating_add(window);
                
                let (asp_score, asp_nodes) = minimax(
                    state,
                    depth - 1,
                    asp_alpha,
                    asp_beta,
                    state.current_player == crate::core::board::Player::Max,
                    tt,
                );
                
                // If aspiration window fails, re-search with wider window
                if asp_score <= asp_alpha || asp_score >= asp_beta {
                    let wider_window = window * 3;
                    let (full_score, full_nodes) = minimax(
                        state,
                        depth - 1,
                        best_score.saturating_sub(wider_window),
                        best_score.saturating_add(wider_window),
                        state.current_player == crate::core::board::Player::Max,
                        tt,
                    );
                    (full_score, asp_nodes + full_nodes)
                } else {
                    (asp_score, asp_nodes)
                }
            } else {
                minimax(
                    state,
                    depth - 1,
                    i32::MIN,
                    i32::MAX,
                    state.current_player == crate::core::board::Player::Max,
                    tt,
                )
            };
            
            state.undo_move(mv);
            nodes_searched += move_nodes;
            first_move = false;

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
        println!(
            "🎯 PV-Search Depth {} completed in {:?}: best_move={:?}, score={}, nodes={}",
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
