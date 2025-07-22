use gomoku::ai::minimax::{iterative_deepening_enhanced, iterative_deepening_search};
use gomoku::ai::transposition::TranspositionTable;
use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::find_best_move_enhanced;
use std::time::Duration;

#[test]
fn test_enhanced_move_ordering_performance() {
    let mut state = GameState::new(15, 5);
    
    // Create an interesting position with multiple threats
    state.make_move((7, 7));
    state.make_move((6, 7));
    state.make_move((7, 8));
    state.make_move((6, 8));
    state.make_move((7, 9));
    
    let mut tt_standard = TranspositionTable::new_default();
    let mut tt_enhanced = TranspositionTable::new_default();
    
    // Test standard search
    let start_time = std::time::Instant::now();
    let standard_result = iterative_deepening_search(&mut state.clone(), 6, None, &mut tt_standard);
    let standard_time = start_time.elapsed();
    
    // Test enhanced search
    let start_time = std::time::Instant::now();
    let enhanced_result = iterative_deepening_enhanced(&mut state.clone(), 6, None, &mut tt_enhanced);
    let enhanced_time = start_time.elapsed();
    
    println!("Standard search: {} nodes in {:?}", standard_result.nodes_searched, standard_time);
    println!("Enhanced search: {} nodes in {:?}", enhanced_result.nodes_searched, enhanced_time);
    
    // Enhanced search should generally search fewer nodes due to better move ordering
    // (though this might not always be true depending on the position)
    assert!(enhanced_result.best_move.is_some());
    assert!(enhanced_result.depth_reached >= 5); // Should reach good depth
}

#[test]
fn test_enhanced_vs_standard_consistency() {
    let mut state = GameState::new(15, 5);
    
    // Simple opening position
    state.make_move((7, 7));
    state.make_move((8, 8));
    
    let mut tt_standard = TranspositionTable::new_default();
    let mut tt_enhanced = TranspositionTable::new_default();
    
    let standard_result = iterative_deepening_search(&mut state.clone(), 4, None, &mut tt_standard);
    let enhanced_result = iterative_deepening_enhanced(&mut state.clone(), 4, None, &mut tt_enhanced);
    
    // Both should find reasonable moves
    assert!(standard_result.best_move.is_some());
    assert!(enhanced_result.best_move.is_some());
    
    // Scores should be in similar range (allowing for different move ordering effects)
    let score_diff = (standard_result.score - enhanced_result.score).abs();
    assert!(score_diff < 1000, "Scores should be reasonably similar");
}

#[test]
fn test_enhanced_interface_function() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Make a few moves
    state.make_move((7, 7));
    state.make_move((8, 8));
    
    let best_move = find_best_move_enhanced(&mut state, 5, Some(Duration::from_secs(2)), &mut tt);
    
    assert!(best_move.is_some());
    let (row, col) = best_move.unwrap();
    assert!(row < state.board.size);
    assert!(col < state.board.size);
}

#[test]
fn test_enhanced_deep_search() {
    let mut state = GameState::new(15, 5);
    let mut tt = TranspositionTable::new_default();
    
    // Start with center move
    state.make_move((7, 7));
    
    // Try to reach depth 8 with enhanced search
    let result = iterative_deepening_enhanced(&mut state, 8, Some(Duration::from_secs(5)), &mut tt);
    
    assert!(result.best_move.is_some());
    assert!(result.depth_reached >= 6); // Should reach at least depth 6
    println!("Enhanced search reached depth {} in {:?}", result.depth_reached, result.time_elapsed);
}
