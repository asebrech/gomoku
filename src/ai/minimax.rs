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
    let mut nodes_visited = 1u64;
    
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
                break;
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
    let is_maximizing = state.current_player == crate::core::board::Player::Max;
    let mut best_score = if is_maximizing { i32::MIN } else { i32::MAX };
    let mut nodes_searched = 0u64;
    let mut depth_reached = 0;

    tt.advance_age();

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

    if let Some(immediate_move) = find_immediate_win_or_block(state) {
        #[cfg(debug_assertions)]
        {
            let player = if is_maximizing { "MAX" } else { "MIN" };
            println!("⚡ Immediate win/block found for {} at {:?}", player, immediate_move);
        }
        return SearchResult {
            best_move: Some(immediate_move),
            score: if is_maximizing { 1_000_000 } else { -1_000_000 },
            depth_reached: 1,
            nodes_searched: initial_moves.len() as u64,
            time_elapsed: start_time.elapsed(),
        };
    }

    for depth in 1..=max_depth {
        let depth_start_time = Instant::now();
        
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                break;
            }
        }

        let mut iteration_best_move = None;
        let mut iteration_best_score = if is_maximizing { i32::MIN } else { i32::MAX };

        let mut moves = state.get_possible_moves();
        MoveOrdering::order_moves(state, &mut moves);
        
        if let Some(prev_best) = best_move {
            if let Some(pos) = moves.iter().position(|&m| m == prev_best) {
                moves.swap(0, pos);
            }
        }

        let mut all_moves_searched = true;
        for mv in moves {
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
                !is_maximizing,
                tt,
            );
            state.undo_move(mv);
            nodes_searched += child_nodes;

            let is_better = if is_maximizing {
                score > iteration_best_score
            } else {
                score < iteration_best_score
            };

            if is_better {
                iteration_best_score = score;
                iteration_best_move = Some(mv);
            }
        }

        if all_moves_searched {
            best_move = iteration_best_move;
            best_score = iteration_best_score;
            depth_reached = depth;
            
            if best_score.abs() >= 1_000_000 {
                #[cfg(debug_assertions)]
                {
                    let result_type = if (is_maximizing && best_score > 0) || (!is_maximizing && best_score < 0) {
                        "WINNING"
                    } else {
                        "LOSING"
                    };
                    let player = if is_maximizing { "MAX" } else { "MIN" };
                    println!("🏆 {} position found for {} at depth {} (score={}), stopping search", 
                        result_type, player, depth, best_score);
                }
                break;
            }
        } else {
            #[cfg(debug_assertions)]
            println!("⏰ Time limit reached at depth {} ({:.1}ms elapsed), using depth {} results", 
                depth, start_time.elapsed().as_millis(), depth_reached);
            break;
        }

        #[cfg(debug_assertions)]
        {
            let depth_time = depth_start_time.elapsed();
            let nps = if depth_time.as_millis() > 0 {
                (nodes_searched as f64 / depth_time.as_millis() as f64 * 1000.0) as u64
            } else {
                nodes_searched
            };
            println!(
                "📊 Depth {} completed: {:.1}ms, move={:?}, score={}, nodes={}, nps={}",
                depth, depth_time.as_millis(), best_move, best_score, nodes_searched, nps
            );
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

fn find_immediate_win_or_block(state: &GameState) -> Option<(usize, usize)> {
    let moves = state.get_possible_moves();
    
    for &mv in &moves {
        let mut temp_board = state.board.clone();
        temp_board.place_stone(mv.0, mv.1, state.current_player);
        if crate::core::rules::WinChecker::check_win_around(&temp_board, mv.0, mv.1, state.win_condition) {
            return Some(mv);
        }
    }
    
    let opponent = state.current_player.opponent();
    for &mv in &moves {
        let mut temp_board = state.board.clone();
        temp_board.place_stone(mv.0, mv.1, opponent);
        if crate::core::rules::WinChecker::check_win_around(&temp_board, mv.0, mv.1, state.win_condition) {
            return Some(mv);
        }
    }
    
    None
}
