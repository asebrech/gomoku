use gomoku::core::board::{Board, Player};
use gomoku::ai::move_generation::MoveGenerator;

#[test]
fn test_winning_position_from_image() {
    // Recreating the position from the first image
    // Black has 4 in a row vertically and should win
    let mut board = Board::new(19);
    
    // Based on the image, Black has stones in a vertical line
    // Let's say they're at column 6, rows 1-4 (0-indexed: rows 0-3)
    board.place_stone(0, 6, Player::Max);  // Black (top)
    board.place_stone(1, 6, Player::Max);  // Black
    board.place_stone(3, 6, Player::Max);  // Black
    board.place_stone(4, 6, Player::Max);  // Black (bottom)
    
    // White has some stones but they're not threatening
    board.place_stone(3, 3, Player::Min);  // White
    board.place_stone(4, 3, Player::Min);  // White
    board.place_stone(5, 3, Player::Min);  // White
    board.place_stone(6, 4, Player::Min);  // White
    
    // Black to move - should find the winning move at (2, 6) to complete 5 in a row
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    println!("Generated moves: {:?}", moves);
    
    // Should return only the winning move
    assert_eq!(moves.len(), 1, "Should find exactly one winning move");
    assert_eq!(moves[0], (2, 6), "Should find the winning move at (2, 6)");
}

#[test]
fn test_winning_position_from_second_image() {
    // Recreating the position from the second image
    // Black has 5 in a row vertically - already won
    let mut board = Board::new(19);
    
    // Black has 5 stones vertically at column 6
    board.place_stone(0, 6, Player::Max);  // Black
    board.place_stone(1, 6, Player::Max);  // Black
    board.place_stone(2, 6, Player::Max);  // Black
    board.place_stone(3, 6, Player::Max);  // Black
    board.place_stone(4, 6, Player::Max);  // Black (5 in a row!)
    
    // White has some stones
    board.place_stone(3, 3, Player::Min);  // White
    board.place_stone(4, 3, Player::Min);  // White
    board.place_stone(5, 3, Player::Min);  // White
    board.place_stone(6, 4, Player::Min);  // White
    
    // This position should be recognized as already won by Black
    // But let's check move generation doesn't crash
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);
    
    println!("Moves after Black won: {:?}", moves);
    
    // Should still generate moves (game end detection is done elsewhere)
    assert!(!moves.is_empty(), "Should still generate moves for White");
}

#[test]
fn test_pattern_with_gap_like_image() {
    // Testing a pattern similar to image where there's a gap: X X X _ X
    let mut board = Board::new(19);
    
    // Black pattern: X X X _ X (vertical)
    board.place_stone(5, 8, Player::Max);
    board.place_stone(6, 8, Player::Max);
    board.place_stone(7, 8, Player::Max);
    // Gap at (8, 8)
    board.place_stone(9, 8, Player::Max);
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    println!("Moves for gap pattern: {:?}", moves);
    
    // Should find (8, 8) as winning move (fills the gap to make 5)
    assert!(!moves.is_empty(), "Should find winning move");
    assert!(moves.contains(&(8, 8)), "Should find the gap-filling winning move at (8, 8)");
}

#[test]
fn test_endpoint_completion_like_image() {
    // Testing endpoint completion: X X X X _ (vertical)
    let mut board = Board::new(19);
    
    // Black pattern: 4 in a row with empty endpoint
    board.place_stone(5, 8, Player::Max);
    board.place_stone(6, 8, Player::Max);
    board.place_stone(7, 8, Player::Max);
    board.place_stone(8, 8, Player::Max);
    // Empty at (9, 8) or (4, 8)
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    println!("Moves for endpoint pattern: {:?}", moves);
    
    // Should find either endpoint as winning move
    assert_eq!(moves.len(), 1, "Should find exactly one winning move");
    assert!(moves[0] == (9, 8) || moves[0] == (4, 8), 
            "Should find winning move at endpoint, got {:?}", moves[0]);
}

#[test]
fn test_both_endpoints_available() {
    // Testing when both endpoints are available: _ X X X X _
    let mut board = Board::new(19);
    
    // Black pattern: 4 in a row with both endpoints empty
    board.place_stone(6, 8, Player::Max);
    board.place_stone(7, 8, Player::Max);
    board.place_stone(8, 8, Player::Max);
    board.place_stone(9, 8, Player::Max);
    // Empty at both (5, 8) and (10, 8)
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    println!("Moves for both endpoints available: {:?}", moves);
    
    // Should find exactly one winning move (implementation picks first found)
    assert_eq!(moves.len(), 1, "Should find exactly one winning move");
    assert!(moves[0] == (10, 8) || moves[0] == (5, 8), 
            "Should find winning move at either endpoint, got {:?}", moves[0]);
}
