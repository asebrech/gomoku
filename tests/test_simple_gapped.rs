//! Simple test for basic gapped pattern move generation.

use gomoku::ai::move_ordering::MoveGenerator;
use gomoku::core::board::{Board, Player};

#[test]
fn test_simple_gapped_pattern_moves() {
    let mut board = Board::new(19);
    
    // Create a simple gapped pattern like in the image: X.X.X vertically
    board.place_stone(6, 9, Player::Max);   // X
    board.place_stone(8, 9, Player::Max);   // X  
    board.place_stone(10, 9, Player::Max);  // X
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    // Should include the gaps at (7,9) and (9,9) as candidate moves
    assert!(moves.contains(&(7, 9)), "Should include gap at (7,9)");
    assert!(moves.contains(&(9, 9)), "Should include gap at (9,9)");
    
    println!("Generated {} moves for gapped pattern", moves.len());
    for &(row, col) in &moves {
        if col == 9 && row >= 5 && row <= 11 {
            println!("Move in pattern area: ({}, {})", row, col);
        }
    }
}

#[test]
fn test_horizontal_gapped_pattern() {
    let mut board = Board::new(19);
    
    // Create horizontal gapped pattern: X.X.X
    board.place_stone(9, 5, Player::Max);   // X
    board.place_stone(9, 7, Player::Max);   // X  
    board.place_stone(9, 9, Player::Max);   // X
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    // Should include the gaps
    assert!(moves.contains(&(9, 6)), "Should include gap at (9,6)");
    assert!(moves.contains(&(9, 8)), "Should include gap at (9,8)");
    
    println!("Horizontal gapped pattern generated {} moves", moves.len());
}

#[test]
fn test_no_false_gaps() {
    let mut board = Board::new(19);
    
    // Place isolated stones that shouldn't create gap moves
    board.place_stone(5, 5, Player::Max);
    board.place_stone(15, 15, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    // Shouldn't suggest random positions between isolated stones
    assert!(!moves.contains(&(10, 10)), "Should not suggest random gaps between isolated stones");
    
    println!("Isolated stones generated {} moves (no false gaps)", moves.len());
}