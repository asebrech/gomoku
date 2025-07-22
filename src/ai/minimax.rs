use crate::core::state::GameState;
use std::cmp::{max, min};
use std::time::{Duration, Instant};

use super::{heuristic::Heuristic, move_ordering::{MoveOrdering, KillerTable, HistoryTable}, transposition::{TranspositionTable, SharedTranspositionTable, EntryType}};

// Enhanced minimax with killer moves and history heuristic
pub fn minimax_enhanced(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &mut TranspositionTable,
    killer_table: &mut KillerTable,
    history_table: &mut HistoryTable,
) -> (i32, u64) {
    let original_alpha = alpha;
    let mut nodes_evaluated = 1;
    
    // Check for terminal states
    if depth == 0 || state.is_terminal() {
        return (Heuristic::evaluate(state, depth), nodes_evaluated);
    }
    
    // Transposition table lookup
    let hash_key = state.board.hash();
    let tt_result = tt.probe(hash_key, depth, alpha, beta);
    
    if tt_result.cutoff {
        return (tt_result.value.unwrap(), nodes_evaluated);
    }
    
    // Enhanced pruning techniques remain the same...
    // Razor pruning: If static eval + margin can't improve alpha, try reduced search
    if depth >= 3 && !maximizing_player && !state.is_terminal() {
        let static_eval = Heuristic::evaluate(state, depth);
        let razor_margin = 150 * depth;
        if static_eval.saturating_add(razor_margin) < alpha {
            let razor_depth = depth - 1;
            if razor_depth <= 0 {
                return (static_eval, nodes_evaluated);
            }
            let (razor_value, razor_nodes) = minimax_enhanced(state, razor_depth, alpha, beta, maximizing_player, tt, killer_table, history_table);
            nodes_evaluated += razor_nodes;
            if razor_value < alpha {
                return (razor_value, nodes_evaluated);
            }
        }
    }

    let mut moves = state.get_possible_moves();
    
    // Enhanced move ordering with killer moves and history
    MoveOrdering::order_moves_enhanced(state, &mut moves, tt_result.best_move, killer_table, history_table, depth);
    
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
                minimax_enhanced(state, depth - 1, alpha, beta, false, tt, killer_table, history_table)
            } else if moves_searched <= 3 {
                // Principal Variation Search for first few moves
                let (null_eval, null_nodes) = minimax_enhanced(state, depth - 1, alpha, alpha + 1, false, tt, killer_table, history_table);
                if null_eval > alpha && null_eval < beta {
                    let (full_eval, full_nodes) = minimax_enhanced(state, depth - 1, alpha, beta, false, tt, killer_table, history_table);
                    (full_eval, null_nodes + full_nodes)
                } else {
                    (null_eval, null_nodes)
                }
            } else {
                // Late Move Reduction with more aggressive reductions for deeper searches
                let reduction = if depth >= 6 && moves_searched > 8 { 3 }
                              else if depth >= 4 && moves_searched > 6 { 2 } 
                              else { 1 };
                let reduced_depth = (depth - 1 - reduction).max(0);
                
                let (lmr_eval, lmr_nodes) = minimax_enhanced(state, reduced_depth, alpha, alpha + 1, false, tt, killer_table, history_table);
                
                if lmr_eval > alpha && reduced_depth < depth - 1 {
                    let (full_eval, full_nodes) = minimax_enhanced(state, depth - 1, alpha, beta, false, tt, killer_table, history_table);
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
                // Store killer move and update history
                killer_table.store_killer(depth, move_);
                history_table.record_cutoff(move_, depth);
                break;
            }
            alpha = max(alpha, value);
        }
    } else {
        value = i32::MAX;
        for move_ in moves {
            state.make_move(move_);
            moves_searched += 1;
            
            let (eval, child_nodes) = if first_move || depth <= 2 {
                minimax_enhanced(state, depth - 1, alpha, beta, true, tt, killer_table, history_table)
            } else if moves_searched <= 3 {
                let (null_eval, null_nodes) = minimax_enhanced(state, depth - 1, beta - 1, beta, true, tt, killer_table, history_table);
                if null_eval > alpha && null_eval < beta {
                    let (full_eval, full_nodes) = minimax_enhanced(state, depth - 1, alpha, beta, true, tt, killer_table, history_table);
                    (full_eval, null_nodes + full_nodes)
                } else {
                    (null_eval, null_nodes)
                }
            } else {
                let reduction = if depth >= 6 && moves_searched > 8 { 3 }
                              else if depth >= 4 && moves_searched > 6 { 2 } 
                              else { 1 };
                let reduced_depth = (depth - 1 - reduction).max(0);
                
                let (lmr_eval, lmr_nodes) = minimax_enhanced(state, reduced_depth, beta - 1, beta, true, tt, killer_table, history_table);
                
                if lmr_eval < beta && reduced_depth < depth - 1 {
                    let (full_eval, full_nodes) = minimax_enhanced(state, depth - 1, alpha, beta, true, tt, killer_table, history_table);
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
                // Store killer move and update history
                killer_table.store_killer(depth, move_);
                history_table.record_cutoff(move_, depth);
                break;
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
    
    let is_pv = entry_type == EntryType::Exact;
    tt.store_with_pv(hash_key, value, depth, entry_type, best_move, is_pv);
    (value, nodes_evaluated)
}

// Enhanced iterative deepening with killer moves and history heuristic
pub fn iterative_deepening_enhanced(
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
    
    // Initialize killer and history tables
    let mut killer_table = KillerTable::new();
    let mut history_table = HistoryTable::new();

    // Advance transposition table age for this search
    tt.advance_age();

    // Check if there are any possible moves at all
    let initial_moves = state.get_possible_moves();
    if initial_moves.is_empty() {
        return SearchResult {
            best_move: None,
            score: 0,
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
        
        // Use enhanced move ordering for better pruning
        MoveOrdering::order_moves_enhanced(state, &mut moves, best_move, &killer_table, &history_table, depth);
        
        // Prioritize best move from previous iteration (PV move)
        if let Some(prev_best) = best_move {
            if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                moves.swap(0, pos);
            }
        }

        // More aggressive move pruning for deeper searches
        if depth >= 10 && moves.len() > 6 {
            moves.truncate(6); // Only top 6 moves at very deep searches
        } else if depth >= 8 && moves.len() > 8 {
            moves.truncate(8); // Only top 8 moves at deep searches  
        } else if depth >= 6 && moves.len() > 12 {
            moves.truncate(12); // Only top 12 moves at deep searches
        } else if depth >= 4 && moves.len() > 16 {
            moves.truncate(16); // Only top 16 moves at medium depths
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
            
            // Use aspiration windows for the first move at higher depths
            let (score, move_nodes) = if first_move && depth > 3 && best_score != i32::MIN && best_score != i32::MAX {
                let window = if depth >= 8 { 25 } else { 50 }; // Narrower windows for deeper searches
                let asp_alpha = best_score.saturating_sub(window);
                let asp_beta = best_score.saturating_add(window);
                
                let (asp_score, asp_nodes) = minimax_enhanced(
                    state,
                    depth - 1,
                    asp_alpha,
                    asp_beta,
                    state.current_player == crate::core::board::Player::Max,
                    tt,
                    &mut killer_table,
                    &mut history_table,
                );
                
                // If aspiration window fails, re-search with wider window
                if asp_score <= asp_alpha || asp_score >= asp_beta {
                    let wider_window = window * 3;
                    let (full_score, full_nodes) = minimax_enhanced(
                        state,
                        depth - 1,
                        best_score.saturating_sub(wider_window),
                        best_score.saturating_add(wider_window),
                        state.current_player == crate::core::board::Player::Max,
                        tt,
                        &mut killer_table,
                        &mut history_table,
                    );
                    (full_score, asp_nodes + full_nodes)
                } else {
                    (asp_score, asp_nodes)
                }
            } else {
                minimax_enhanced(
                    state,
                    depth - 1,
                    i32::MIN,
                    i32::MAX,
                    state.current_player == crate::core::board::Player::Max,
                    tt,
                    &mut killer_table,
                    &mut history_table,
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
            if best_score.abs() > 900_000 {
                break;
            }
        }
        
        // Age history table periodically to prevent stale data
        if depth % 3 == 0 {
            history_table.age_history();
        }

        println!(
            "📊 Enhanced Depth {}: score={}, move={:?}, time={:?}, nodes={}",
            depth, iteration_best_score, iteration_best_move, depth_start_time.elapsed(), nodes_searched
        );
        
        // Early termination if we're running out of time
        if let Some(limit) = time_limit {
            let elapsed = start_time.elapsed();
            let time_per_depth = elapsed.as_millis() / depth as u128;
            let estimated_next = time_per_depth * (depth + 1) as u128;
            
            if elapsed.as_millis() + estimated_next >= limit.as_millis() {
                break;
            }
        }
    }

    SearchResult {
        best_move,
        score: best_score,
        depth_reached,
        nodes_searched,
        time_elapsed: start_time.elapsed(),
    }
}

// Enhanced parallel iterative deepening with killer moves and history heuristic
pub fn parallel_iterative_deepening_enhanced(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    shared_tt: &SharedTranspositionTable,
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
    
    // Initialize killer and history tables (these will be shared across depths but not threads)
    let mut killer_table = KillerTable::new();
    let mut history_table = HistoryTable::new();

    // Advance age for each search to help with replacement policy
    shared_tt.advance_age();

    // Check if there are any possible moves at all
    let initial_moves = state.get_possible_moves();
    if initial_moves.is_empty() {
        return SearchResult {
            best_move: None,
            score: 0,
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

        let mut moves = state.get_possible_moves();
        
        // Use enhanced move ordering for better pruning
        MoveOrdering::order_moves_enhanced(state, &mut moves, best_move, &killer_table, &history_table, depth);
        
        // Prioritize best move from previous iteration (PV move)
        if let Some(prev_best) = best_move {
            if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                moves.swap(0, pos);
            }
        }

        // More aggressive move pruning for deeper searches
        if depth >= 10 && moves.len() > 6 {
            moves.truncate(6); // Only top 6 moves at very deep searches
        } else if depth >= 8 && moves.len() > 8 {
            moves.truncate(8); // Only top 8 moves at deep searches  
        } else if depth >= 6 && moves.len() > 12 {
            moves.truncate(12); // Only top 12 moves at deep searches
        } else if depth >= 4 && moves.len() > 16 {
            moves.truncate(16); // Only top 16 moves at medium depths
        }

        // For shallow depths (1-3), use sequential search to avoid overhead
        let (iteration_best_move, iteration_best_score, depth_nodes, all_moves_searched) = if depth <= 3 {
            search_moves_sequential_enhanced(state, &moves, depth, best_score, &shared_tt, &mut killer_table, &mut history_table, time_limit, start_time)
        } else {
            search_moves_parallel_enhanced(state, &moves, depth, best_score, &shared_tt, &killer_table, &history_table, time_limit, start_time)
        };

        nodes_searched += depth_nodes;

        // Only update best result if we completed the full depth search
        if all_moves_searched {
            best_move = iteration_best_move;
            best_score = iteration_best_score;
            depth_reached = depth;
            
            // Check for immediate win/loss - no need to search deeper
            if best_score.abs() > 900_000 {
                break;
            }
        } else {
            // If we didn't complete this depth, don't use its results
            break;
        }
        
        // Age history table periodically to prevent stale data
        if depth % 3 == 0 {
            history_table.age_history();
        }

        let depth_time = depth_start_time.elapsed();
        println!(
            "🚀 Enhanced Parallel Depth {}: score={}, move={:?}, time={:?}, nodes={}, completed={}",
            depth, iteration_best_score, iteration_best_move, depth_time, nodes_searched, all_moves_searched
        );
        
        // Early termination if we're running out of time
        if let Some(limit) = time_limit {
            let elapsed = start_time.elapsed();
            let time_per_depth = elapsed.as_millis() / depth as u128;
            let estimated_next = time_per_depth * (depth + 1) as u128;
            
            if elapsed.as_millis() + estimated_next >= limit.as_millis() {
                break;
            }
        }
    }

    SearchResult {
        best_move,
        score: best_score,
        depth_reached,
        nodes_searched,
        time_elapsed: start_time.elapsed(),
    }
}

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

// Thread-safe minimax function that works with SharedTranspositionTable
pub fn minimax_shared(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &SharedTranspositionTable,
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
            let (razor_value, razor_nodes) = minimax_shared(state, razor_depth, alpha, beta, maximizing_player, tt);
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
                let (null_score, null_nodes) = minimax_shared(
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
                minimax_shared(state, depth - 1, alpha, beta, false, tt)
            } else if moves_searched <= 3 {
                // Search first few moves with null window
                let (null_eval, null_nodes) = minimax_shared(state, depth - 1, alpha, alpha + 1, false, tt);
                if null_eval > alpha && null_eval < beta {
                    // Re-search with full window if null window indicates this might be better
                    let (full_eval, full_nodes) = minimax_shared(state, depth - 1, alpha, beta, false, tt);
                    (full_eval, null_nodes + full_nodes)
                } else {
                    (null_eval, null_nodes)
                }
            } else {
                // Late Move Reduction: Reduce depth for moves beyond the first few
                let reduction = if depth >= 4 && moves_searched > 6 { 2 } else { 1 };
                let reduced_depth = (depth - 1 - reduction).max(0);
                
                let (lmr_eval, lmr_nodes) = minimax_shared(state, reduced_depth, alpha, alpha + 1, false, tt);
                
                // If LMR search fails high, re-search with full depth and window
                if lmr_eval > alpha && reduced_depth < depth - 1 {
                    let (full_eval, full_nodes) = minimax_shared(state, depth - 1, alpha, beta, false, tt);
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
                minimax_shared(state, depth - 1, alpha, beta, true, tt)
            } else if moves_searched <= 3 {
                // Search first few moves with null window
                let (null_eval, null_nodes) = minimax_shared(state, depth - 1, beta - 1, beta, true, tt);
                if null_eval > alpha && null_eval < beta {
                    // Re-search with full window if null window indicates this might be better
                    let (full_eval, full_nodes) = minimax_shared(state, depth - 1, alpha, beta, true, tt);
                    (full_eval, null_nodes + full_nodes)
                } else {
                    (null_eval, null_nodes)
                }
            } else {
                // Late Move Reduction: Reduce depth for moves beyond the first few
                let reduction = if depth >= 4 && moves_searched > 6 { 2 } else { 1 };
                let reduced_depth = (depth - 1 - reduction).max(0);
                
                let (lmr_eval, lmr_nodes) = minimax_shared(state, reduced_depth, beta - 1, beta, true, tt);
                
                // If LMR search fails low, re-search with full depth and window
                if lmr_eval < beta && reduced_depth < depth - 1 {
                    let (full_eval, full_nodes) = minimax_shared(state, depth - 1, alpha, beta, true, tt);
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

// Multi-threaded iterative deepening search using root-level parallelization
pub fn parallel_iterative_deepening_search(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
) -> SearchResult {
    let shared_tt = SharedTranspositionTable::new_default();
    parallel_iterative_deepening_search_with_tt(state, max_depth, time_limit, &shared_tt)
}

// Multi-threaded iterative deepening search with provided transposition table
pub fn parallel_iterative_deepening_search_with_tt(
    state: &mut GameState,
    max_depth: i32,
    time_limit: Option<Duration>,
    shared_tt: &SharedTranspositionTable,
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

    // Advance age for each search to help with replacement policy
    shared_tt.advance_age();

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

        // For shallow depths (1-3), use sequential search to avoid overhead
        let (iteration_best_move, iteration_best_score, depth_nodes, all_moves_searched) = if depth <= 3 {
            search_moves_sequential(state, &moves, depth, best_score, &shared_tt, time_limit, start_time)
        } else {
            search_moves_parallel(state, &moves, depth, best_score, &shared_tt, time_limit, start_time)
        };

        nodes_searched += depth_nodes;

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
            "🧵 Parallel Depth {} completed in {:?}: best_move={:?}, score={}, nodes={}, all_moves_searched={}",
            depth, depth_time, best_move, best_score, nodes_searched, all_moves_searched
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

// Sequential search for shallow depths with enhanced move ordering
fn search_moves_sequential_enhanced(
    state: &mut GameState,
    moves: &[(usize, usize)],
    depth: i32,
    best_score: i32,
    shared_tt: &SharedTranspositionTable,
    killer_table: &mut KillerTable,
    history_table: &mut HistoryTable,
    time_limit: Option<Duration>,
    start_time: Instant,
) -> (Option<(usize, usize)>, i32, u64, bool) {
    let mut iteration_best_move = None;
    let mut iteration_best_score = if state.current_player == crate::core::board::Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut total_nodes = 0u64;
    let mut all_moves_searched = true;
    let mut first_move = true;

    for &mv in moves {
        // Check time limit during move iteration
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                all_moves_searched = false;
                break;
            }
        }

        state.make_move(mv);
        
        let (score, move_nodes) = if first_move && depth > 2 && best_score != i32::MIN && best_score != i32::MAX {
            // Use aspiration windows for the first move (expected PV move) at higher depths
            let window = if depth >= 8 { 25 } else { 50 };
            let asp_alpha = best_score.saturating_sub(window);
            let asp_beta = best_score.saturating_add(window);
            
            let (asp_score, asp_nodes) = minimax_enhanced_shared(
                state,
                depth - 1,
                asp_alpha,
                asp_beta,
                state.current_player == crate::core::board::Player::Max,
                shared_tt,
                killer_table,
                history_table,
            );
            
            // If aspiration window fails, re-search with wider window
            if asp_score <= asp_alpha || asp_score >= asp_beta {
                let wider_window = window * 3;
                let (full_score, full_nodes) = minimax_enhanced_shared(
                    state,
                    depth - 1,
                    best_score.saturating_sub(wider_window),
                    best_score.saturating_add(wider_window),
                    state.current_player == crate::core::board::Player::Max,
                    shared_tt,
                    killer_table,
                    history_table,
                );
                (full_score, asp_nodes + full_nodes)
            } else {
                (asp_score, asp_nodes)
            }
        } else {
            minimax_enhanced_shared(
                state,
                depth - 1,
                i32::MIN,
                i32::MAX,
                state.current_player == crate::core::board::Player::Max,
                shared_tt,
                killer_table,
                history_table,
            )
        };
        
        state.undo_move(mv);
        total_nodes += move_nodes;
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

    (iteration_best_move, iteration_best_score, total_nodes, all_moves_searched)
}

// Parallel search for deeper depths with enhanced move ordering
fn search_moves_parallel_enhanced(
    state: &mut GameState,
    moves: &[(usize, usize)],
    depth: i32,
    best_score: i32,
    shared_tt: &SharedTranspositionTable,
    killer_table: &KillerTable,
    history_table: &HistoryTable,
    time_limit: Option<Duration>,
    start_time: Instant,
) -> (Option<(usize, usize)>, i32, u64, bool) {
    use rayon::prelude::*;
    
    let mut iteration_best_move = None;
    let mut iteration_best_score = if state.current_player == crate::core::board::Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };
    
    // Use parallel iterator to search moves concurrently
    let results: Vec<_> = moves.par_iter().map(|&mv| {
        // Check time limit during move iteration
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                return None; // Skip this move if time limit exceeded
            }
        }
        
        // Clone the state and tables for this thread
        let mut local_state = state.clone();
        let mut local_killer_table = killer_table.clone();
        let mut local_history_table = history_table.clone();
        
        local_state.make_move(mv);
        
        let is_maximizing = local_state.current_player == crate::core::board::Player::Max;
        let (score, move_nodes) = minimax_enhanced_shared(
            &mut local_state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            is_maximizing,
            shared_tt,
            &mut local_killer_table,
            &mut local_history_table,
        );
        
        Some((mv, score, move_nodes))
    }).filter_map(|result| result).collect();
    
    let mut total_nodes = 0u64;
    let all_moves_searched = results.len() == moves.len();
    
    // Find the best move from results
    for (mv, score, move_nodes) in results {
        total_nodes += move_nodes;
        
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
    
    (iteration_best_move, iteration_best_score, total_nodes, all_moves_searched)
}

// Enhanced minimax with killer moves and history heuristic for shared transposition table
pub fn minimax_enhanced_shared(
    state: &mut GameState,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing_player: bool,
    tt: &SharedTranspositionTable,
    killer_table: &mut KillerTable,
    history_table: &mut HistoryTable,
) -> (i32, u64) {
    let original_alpha = alpha;
    let mut nodes_evaluated = 1;
    
    // Check for terminal states
    if depth == 0 || state.is_terminal() {
        return (Heuristic::evaluate(state, depth), nodes_evaluated);
    }
    
    // Transposition table lookup
    let hash_key = state.board.hash();
    let tt_result = tt.probe(hash_key, depth, alpha, beta);
    
    if tt_result.cutoff {
        return (tt_result.value.unwrap(), nodes_evaluated);
    }
    
    // Enhanced pruning techniques remain the same...
    // Razor pruning: If static eval + margin can't improve alpha, try reduced search
    if depth >= 3 && !maximizing_player && !state.is_terminal() {
        let static_eval = Heuristic::evaluate(state, depth);
        let razor_margin = 150 * depth;
        if static_eval.saturating_add(razor_margin) < alpha {
            let razor_depth = depth - 1;
            if razor_depth <= 0 {
                return (static_eval, nodes_evaluated);
            }
            let (razor_value, razor_nodes) = minimax_enhanced_shared(state, razor_depth, alpha, beta, maximizing_player, tt, killer_table, history_table);
            nodes_evaluated += razor_nodes;
            if razor_value < alpha {
                return (razor_value, nodes_evaluated);
            }
        }
    }

    let mut moves = state.get_possible_moves();
    
    // Enhanced move ordering with killer moves and history
    MoveOrdering::order_moves_enhanced(state, &mut moves, tt_result.best_move, killer_table, history_table, depth);
    
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
                minimax_enhanced_shared(state, depth - 1, alpha, beta, false, tt, killer_table, history_table)
            } else if moves_searched <= 3 {
                // Principal Variation Search for first few moves
                let (null_eval, null_nodes) = minimax_enhanced_shared(state, depth - 1, alpha, alpha + 1, false, tt, killer_table, history_table);
                if null_eval > alpha && null_eval < beta {
                    let (full_eval, full_nodes) = minimax_enhanced_shared(state, depth - 1, alpha, beta, false, tt, killer_table, history_table);
                    (full_eval, null_nodes + full_nodes)
                } else {
                    (null_eval, null_nodes)
                }
            } else {
                // Late Move Reduction with more aggressive reductions for deeper searches
                let reduction = if depth >= 6 && moves_searched > 8 { 3 }
                              else if depth >= 4 && moves_searched > 6 { 2 } 
                              else { 1 };
                let reduced_depth = (depth - 1 - reduction).max(0);
                
                let (lmr_eval, lmr_nodes) = minimax_enhanced_shared(state, reduced_depth, alpha, alpha + 1, false, tt, killer_table, history_table);
                
                if lmr_eval > alpha && reduced_depth < depth - 1 {
                    let (full_eval, full_nodes) = minimax_enhanced_shared(state, depth - 1, alpha, beta, false, tt, killer_table, history_table);
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
                // Store killer move and update history
                killer_table.store_killer(depth, move_);
                history_table.record_cutoff(move_, depth);
                break;
            }
            alpha = max(alpha, value);
        }
    } else {
        value = i32::MAX;
        for move_ in moves {
            state.make_move(move_);
            moves_searched += 1;
            
            let (eval, child_nodes) = if first_move || depth <= 2 {
                minimax_enhanced_shared(state, depth - 1, alpha, beta, true, tt, killer_table, history_table)
            } else if moves_searched <= 3 {
                let (null_eval, null_nodes) = minimax_enhanced_shared(state, depth - 1, beta - 1, beta, true, tt, killer_table, history_table);
                if null_eval > alpha && null_eval < beta {
                    let (full_eval, full_nodes) = minimax_enhanced_shared(state, depth - 1, alpha, beta, true, tt, killer_table, history_table);
                    (full_eval, null_nodes + full_nodes)
                } else {
                    (null_eval, null_nodes)
                }
            } else {
                let reduction = if depth >= 6 && moves_searched > 8 { 3 }
                              else if depth >= 4 && moves_searched > 6 { 2 } 
                              else { 1 };
                let reduced_depth = (depth - 1 - reduction).max(0);
                
                let (lmr_eval, lmr_nodes) = minimax_enhanced_shared(state, reduced_depth, beta - 1, beta, true, tt, killer_table, history_table);
                
                if lmr_eval < beta && reduced_depth < depth - 1 {
                    let (full_eval, full_nodes) = minimax_enhanced_shared(state, depth - 1, alpha, beta, true, tt, killer_table, history_table);
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
                // Store killer move and update history
                killer_table.store_killer(depth, move_);
                history_table.record_cutoff(move_, depth);
                break;
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
    
    let is_pv = entry_type == EntryType::Exact;
    tt.store_with_pv(hash_key, value, depth, entry_type, best_move, is_pv);
    (value, nodes_evaluated)
}

// Sequential search for shallow depths
fn search_moves_sequential(
    state: &mut GameState,
    moves: &[(usize, usize)],
    depth: i32,
    best_score: i32,
    shared_tt: &SharedTranspositionTable,
    time_limit: Option<Duration>,
    start_time: Instant,
) -> (Option<(usize, usize)>, i32, u64, bool) {
    let mut iteration_best_move = None;
    let mut iteration_best_score = if state.current_player == crate::core::board::Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut total_nodes = 0u64;
    let mut all_moves_searched = true;
    let mut first_move = true;

    for &mv in moves {
        // Check time limit during move iteration
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                all_moves_searched = false;
                break;
            }
        }

        state.make_move(mv);
        
        let (score, move_nodes) = if first_move && depth > 2 && best_score != i32::MIN && best_score != i32::MAX {
            // Use aspiration windows for the first move (expected PV move) at higher depths
            let window = 50;
            let asp_alpha = best_score.saturating_sub(window);
            let asp_beta = best_score.saturating_add(window);
            
            let (asp_score, asp_nodes) = minimax_shared(
                state,
                depth - 1,
                asp_alpha,
                asp_beta,
                state.current_player == crate::core::board::Player::Max,
                shared_tt,
            );
            
            // If aspiration window fails, re-search with wider window
            if asp_score <= asp_alpha || asp_score >= asp_beta {
                let wider_window = window * 3;
                let (full_score, full_nodes) = minimax_shared(
                    state,
                    depth - 1,
                    best_score.saturating_sub(wider_window),
                    best_score.saturating_add(wider_window),
                    state.current_player == crate::core::board::Player::Max,
                    shared_tt,
                );
                (full_score, asp_nodes + full_nodes)
            } else {
                (asp_score, asp_nodes)
            }
        } else {
            minimax_shared(
                state,
                depth - 1,
                i32::MIN,
                i32::MAX,
                state.current_player == crate::core::board::Player::Max,
                shared_tt,
            )
        };
        
        state.undo_move(mv);
        total_nodes += move_nodes;
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

    (iteration_best_move, iteration_best_score, total_nodes, all_moves_searched)
}

// Parallel search for deeper depths
fn search_moves_parallel(
    state: &mut GameState,
    moves: &[(usize, usize)],
    depth: i32,
    best_score: i32,
    shared_tt: &SharedTranspositionTable,
    time_limit: Option<Duration>,
    start_time: Instant,
) -> (Option<(usize, usize)>, i32, u64, bool) {
    use rayon::prelude::*;
    
    let mut iteration_best_move = None;
    let mut iteration_best_score = if state.current_player == crate::core::board::Player::Max {
        i32::MIN
    } else {
        i32::MAX
    };
    
    // Use parallel iterator to search moves concurrently
    let results: Vec<_> = moves.par_iter().map(|&mv| {
        // Check time limit during move iteration
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                return None; // Skip this move if time limit exceeded
            }
        }
        
        // Clone the state for this thread
        let mut local_state = state.clone();
        local_state.make_move(mv);
        
        let is_maximizing = local_state.current_player == crate::core::board::Player::Max;
        let (score, move_nodes) = minimax_shared(
            &mut local_state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            is_maximizing,
            shared_tt,
        );
        
        Some((mv, score, move_nodes))
    }).filter_map(|result| result).collect();
    
    let mut total_nodes = 0u64;
    let all_moves_searched = results.len() == moves.len();
    
    // Find the best move from results
    for (mv, score, move_nodes) in results {
        total_nodes += move_nodes;
        
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
    
    (iteration_best_move, iteration_best_score, total_nodes, all_moves_searched)
}
