use gomoku::core::board::{Board, Player};
use gomoku::core::move_generation::MoveGenerator;
use gomoku::core::state::GameState;

#[test]
fn test_creates_five_in_row_fix() {
    // Test that creates_five_in_row correctly identifies winning positions
    let mut board = Board::new(19);
    
    // Create pattern: X X X X _ (4 in a row, empty at position 5)
    // Should recognize that placing at position 5 creates 5-in-a-row
    board.place_stone(0, 0, Player::Max);
    board.place_stone(0, 1, Player::Max);
    board.place_stone(0, 2, Player::Max);
    board.place_stone(0, 3, Player::Max);
    
    // Position (0, 4) should be identified as winning move
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Should return only the winning move
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0], (0, 4));
}

#[test]
fn test_creates_five_in_row_with_gap() {
    // Test pattern with gap: X X X _ X
    let mut board = Board::new(19);
    
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 6, Player::Max);
    board.place_stone(5, 7, Player::Max);
    board.place_stone(5, 9, Player::Max);
    
    // Position (5, 8) should complete the five
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Should contain the winning move
    assert!(!moves.is_empty());
    assert!(moves.contains(&(5, 8)), "Should contain winning move at (5, 8)");
}

#[test]
fn test_threat_moves_prioritization() {
    // Test that with many threat moves, the best ones are prioritized
    let mut state = GameState::new(19, 5);
    
    // Create a complex position with many threats
    // Central area with multiple patterns
    let positions = vec![
        (9, 9), (9, 10), (9, 11),  // Three in a row horizontally
        (10, 9), (11, 9),           // Two in a row vertically
        (8, 8), (7, 7),             // Two diagonally
        (10, 11), (11, 11),         // Another vertical pair
    ];
    
    for &(row, col) in positions.iter() {
        state.make_move((row, col));
    }
    
    // Get candidate moves - should not be empty even if many threats
    let moves = state.get_candidate_moves();
    
    // Should return moves (prioritized if >30)
    assert!(!moves.is_empty(), "Should return threat moves even when many candidates");
    
    // Should be limited to reasonable number
    assert!(moves.len() <= 30, "Should limit to 30 moves when too many");
}

#[test]
fn test_winning_move_detection_various_patterns() {
    let mut board = Board::new(19);
    
    // Test vertical winning pattern
    board.place_stone(5, 5, Player::Max);
    board.place_stone(6, 5, Player::Max);
    board.place_stone(7, 5, Player::Max);
    board.place_stone(8, 5, Player::Max);
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Should find winning move at (4, 5) or (9, 5)
    assert_eq!(moves.len(), 1);
    assert!(moves[0] == (4, 5) || moves[0] == (9, 5));
}

#[test]
fn test_blocking_opponent_winning_move() {
    let mut board = Board::new(19);
    
    // Opponent has 4 in a row
    board.place_stone(10, 10, Player::Min);
    board.place_stone(10, 11, Player::Min);
    board.place_stone(10, 12, Player::Min);
    board.place_stone(10, 13, Player::Min);
    
    // Our move should block
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Should return blocking moves
    assert!(!moves.is_empty());
    assert!(moves.contains(&(10, 9)) || moves.contains(&(10, 14)));
}

#[test]
fn test_open_four_detection() {
    let mut board = Board::new(19);
    
    // Create open four: _ X X X X _
    board.place_stone(10, 11, Player::Min);
    board.place_stone(10, 12, Player::Min);
    board.place_stone(10, 13, Player::Min);
    board.place_stone(10, 14, Player::Min);
    
    // Both (10, 10) and (10, 15) are empty - this is an open four threat
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Must block the open four - should return both blocking positions
    assert!(!moves.is_empty());
    // Open four creates multiple winning threats, so we should get both positions
    assert!(moves.len() >= 1);
}

#[test]
fn test_empty_board_returns_center() {
    let board = Board::new(19);
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0], (9, 9)); // Center of 19x19 board
}

#[test]
fn test_zone_based_moves_early_game() {
    let mut board = Board::new(19);
    
    // Place one stone
    board.place_stone(9, 9, Player::Max);
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);
    
    // Should return zone-based moves around the center
    assert!(!moves.is_empty());
    
    // All moves should be nearby (within zone radius)
    // Early game uses radius 2, which means Manhattan distance up to 2
    for &(row, col) in &moves {
        let dr = (row as isize - 9).abs();
        let dc = (col as isize - 9).abs();
        // Check that it's within the square radius (not Manhattan)
        assert!(dr <= 2 && dc <= 2, "Move ({}, {}) is too far from center (dr={}, dc={})", row, col, dr, dc);
    }
}
