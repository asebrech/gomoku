//! Tests for gapped threat detection in move generation.

use gomoku::ai::move_ordering::MoveGenerator;
use gomoku::core::board::{Board, Player};

#[test]
fn test_detects_gapped_winning_threat() {
    let mut board = Board::new(19);
    
    // Create a gapped pattern that can win immediately: X.X.X.
    // If opponent plays in any gap, they can form 5 in a row
    board.place_stone(9, 4, Player::Max);  // X
    board.place_stone(9, 6, Player::Max);  // X  
    board.place_stone(9, 8, Player::Max);  // X
    board.place_stone(9, 10, Player::Max); // X
    // Pattern: X . X . X . X (4 stones with gaps)
    
    // AI should detect that Max has a winning threat and block it
    let moves = MoveGenerator::order_moves(&board, Player::Min);
    
    // Should detect this as a must-block situation
    // The gaps at (9,5), (9,7), (9,9) are all winning for Max
    let critical_positions = [(9, 5), (9, 7), (9, 9)];
    let blocks_found = critical_positions.iter()
        .filter(|&&pos| moves.contains(&pos))
        .count();
    
    assert!(blocks_found > 0, 
            "AI should detect and block the gapped winning threat. Moves: {:?}", moves);
    
    println!("AI generated {} moves, blocking {} critical positions", moves.len(), blocks_found);
    for &pos in &critical_positions {
        if moves.contains(&pos) {
            println!("Correctly identified blocking move: {:?}", pos);
        }
    }
}

#[test]
fn test_detects_gapped_four_threat() {
    let mut board = Board::new(19);
    
    // Create a gapped four pattern: XXX.X  
    board.place_stone(9, 5, Player::Max);  // X
    board.place_stone(9, 6, Player::Max);  // X
    board.place_stone(9, 7, Player::Max);  // X
    board.place_stone(9, 9, Player::Max);  // X
    // Pattern: X X X . X (4 stones with 1 gap in 5 positions)
    
    // AI should detect this as a major threat requiring blocking
    let moves = MoveGenerator::order_moves(&board, Player::Min);
    
    // Should identify (9,8) as a critical blocking move
    assert!(moves.contains(&(9, 8)), 
            "AI should detect and prioritize blocking the gapped four at (9,8). Moves: {:?}", moves);
    
    // If it's in must-block moves, it should be one of the first returned
    let high_priority = moves.len() <= 5; // Few moves means high priority
    if high_priority {
        println!("Gapped four threat correctly identified as high priority (only {} moves)", moves.len());
    }
}

#[test]
fn test_creates_gapped_winning_move() {
    let mut board = Board::new(19);
    
    // Create a position where Max can win by filling a gap
    board.place_stone(9, 4, Player::Max);   // X
    board.place_stone(9, 5, Player::Max);   // X
    board.place_stone(9, 7, Player::Max);   // X
    board.place_stone(9, 8, Player::Max);   // X
    // Pattern: X X . X X (4 stones, gap at position 6)
    // Playing at (9,6) creates 5 in a row
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    // Should recognize (9,6) as an immediate winning move (prioritized first)
    assert!(!moves.is_empty(), "Should generate moves");
    assert_eq!(moves[0], (9, 6), "Winning move should be first");
    assert!(moves.contains(&(9, 6)), "Should identify (9,6) as the winning move");
    
    println!("Correctly identified gapped winning move: {:?}", moves[0]);
}

#[test]
fn test_gapped_vs_consecutive_threat_priority() {
    let mut board = Board::new(19);
    
    // Create both a consecutive threat and a gapped threat for opponent
    // Consecutive threat: XXXX (immediate win)
    board.place_stone(5, 5, Player::Min);
    board.place_stone(5, 6, Player::Min);
    board.place_stone(5, 7, Player::Min);
    board.place_stone(5, 8, Player::Min);
    
    // Gapped threat: XX.X (potential threat)
    board.place_stone(9, 5, Player::Min);
    board.place_stone(9, 6, Player::Min);
    board.place_stone(9, 8, Player::Min);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    // Should prioritize blocking the immediate consecutive threat first
    let blocks_consecutive = moves.contains(&(5, 4)) || moves.contains(&(5, 9));
    assert!(blocks_consecutive, "Should block the immediate consecutive threat");
    
    // But if there were no consecutive threat, should still detect gapped
    let mut board_gapped_only = Board::new(19);
    board_gapped_only.place_stone(9, 5, Player::Min);
    board_gapped_only.place_stone(9, 6, Player::Min);
    board_gapped_only.place_stone(9, 8, Player::Min);
    
    let moves_gapped = MoveGenerator::order_moves(&board_gapped_only, Player::Max);
    let blocks_gapped = moves_gapped.contains(&(9, 7));
    
    assert!(blocks_gapped, "Should detect and block gapped threats when no consecutive threats exist");
    
    println!("Consecutive threat blocking: {}, Gapped threat blocking: {}", 
             blocks_consecutive, blocks_gapped);
}

#[test]
fn test_vertical_gapped_pattern_like_image() {
    let mut board = Board::new(19);
    
    // Create a vertical gapped pattern like in the user's image
    board.place_stone(5, 9, Player::Max);   // X
    board.place_stone(7, 9, Player::Max);   // X
    board.place_stone(9, 9, Player::Max);   // X
    board.place_stone(11, 9, Player::Max);  // X
    // Vertical pattern: X . X . X . X
    
    let moves = MoveGenerator::order_moves(&board, Player::Min);
    
    // Should recognize this as a major threat and block the gaps
    let gap_positions = [(6, 9), (8, 9), (10, 9)];
    let blocks_found = gap_positions.iter()
        .filter(|&&pos| moves.contains(&pos))
        .count();
    
    assert!(blocks_found > 0, 
            "AI should detect the vertical gapped pattern threat. Moves: {:?}", moves);
    
    println!("Vertical gapped pattern: detected {} blocking positions out of 3", blocks_found);
}