//! Test to verify AI detects gapped threats like in the user's image.

use gomoku::ai::move_ordering::MoveGenerator;
use gomoku::core::board::{Board, Player};

#[test]
fn test_ai_blocks_gapped_threat_from_image() {
    let mut board = Board::new(19);
    
    // Create the gapped pattern from the user's image - vertical X.X.X.X
    board.place_stone(6, 9, Player::Max);   // X
    board.place_stone(8, 9, Player::Max);   // X  
    board.place_stone(10, 9, Player::Max);  // X
    board.place_stone(12, 9, Player::Max);  // X
    // Pattern: X . X . X . X (4 stones with gaps that could become 5)
    
    // AI playing as Min should detect this as a must-block threat
    let moves = MoveGenerator::order_moves(&board, Player::Min);
    
    // Should detect the gaps as critical blocking positions
    let gap_positions = [(7, 9), (9, 9), (11, 9)];
    let blocks_found = gap_positions.iter()
        .filter(|&&pos| moves.contains(&pos))
        .count();
    
    assert!(blocks_found > 0, 
            "AI should detect the gapped threat and block at least one gap. Found moves: {:?}", 
            moves);
    
    // Should be high priority (few moves = must-block detected)
    assert!(moves.len() <= 10, 
            "Should be high priority blocking (few moves), got {} moves", moves.len());
    
    println!("SUCCESS: AI detected gapped threat, generated {} moves, blocking {} gaps", 
             moves.len(), blocks_found);
    for &pos in &gap_positions {
        if moves.contains(&pos) {
            println!("  ✓ Blocking gap at {:?}", pos);
        }
    }
}

#[test]
fn test_ai_creates_gapped_winning_move() {
    let mut board = Board::new(19);
    
    // Create a position where Max can win by filling a gap in X.X.X.X pattern
    board.place_stone(6, 9, Player::Max);   // X
    board.place_stone(8, 9, Player::Max);   // X  
    board.place_stone(10, 9, Player::Max);  // X
    board.place_stone(12, 9, Player::Max);  // X
    // Playing at (7,9), (9,9), or (11,9) could create a winning line
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    // Should include the gap positions as candidate moves
    let gap_positions = [(7, 9), (9, 9), (11, 9)];
    let gaps_included = gap_positions.iter()
        .filter(|&&pos| moves.contains(&pos))
        .count();
    
    assert!(gaps_included > 0, 
            "AI should consider filling gaps in the winning pattern. Moves: {:?}", moves);
    
    println!("AI considers {} out of 3 gap positions for winning", gaps_included);
}

#[test]
fn test_horizontal_gapped_threat() {
    let mut board = Board::new(19);
    
    // Create horizontal gapped threat: X.X.X.X
    board.place_stone(9, 5, Player::Max);   // X
    board.place_stone(9, 7, Player::Max);   // X  
    board.place_stone(9, 9, Player::Max);   // X
    board.place_stone(9, 11, Player::Max);  // X
    
    let moves = MoveGenerator::order_moves(&board, Player::Min);
    
    // Should block the horizontal gaps
    let gap_positions = [(9, 6), (9, 8), (9, 10)];
    let blocks_found = gap_positions.iter()
        .filter(|&&pos| moves.contains(&pos))
        .count();
    
    assert!(blocks_found > 0, 
            "AI should detect horizontal gapped threat. Found moves: {:?}", moves);
    
    println!("Horizontal gapped threat: AI blocks {} out of 3 gaps", blocks_found);
}