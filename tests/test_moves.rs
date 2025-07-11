use gomoku::core::board::{Board, Player};
use gomoku::core::moves::{MoveHandler, RuleValidator};

#[test]
fn test_first_move_center_only() {
    let board = Board::new(19);
    let moves = MoveHandler::get_possible_moves(&board, Player::Max);
    
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0], (9, 9)); // Center of 19x19 board
}

#[test]
fn test_first_move_different_board_sizes() {
    let board15 = Board::new(15);
    let moves15 = MoveHandler::get_possible_moves(&board15, Player::Max);
    assert_eq!(moves15.len(), 1);
    assert_eq!(moves15[0], (7, 7)); // Center of 15x15 board
    
    let board13 = Board::new(13);
    let moves13 = MoveHandler::get_possible_moves(&board13, Player::Max);
    assert_eq!(moves13.len(), 1);
    assert_eq!(moves13[0], (6, 6)); // Center of 13x13 board
}

#[test]
fn test_adjacent_moves_only() {
    let mut board = Board::new(19);
    board.place_stone(9, 9, Player::Max);
    
    let moves = MoveHandler::get_possible_moves(&board, Player::Min);
    
    // Should only include adjacent positions
    assert!(moves.len() > 0);
    for &(row, col) in &moves {
        assert!(board.is_adjacent_to_stone(row, col));
        assert!(board.is_empty_position(row, col));
    }
    
    // Should include all 8 adjacent positions
    let expected_adjacent = vec![
        (8, 8), (8, 9), (8, 10),
        (9, 8),         (9, 10),
        (10, 8), (10, 9), (10, 10),
    ];
    
    for &pos in &expected_adjacent {
        assert!(moves.contains(&pos), "Missing adjacent position: {:?}", pos);
    }
}

#[test]
fn test_no_occupied_moves() {
    let mut board = Board::new(19);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(8, 8, Player::Min);
    
    let moves = MoveHandler::get_possible_moves(&board, Player::Max);
    
    // Should not include occupied positions
    assert!(!moves.contains(&(9, 9)));
    assert!(!moves.contains(&(8, 8)));
    
    // Should still include other adjacent positions
    assert!(moves.contains(&(8, 9)));
    assert!(moves.contains(&(9, 8)));
}

#[test]
fn test_double_three_detection() {
    let mut board = Board::new(19);
    
    // Set up a scenario where placing at (9,9) would create double three
    // Pattern 1: horizontal three
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);
    // (9, 9) would complete horizontal three
    board.place_stone(9, 10, Player::Max);
    
    // Pattern 2: vertical three  
    board.place_stone(7, 9, Player::Max);
    board.place_stone(8, 9, Player::Max);
    // (9, 9) would complete vertical three
    board.place_stone(10, 9, Player::Max);
    
    // This should detect double three
    assert!(RuleValidator::creates_double_three(&board, 9, 9, Player::Max));
}

#[test]
fn test_no_double_three_single_pattern() {
    let mut board = Board::new(19);
    
    // Set up only one three pattern
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);
    
    // This should not detect double three (only one pattern)
    assert!(!RuleValidator::creates_double_three(&board, 9, 9, Player::Max));
}

#[test]
fn test_free_three_detection() {
    let mut board = Board::new(19);
    
    // Set up a free three pattern: _ X X X _
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    
    // Check if placing at (9, 7) or (9, 11) would create a free three
    assert!(RuleValidator::is_free_three(&board, 9, 7, Player::Max, (0, 1)));
    assert!(RuleValidator::is_free_three(&board, 9, 11, Player::Max, (0, 1)));
}

#[test]
fn test_blocked_three_not_free() {
    let mut board = Board::new(19);
    
    // Set up a blocked three pattern: O X X X _
    board.place_stone(9, 7, Player::Min); // Blocking stone
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    
    // This should not be detected as a free three
    assert!(!RuleValidator::is_free_three(&board, 9, 11, Player::Max, (0, 1)));
}

#[test]
fn test_moves_exclude_double_three() {
    let mut board = Board::new(19);
    
    // Set up potential double three scenario
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(7, 9, Player::Max);
    board.place_stone(8, 9, Player::Max);
    board.place_stone(10, 9, Player::Max);
    
    let moves = MoveHandler::get_possible_moves(&board, Player::Max);
    
    // Should not include the double three move
    assert!(!moves.contains(&(9, 9)));
}

#[test]
fn test_edge_case_moves() {
    let mut board = Board::new(19);
    
    // Place stone at edge
    board.place_stone(0, 0, Player::Max);
    
    let moves = MoveHandler::get_possible_moves(&board, Player::Min);
    
    // Should include only valid adjacent positions within board bounds
    let expected_moves = vec![(0, 1), (1, 0), (1, 1)];
    
    for &pos in &expected_moves {
        assert!(moves.contains(&pos), "Missing edge adjacent position: {:?}", pos);
    }
    
    // Should not include out-of-bounds positions
    assert!(!moves.contains(&(0, 19)));
    assert!(!moves.contains(&(19, 0)));
}

#[test]
fn test_corner_moves() {
    let mut board = Board::new(19);
    
    // Place stones at all corners
    board.place_stone(0, 0, Player::Max);
    board.place_stone(0, 18, Player::Max);
    board.place_stone(18, 0, Player::Max);
    board.place_stone(18, 18, Player::Max);
    
    let moves = MoveHandler::get_possible_moves(&board, Player::Min);
    
    // Should include adjacent positions for all corners
    let expected_corners = vec![
        // Top-left corner adjacents
        (0, 1), (1, 0), (1, 1),
        // Top-right corner adjacents  
        (0, 17), (1, 17), (1, 18),
        // Bottom-left corner adjacents
        (17, 0), (17, 1), (18, 1),
        // Bottom-right corner adjacents
        (17, 17), (17, 18), (18, 17),
    ];
    
    for &pos in &expected_corners {
        assert!(moves.contains(&pos), "Missing corner adjacent position: {:?}", pos);
    }
}

#[test]
fn test_complex_double_three_scenario() {
    let mut board = Board::new(19);
    
    // Create a more complex board state
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(10, 9, Player::Max);
    board.place_stone(10, 10, Player::Max);
    
    // Test various positions for double three
    let moves = MoveHandler::get_possible_moves(&board, Player::Max);
    
    // Should include valid moves that don't create double three
    assert!(moves.len() > 0);
    
    for &(row, col) in &moves {
        assert!(!RuleValidator::creates_double_three(&board, row, col, Player::Max));
    }
}

