use crate::ai::{minimax::{minimax, iterative_deepening_search}, transposition::TranspositionTable};
use crate::core::state::GameState;
use crate::core::board::Player;
use crate::core::rules::WinChecker;
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

    // Record initial stats for this search
    //let (initial_hits, initial_misses, initial_collisions) = tt.get_stats();

    let mut moves = state.get_possible_moves();
    prioritize_defensive_moves(state, &mut moves);

    for mv in moves {
        state.make_move(mv);
        let score = minimax(
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

    // Print transposition table statistics for this search
    /*let (final_hits, final_misses, final_collisions) = tt.get_stats();
    let search_hits = final_hits - initial_hits;
    let search_misses = final_misses - initial_misses;
    let search_collisions = final_collisions - initial_collisions;
    let total_probes = search_hits + search_misses;
    
    if total_probes > 0 {
        let hit_rate = search_hits as f64 / total_probes as f64 * 100.0;
        println!(
            "🔍 AI Search Stats - Depth: {}, Player: {:?}, TT Hit Rate: {:.1}% ({}/{} probes), Collisions: {}, Table Size: {}",
            depth, current_player, hit_rate, search_hits, total_probes, search_collisions, tt.size()
        );
    }*/

    best_move
}

pub fn find_best_move_iterative(
    state: &mut GameState, 
    max_depth: i32, 
    tt: &mut TranspositionTable
) -> Option<(usize, usize)> {
    let result = iterative_deepening_search(state, max_depth, None, tt);
    
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
    
    println!(
        "⏱️  Timed search completed: depth={}/{}, score={}, nodes={}, time={:?}",
        result.depth_reached, max_depth, result.score, result.nodes_searched, result.time_elapsed
    );
    
    result.best_move
}

fn prioritize_defensive_moves(state: &GameState, moves: &mut Vec<(usize, usize)>) {
    let opponent = state.current_player.opponent();
    let mut threat_blocking_moves = Vec::new();
    let mut other_moves = Vec::new();
    
    for &mv in moves.iter() {
        if blocks_immediate_threat(state, mv, opponent) {
            threat_blocking_moves.push(mv);
        } else {
            other_moves.push(mv);
        }
    }
    
    moves.clear();
    moves.extend(threat_blocking_moves);
    moves.extend(other_moves);
}

fn blocks_immediate_threat(state: &GameState, mv: (usize, usize), opponent: Player) -> bool {
    let mut test_state = state.clone();
    test_state.board.place_stone(mv.0, mv.1, opponent);
    
    WinChecker::check_win_around(&test_state.board, mv.0, mv.1, state.win_condition)
}
