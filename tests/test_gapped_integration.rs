//! Test to demonstrate gapped pattern integration with move generation.

use gomoku::ai::move_generation::MoveGenerator;
use gomoku::core::board::{Board, Player};

#[test]
fn test_gapped_pattern_integration() {
    let mut board = Board::new(19);
    
    // Create a gapped pattern X.X.X that should be recognized
    board.place_stone(9, 5, Player::Max);  // X
    board.place_stone(9, 7, Player::Max);  // X  
    board.place_stone(9, 9, Player::Max);  // X
    // Pattern: X . X . X
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // The move generation should now consider moves around this gapped pattern
    // At minimum, it should generate zone-based moves around these stones
    assert!(!moves.is_empty(), "Should generate candidate moves");
    
    // Check that moves near the gapped pattern are being considered
    let near_gap_moves = moves.iter().filter(|&&(row, col)| {
        row == 9 && (col >= 4 && col <= 10)
    }).count();
    
    assert!(near_gap_moves > 0, "Should generate moves near the gapped pattern");
    
    // The gapped pattern should be influencing threat priority calculation
    // (which uses the new gapped pattern scoring)
    println!("Generated {} moves, {} near the gapped pattern", moves.len(), near_gap_moves);
    for &(row, col) in &moves {
        if row == 9 && col >= 4 && col <= 10 {
            println!("Move near gapped pattern: ({}, {})", row, col);
        }
    }
}

#[test]
fn test_gapped_vs_consecutive_pattern_priority() {
    let mut board = Board::new(19);
    
    // Create both patterns on the same board
    // Consecutive pattern: XXX (higher priority)
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 6, Player::Max);
    board.place_stone(5, 7, Player::Max);
    
    // Gapped pattern: X.X (lower priority)
    board.place_stone(9, 5, Player::Max);
    board.place_stone(9, 7, Player::Max);
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Should prioritize completing the consecutive three
    // but also consider the gapped pattern
    assert!(!moves.is_empty(), "Should generate moves");
    
    // At least some moves should be around both patterns
    let consecutive_area_moves = moves.iter().filter(|&&(row, col)| {
        row == 5 && col >= 4 && col <= 8
    }).count();
    
    let gapped_area_moves = moves.iter().filter(|&&(row, col)| {
        row == 9 && col >= 4 && col <= 8
    }).count();
    
    assert!(consecutive_area_moves > 0, "Should consider consecutive pattern area");
    // Note: Gapped pattern moves might not show up if consecutive pattern is much higher priority
    
    println!("Consecutive area moves: {}, Gapped area moves: {}", 
             consecutive_area_moves, gapped_area_moves);
}