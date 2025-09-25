use gomoku::ai::heuristic::Heuristic;
use gomoku::ai::minimax::minimax;
use gomoku::ai::pattern_history::{MoveType, PatternHistoryAnalyzer};
use gomoku::ai::transposition::TranspositionTable;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use std::time::Instant;

#[test]
fn test_dynamic_heuristic_basic_functionality() {
    let mut state = GameState::new(19, 5);

    // Make some initial moves
    state.make_move((9, 9));   // Max
    state.make_move((9, 10));  // Min
    state.make_move((9, 8));   // Max
    state.make_move((9, 11));  // Min

    // Verify that pattern analyzer has tracked the moves
    assert_eq!(state.move_history.len(), 4);
    assert_eq!(state.pattern_analyzer.get_recent_patterns().len(), 4);

    // Test that heuristic evaluation includes historical bonus
    let eval_with_history = Heuristic::evaluate(&mut state, 0);
    
    // The evaluation should not crash and should return a reasonable value
    assert!(eval_with_history.abs() < 1_000_000); // Should not be a winning score
}

#[test]
fn test_capture_momentum_bonus() {
    let mut state = GameState::new(19, 5);

    // Set up a capture scenario
    state.make_move((9, 9));   // Max
    state.make_move((9, 10));  // Min
    state.make_move((8, 8));   // Max (dummy move)
    state.make_move((9, 11));  // Min
    
    // Record evaluation before capture
    let eval_before = Heuristic::evaluate(&mut state, 0);
    
    // Make capture move: X-O-O-X pattern
    state.make_move((9, 12));  // Max - this should capture (9,10) and (9,11)
    
    // Record evaluation after capture
    let eval_after = Heuristic::evaluate(&mut state, 0);
    
    // Max should have better evaluation after making a capture
    assert!(eval_after > eval_before);
    
    // Verify capture was recorded in move history
    let recent_patterns = state.pattern_analyzer.get_recent_patterns();
    let last_move = recent_patterns.first().unwrap();
    assert_eq!(last_move.move_type, MoveType::Capture);
    assert!(last_move.captures_made > 0);
}

#[test]
fn test_tempo_and_initiative_tracking() {
    let mut state = GameState::new(19, 5);
    
    // Create a sequence of aggressive moves by Max
    state.make_move((9, 9));   // Max
    state.make_move((10, 10)); // Min
    state.make_move((9, 8));   // Max (extending pattern)
    state.make_move((10, 11)); // Min
    state.make_move((9, 7));   // Max (continuing pattern)
    state.make_move((10, 12)); // Min
    
    // Check that pattern analyzer recognizes the tempo
    let recent_patterns = state.pattern_analyzer.get_recent_patterns();
    
    // Verify moves are being tracked
    assert!(recent_patterns.len() >= 4);
    
    // Evaluate position - Max should get bonus for maintaining initiative
    let evaluation = Heuristic::evaluate(&mut state, 0);
    
    // Should return a valid evaluation (not crash)
    assert!(evaluation.abs() < 1_000_000);
}

#[test]
fn test_pattern_history_analyzer_reset() {
    let mut analyzer = PatternHistoryAnalyzer::new();
    let mut state = GameState::new(19, 5);
    
    // Simulate some moves
    state.make_move((9, 9));
    analyzer.analyze_move_simple((9, 9), Player::Max, 0);
    
    assert!(!analyzer.get_recent_patterns().is_empty());
    
    // Reset analyzer
    analyzer.reset();
    
    // Should be empty after reset
    assert!(analyzer.get_recent_patterns().is_empty());
}

#[test]
fn test_move_history_undo() {
    let mut state = GameState::new(19, 5);
    
    // Make some moves
    state.make_move((9, 9));
    state.make_move((9, 10));
    state.make_move((9, 8));
    
    assert_eq!(state.move_history.len(), 3);
    
    // Undo last move
    state.undo_move((9, 8));
    
    // Move history should be reduced
    assert_eq!(state.move_history.len(), 2);
    
    // Verify the correct move was removed
    assert_eq!(state.move_history.last(), Some(&(9, 10)));
}

#[test]
fn test_historical_bonus_calculation() {
    let state = GameState::new(19, 5);
    
    // Test that historical bonus calculation doesn't crash
    let bonus = state.pattern_analyzer.calculate_historical_bonus(&state);
    
    // Should return a reasonable value (could be 0 for empty game)
    assert!(bonus.abs() < 10_000);
}

#[test]
fn test_move_type_classification() {
    let mut state = GameState::new(19, 5);
    
    // Test basic move classification
    state.make_move((9, 9));   // Max - first move
    
    let recent_patterns = state.pattern_analyzer.get_recent_patterns();
    let last_move = recent_patterns.first().unwrap();
    
    // First move should be classified as positional
    assert_eq!(last_move.move_type, MoveType::Positional);
    assert_eq!(last_move.player, Player::Max);
    assert_eq!(last_move.position, (9, 9));
}

#[test]
fn test_minimax_with_dynamic_heuristic() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new(10_000);
    let start_time = Instant::now();
    
    // Make some initial moves to create history
    state.make_move((9, 9));   // Max
    state.make_move((9, 10));  // Min
    state.make_move((9, 8));   // Max
    state.make_move((9, 11));  // Min
    
    // Store initial state
    let initial_move_history_len = state.move_history.len();
    let initial_pattern_count = state.pattern_analyzer.get_recent_patterns().len();
    let initial_hash = state.hash();
    let initial_player = state.current_player;
    let is_maximizing = initial_player == Player::Max;
    
    // Run minimax - this should not corrupt the state
    let (evaluation, _nodes) = minimax(
        &mut state, 
        2, 
        i32::MIN, 
        i32::MAX, 
        is_maximizing, 
        &mut tt,
        &start_time,
        None
    );
    
    // Verify state is restored correctly after minimax
    assert_eq!(state.move_history.len(), initial_move_history_len);
    assert_eq!(state.pattern_analyzer.get_recent_patterns().len(), initial_pattern_count);
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, initial_player);
    
    // Evaluation should be reasonable (not a winning score)
    assert!(evaluation.abs() < 100_000);
    
    println!("Minimax evaluation with dynamic heuristic: {}", evaluation);
}

#[test]
fn test_minimax_state_consistency_with_patterns() {
    let mut state = GameState::new(19, 5);
    let mut tt = TranspositionTable::new(10_000);
    let start_time = Instant::now();
    
    // Create a more complex position with captures and patterns
    state.make_move((9, 9));   // Max
    state.make_move((9, 10));  // Min
    state.make_move((8, 8));   // Max
    state.make_move((9, 11));  // Min
    state.make_move((9, 12));  // Max - potential capture setup
    
    let pre_search_patterns = state.pattern_analyzer.get_recent_patterns().len();
    let pre_search_moves = state.move_history.len();
    let current_player = state.current_player;
    let is_min_turn = current_player == Player::Min;
    
    // Run deeper minimax
    let (eval1, _) = minimax(
        &mut state, 
        3, 
        i32::MIN, 
        i32::MAX, 
        is_min_turn, 
        &mut tt,
        &start_time,
        None
    );
    
    // State should be unchanged
    assert_eq!(state.pattern_analyzer.get_recent_patterns().len(), pre_search_patterns);
    assert_eq!(state.move_history.len(), pre_search_moves);
    
    // Run again - should get same result
    let (eval2, _) = minimax(
        &mut state, 
        3, 
        i32::MIN, 
        i32::MAX, 
        is_min_turn, 
        &mut tt,
        &start_time,
        None
    );
    
    // Results should be identical
    assert_eq!(eval1, eval2);
}