use gomoku::ai::transposition::TranspositionTable;
use gomoku::ai::minimax::{iterative_deepening_search, SearchResult};
use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::{find_best_move_iterative, find_best_move_timed};
use std::time::Duration;

#[test]
fn test_iterative_deepening_basic() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Make a few moves to create a non-trivial position
    state.make_move((7, 7)); // Center
    state.make_move((7, 8)); // Adjacent
    state.make_move((7, 6)); // Other side
    
    let result = iterative_deepening_search(&mut state, 3, None, &mut tt);
    
    assert!(result.best_move.is_some());
    assert!(result.depth_reached > 0);
    assert!(result.depth_reached <= 3);
    assert!(result.nodes_searched > 0);
    println!("Basic test result: {:?}", result);
}

#[test]
fn test_iterative_deepening_time_limit() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Make a few moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    
    let time_limit = Duration::from_millis(100);
    let result = iterative_deepening_search(&mut state, 10, Some(time_limit), &mut tt);
    
    assert!(result.best_move.is_some());
    assert!(result.time_elapsed <= Duration::from_millis(150)); // Allow some margin
    println!("Timed test result: {:?}", result);
}

#[test]
fn test_find_best_move_iterative() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Make some moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    
    let best_move = find_best_move_iterative(&mut state, 3, &mut tt);
    assert!(best_move.is_some());
    println!("Best move from iterative search: {:?}", best_move);
}

#[test]
fn test_find_best_move_timed() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Make some moves
    state.make_move((7, 7));
    state.make_move((7, 8));
    
    let time_limit = Duration::from_millis(50);
    let best_move = find_best_move_timed(&mut state, 5, time_limit, &mut tt);
    assert!(best_move.is_some());
    println!("Best move from timed search: {:?}", best_move);
}

#[test]
fn test_iterative_deepening_vs_direct_minimax() {
    let mut state = GameState::new(15, 5);
    let mut tt1 = TranspositionTable::new_default();
    let mut tt2 = TranspositionTable::new_default();
    
    // Create identical game states
    let moves = [(7, 7), (7, 8), (8, 7)];
    for &mv in &moves {
        state.make_move(mv);
    }
    
    // Test iterative deepening
    let iterative_result = iterative_deepening_search(&mut state, 3, None, &mut tt1);
    
    // Test direct search using the legacy function
    let legacy_result = gomoku::interface::utils::find_best_move_legacy(&mut state, 3, &mut tt2);
    
    // Both should find a move
    assert!(iterative_result.best_move.is_some());
    assert!(legacy_result.is_some());
    
    println!("Iterative result: {:?}", iterative_result.best_move);
    println!("Legacy result: {:?}", legacy_result);
    
    // The moves might be different due to different search strategies, but both should be valid
    let moves = state.get_possible_moves();
    assert!(moves.contains(&iterative_result.best_move.unwrap()));
    assert!(moves.contains(&legacy_result.unwrap()));
}

#[test]
fn test_early_termination_on_win() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Create a position where there's an immediate winning move
    // Set up a line of 4 for the current player
    state.make_move((7, 7)); // Max
    state.make_move((8, 7)); // Min  
    state.make_move((7, 8)); // Max
    state.make_move((8, 8)); // Min
    state.make_move((7, 9)); // Max
    state.make_move((8, 9)); // Min
    state.make_move((7, 10)); // Max - now has 4 in a row
    
    // Min should find the blocking move immediately
    let result = iterative_deepening_search(&mut state, 5, None, &mut tt);
    
    assert!(result.best_move.is_some());
    // Should find the winning/blocking move quickly
    assert!(result.score.abs() >= 1000); // High score for critical position
    println!("Early termination test result: {:?}", result);
}
