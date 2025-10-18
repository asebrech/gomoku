//! Tests for shared pattern analysis utilities.

use gomoku::core::patterns::{PatternAnalyzer, PatternFreedom};
use gomoku::core::board::{Board, Player};

#[test]
fn test_analyze_pattern_freedom() {
    let mut board = Board::new(19);

    // Test free pattern: . X X X .
    board.place_stone(9, 6, Player::Max);
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);

    let freedom = PatternAnalyzer::analyze_pattern_freedom(&board, 9, 6, 0, 1, 3);
    assert_eq!(freedom, PatternFreedom::Free);

    // Test half-free pattern: O X X X .
    board.place_stone(9, 5, Player::Min); // Block one end
    let freedom = PatternAnalyzer::analyze_pattern_freedom(&board, 9, 6, 0, 1, 3);
    assert_eq!(freedom, PatternFreedom::HalfFree);

    // Test flanked pattern: O X X X O
    board.place_stone(9, 9, Player::Min); // Block other end
    let freedom = PatternAnalyzer::analyze_pattern_freedom(&board, 9, 6, 0, 1, 3);
    assert_eq!(freedom, PatternFreedom::Flanked);
}

#[test]
fn test_count_total_space() {
    let mut board = Board::new(19);

    // Test pattern with plenty of space: . . X X X . .
    board.place_stone(9, 6, Player::Max);
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);

    let space = PatternAnalyzer::count_total_space(&board, 9, 6, 0, 1, 3);
    // Should have space for pattern (3) + empty spaces on both sides
    assert!(space >= 5, "Should have at least 5 spaces for win condition");

    // Test pattern near edge with limited space
    let mut small_board = Board::new(4);
    small_board.place_stone(1, 1, Player::Max);
    small_board.place_stone(1, 2, Player::Max);

    let space = PatternAnalyzer::count_total_space(&small_board, 1, 1, 0, 1, 2);
    // On 4x4 board, pattern starting at (1,1) with length 2 should have total space of 4
    assert_eq!(space, 4, "Pattern on 4x4 board should have exactly 4 spaces");
}

#[test]
fn test_count_empty_in_direction() {
    let mut board = Board::new(19);

    // Place stones to create pattern: X X . . . O
    board.place_stone(9, 5, Player::Max);
    board.place_stone(9, 6, Player::Max);
    board.place_stone(9, 9, Player::Min); // Blocking stone

    // Count empty spaces to the right from position (9, 7)
    let empty_count = PatternAnalyzer::count_empty_in_direction(&board, 9, 7, 0, 1);
    assert_eq!(empty_count, 2, "Should count 2 empty spaces before hitting the blocking stone");

    // Count empty spaces to the left from position (9, 4) 
    let empty_count = PatternAnalyzer::count_empty_in_direction(&board, 9, 4, 0, -1);
    assert_eq!(empty_count, 5, "Should count 5 empty spaces to the left before hitting board edge");
    
    // Count from position right after stones - should count 1 empty space then hit stone
    let empty_count = PatternAnalyzer::count_empty_in_direction(&board, 9, 7, 0, -1);
    assert_eq!(empty_count, 1, "Should count 1 empty space before hitting the stone at (9,6)");
}

#[test]
fn test_pattern_utils_consistency() {
    // Test that the shared utilities produce consistent results
    let mut board = Board::new(19);
    
    // Create a test pattern
    board.place_stone(9, 6, Player::Max);
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Max);

    let freedom = PatternAnalyzer::analyze_pattern_freedom(&board, 9, 6, 0, 1, 4);
    let space = PatternAnalyzer::count_total_space(&board, 9, 6, 0, 1, 4);

    // Verify values are reasonable
    assert!(space >= 4, "Pattern should have at least its own length in space");
    assert_ne!(freedom, PatternFreedom::Flanked, "Pattern should not be flanked on empty board");
}