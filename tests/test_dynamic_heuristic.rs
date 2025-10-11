use gomoku::ai::heuristic::Heuristic;
use gomoku::ai::minimax::mtdf;
use gomoku::ai::pattern_history::PatternHistoryAnalyzer;
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
    assert_eq!(state.pattern_analyzer.move_count(), 4);

    // Test that heuristic evaluation includes historical bonus
    let eval_with_history = Heuristic::evaluate(&state, 0);
    
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
    let eval_before = Heuristic::evaluate(&state, 0);
    
    // Make capture move: X-O-O-X pattern
    state.make_move((9, 12));  // Max - this should capture (9,10) and (9,11)
    
    // Record evaluation after capture
    let eval_after = Heuristic::evaluate(&state, 0);
    
    // Max should have better evaluation after making a capture
    assert!(eval_after > eval_before);
    
    // Verify capture was recorded in move history
    let last_move = state.pattern_analyzer.latest_move().unwrap();
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
    let pattern_count = state.pattern_analyzer.move_count();
    
    // Verify moves are being tracked
    assert!(pattern_count >= 4);
    
    // Evaluate position - Max should get bonus for maintaining initiative
    let evaluation = Heuristic::evaluate(&state, 0);
    
    // Should return a valid evaluation (not crash)
    assert!(evaluation.abs() < 1_000_000);
}

#[test]
fn test_pattern_history_analyzer_reset() {
    let mut analyzer = PatternHistoryAnalyzer::new();
    let mut state = GameState::new(19, 5);
    
    // Simulate some moves
    state.make_move((9, 9));
    analyzer.analyze_move(Player::Max, 0);
    
    assert!(analyzer.move_count() > 0);
    
    // Reset analyzer
    analyzer.reset();
    
    // Should be empty after reset
    assert_eq!(analyzer.move_count(), 0);
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
    let bonus = state.pattern_analyzer.calculate_historical_bonus(state.current_player);
    
    // Should return a reasonable value (could be 0 for empty game)
    assert!(bonus.abs() < 10_000);
}

#[test]
fn test_move_type_classification() {
    let mut state = GameState::new(19, 5);
    
    // Test basic move classification
    state.make_move((9, 9));   // Max - first move
    
    let last_move = state.pattern_analyzer.latest_move().unwrap();
    
    // First move should be classified as normal (not capture)
    assert_eq!(last_move.captures_made, 0);
    assert_eq!(last_move.player, Player::Max);
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
    let initial_pattern_count = state.pattern_analyzer.move_count();
    let initial_hash = state.hash();
    let initial_player = state.current_player;
    
    // Run mtdf - this should not corrupt the state
    let (evaluation, _nodes, _) = mtdf(
        &mut state, 
        0, 
        2, 
        &mut tt,
        &start_time,
        None
    );
    
    // Verify state is restored correctly after minimax
    assert_eq!(state.move_history.len(), initial_move_history_len);
    assert_eq!(state.pattern_analyzer.move_count(), initial_pattern_count);
    assert_eq!(state.hash(), initial_hash);
    assert_eq!(state.current_player, initial_player);
    
    // Evaluation should be reasonable (not a winning score)
    assert!(evaluation.abs() < 100_000);
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
    
    let pre_search_patterns = state.pattern_analyzer.move_count();
    let pre_search_moves = state.move_history.len();
    
    // Run deeper mtdf
    let (eval1, _, _) = mtdf(
        &mut state, 
        0, 
        3, 
        &mut tt,
        &start_time,
        None
    );
    
    // State should be unchanged
    assert_eq!(state.pattern_analyzer.move_count(), pre_search_patterns);
    assert_eq!(state.move_history.len(), pre_search_moves);
    
    // Run again - should get same result
    let (eval2, _, _) = mtdf(
        &mut state, 
        eval1, 
        3, 
        &mut tt,
        &start_time,
        None
    );
    
    // Results should be identical
    assert_eq!(eval1, eval2);
}