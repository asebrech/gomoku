//! Tests for gapped pattern scoring in pattern utilities.

use gomoku::ai::pattern_utils::get_gapped_pattern_score;
use gomoku::core::board::{Board, Player};

#[test]
fn test_gapped_pattern_score_x_dot_x_dot_x() {
    let mut board = Board::new(19);
    
    // Create pattern X.X.X horizontally at row 9
    board.place_stone(9, 5, Player::Max);  // X
    board.place_stone(9, 7, Player::Max);  // X
    board.place_stone(9, 9, Player::Max);  // X
    
    // Test scoring from the middle stone
    let score = get_gapped_pattern_score(&board, 9, 7, 0, 1, Player::Max);
    assert!(score > 0, "X.X.X pattern should have positive score, got {}", score);
    
    // Should be less than a solid three but still valuable
    assert!(score >= 50, "X.X.X pattern should be valuable enough, got {}", score);
}

#[test]
fn test_gapped_pattern_score_x_dot_dot_x() {
    let mut board = Board::new(19);
    
    // Create pattern X..X vertically
    board.place_stone(5, 9, Player::Max);  // X
    board.place_stone(8, 9, Player::Max);  // X
    
    // Test scoring from either stone
    let score1 = get_gapped_pattern_score(&board, 5, 9, 1, 0, Player::Max);
    let score2 = get_gapped_pattern_score(&board, 8, 9, 1, 0, Player::Max);
    
    assert!(score1 > 0 || score2 > 0, "X..X pattern should have positive score");
    let max_score = score1.max(score2);
    assert!(max_score >= 15, "X..X pattern should have some value, got {}", max_score);
}

#[test]
fn test_gapped_pattern_score_xx_dot_x() {
    let mut board = Board::new(19);
    
    // Create pattern XX.X horizontally
    board.place_stone(9, 5, Player::Max);   // X
    board.place_stone(9, 6, Player::Max);   // X
    board.place_stone(9, 8, Player::Max);   // X
    
    // Test scoring - this should be quite valuable (3 stones in 4 spaces)
    let score = get_gapped_pattern_score(&board, 9, 6, 0, 1, Player::Max);
    assert!(score > 0, "XX.X pattern should have positive score, got {}", score);
    assert!(score >= 100, "XX.X pattern should be quite valuable, got {}", score);
}

#[test]
fn test_gapped_pattern_score_four_stones_with_gap() {
    let mut board = Board::new(19);
    
    // Create pattern XXX.X (4 stones with 1 gap)
    board.place_stone(9, 5, Player::Max);  // X
    board.place_stone(9, 6, Player::Max);  // X
    board.place_stone(9, 7, Player::Max);  // X
    board.place_stone(9, 9, Player::Max);  // X
    
    // This should be very valuable - almost winning
    let score = get_gapped_pattern_score(&board, 9, 6, 0, 1, Player::Max);
    assert!(score > 0, "XXX.X pattern should have positive score, got {}", score);
    assert!(score >= 500, "XXX.X pattern should be very valuable, got {}", score);
}

#[test]
fn test_gapped_pattern_score_blocked_pattern() {
    let mut board = Board::new(19);
    
    // Create pattern X.X with opponent blocking
    board.place_stone(9, 5, Player::Max);  // X
    board.place_stone(9, 7, Player::Max);  // X
    board.place_stone(9, 4, Player::Min);  // O (blocking)
    
    // Should still have some score but reduced
    let score = get_gapped_pattern_score(&board, 9, 5, 0, 1, Player::Max);
    assert!(score > 0, "Blocked X.X pattern should still have some score, got {}", score);
    
    // Should be less than a fully free pattern
    let mut free_board = Board::new(19);
    free_board.place_stone(9, 5, Player::Max);
    free_board.place_stone(9, 7, Player::Max);
    let free_score = get_gapped_pattern_score(&free_board, 9, 5, 0, 1, Player::Max);
    
    assert!(score <= free_score, "Blocked pattern should score less than free pattern");
}

#[test]
fn test_gapped_pattern_score_no_pattern() {
    let mut board = Board::new(19);
    
    // Place isolated stones that don't form patterns
    board.place_stone(5, 5, Player::Max);
    board.place_stone(15, 15, Player::Max);
    
    // Should not score gapped patterns for isolated stones
    let score = get_gapped_pattern_score(&board, 5, 5, 0, 1, Player::Max);
    assert_eq!(score, 0, "Isolated stones should not form gapped patterns, got {}", score);
}

#[test]
fn test_gapped_pattern_score_diagonal() {
    let mut board = Board::new(19);
    
    // Create diagonal gapped pattern
    board.place_stone(7, 7, Player::Max);   // X
    board.place_stone(9, 9, Player::Max);   // X
    board.place_stone(11, 11, Player::Max); // X
    
    // Test diagonal direction
    let score = get_gapped_pattern_score(&board, 9, 9, 1, 1, Player::Max);
    assert!(score > 0, "Diagonal X.X.X pattern should have positive score, got {}", score);
}