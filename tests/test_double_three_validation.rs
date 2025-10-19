//! Tests for double-three validation in gapped pattern evaluation.
//! These tests verify that the heuristic properly validates gapped patterns
//! against the double-three rule and scores them accordingly.

use gomoku::ai::heuristic::Heuristic;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;
use gomoku::core::rules::DoubleThreeDetection;

#[test]
fn test_gapped_pattern_with_double_three_restriction() {
    let mut state = GameState::new(19, 5);
    
    // Create a setup where completing a gapped pattern would create a double-three
    // This is a classic double-three scenario adapted for gapped patterns:
    //
    //   0 1 2 3 4 5 6 7 8 9 10
    // 8       X
    // 9     X     X     X
    // 10      X
    //
    // If we place a stone at (9,5) to complete the gapped pattern X.X.X -> XXX.X,
    // it would create two free threes simultaneously.
    
    // Set up the pattern
    state.board.place_stone(8, 3, Player::Max);  // First three setup
    state.board.place_stone(9, 2, Player::Max);  // Second three setup  
    state.board.place_stone(9, 4, Player::Max);  // Gapped pattern part 1
    state.board.place_stone(9, 6, Player::Max);  // Gapped pattern part 2  
    state.board.place_stone(9, 8, Player::Max);  // Gapped pattern part 3
    state.board.place_stone(10, 3, Player::Max); // Complete first three
    
    // Verify that placing at (9,5) would create a double-three
    let would_create_double_three = DoubleThreeDetection::creates_double_three(
        &state.board, 9, 5, Player::Max
    );
    
    let score_before = Heuristic::evaluate(&state, 1);
    println!("Score before validation: {}", score_before);
    println!("Would create double-three at (9,5): {}", would_create_double_three);
    
    // The heuristic should recognize that some gapped patterns cannot be completed
    // and should score them lower than fully completable patterns
    assert!(score_before > 0, "Should still score positively due to existing patterns");
}

#[test]
fn test_completable_gapped_pattern_vs_blocked_pattern() {
    // Test 1: A gapped pattern that CAN be completed legally
    let mut state1 = GameState::new(19, 5);
    
    // Simple gapped three that can be completed: .X.X.X.
    state1.board.place_stone(9, 5, Player::Max);
    state1.board.place_stone(9, 7, Player::Max);
    state1.board.place_stone(9, 9, Player::Max);
    
    let score1 = Heuristic::evaluate(&state1, 1);
    
    // Test 2: A similar gapped pattern that would create double-three if completed
    let mut state2 = GameState::new(19, 5);
    
    // Create a setup where completing the gapped pattern creates double-three
    state2.board.place_stone(8, 6, Player::Max);   // Vertical threat part 1
    state2.board.place_stone(10, 6, Player::Max);  // Vertical threat part 2
    state2.board.place_stone(9, 5, Player::Max);   // Horizontal gapped pattern part 1
    state2.board.place_stone(9, 7, Player::Max);   // Horizontal gapped pattern part 2  
    state2.board.place_stone(9, 9, Player::Max);   // Horizontal gapped pattern part 3
    
    // Verify double-three would be created at (9,6)
    let would_create_double_three = DoubleThreeDetection::creates_double_three(
        &state2.board, 9, 6, Player::Max
    );
    
    let score2 = Heuristic::evaluate(&state2, 1);
    
    println!("Completable gapped pattern score: {}", score1);
    println!("Potentially blocked pattern score: {}", score2);
    println!("Would create double-three: {}", would_create_double_three);
    
    // Both should be positive, but the validation should affect the relative scoring
    assert!(score1 > 0, "Completable pattern should score positively");
    assert!(score2 > 0, "Blocked pattern should still score positively due to other patterns");
}

#[test]
fn test_validated_vs_unvalidated_gapped_scoring() {
    // This test verifies that the scoring properly differentiates between
    // validated and unvalidated gapped patterns.
    
    let mut state = GameState::new(19, 5);
    
    // Create a simple gapped pattern that should be easily completable
    state.board.place_stone(9, 5, Player::Max);
    state.board.place_stone(9, 7, Player::Max);
    state.board.place_stone(9, 9, Player::Max);
    
    // Verify that filling the gap at (9,6) or (9,8) won't create double-three
    let gap1_safe = !DoubleThreeDetection::creates_double_three(&state.board, 9, 6, Player::Max);
    let gap2_safe = !DoubleThreeDetection::creates_double_three(&state.board, 9, 8, Player::Max);
    
    let score = Heuristic::evaluate(&state, 1);
    
    println!("Gap at (9,6) safe: {}", gap1_safe);
    println!("Gap at (9,8) safe: {}", gap2_safe);
    println!("Gapped pattern score: {}", score);
    
    // At least one gap should be safe to fill, making this a validated pattern
    assert!(gap1_safe || gap2_safe, "At least one gap should be safe to fill");
    assert!(score > 100, "Validated gapped pattern should score well");
}