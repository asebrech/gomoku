use gomoku::core::board::{Board, Player};
use gomoku::core::captures::CaptureHandler;

#[test]
fn test_horizontal_capture() {
    let mut board = Board::new(19);

    // Set up: X - O - O - X (horizontal capture)
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 6, Player::Min);
    board.place_stone(5, 7, Player::Min);
    board.place_stone(5, 8, Player::Max);

    // Test capture detection
    let captures = CaptureHandler::detect_captures(&board, 5, 8, Player::Max);
    assert_eq!(captures.len(), 2);
    assert!(captures.contains(&(5, 6)));
    assert!(captures.contains(&(5, 7)));
}

#[test]
fn test_vertical_capture() {
    let mut board = Board::new(19);

    // Set up: X - O - O - X (vertical capture)
    board.place_stone(5, 5, Player::Max);
    board.place_stone(6, 5, Player::Min);
    board.place_stone(7, 5, Player::Min);
    board.place_stone(8, 5, Player::Max);

    // Test capture detection
    let captures = CaptureHandler::detect_captures(&board, 8, 5, Player::Max);
    assert_eq!(captures.len(), 2);
    assert!(captures.contains(&(6, 5)));
    assert!(captures.contains(&(7, 5)));
}

#[test]
fn test_diagonal_capture() {
    let mut board = Board::new(19);

    // Set up: X - O - O - X (diagonal capture)
    board.place_stone(5, 5, Player::Max);
    board.place_stone(6, 6, Player::Min);
    board.place_stone(7, 7, Player::Min);
    board.place_stone(8, 8, Player::Max);

    // Test capture detection
    let captures = CaptureHandler::detect_captures(&board, 8, 8, Player::Max);
    assert_eq!(captures.len(), 2);
    assert!(captures.contains(&(6, 6)));
    assert!(captures.contains(&(7, 7)));
}

#[test]
fn test_anti_diagonal_capture() {
    let mut board = Board::new(19);

    // Set up: X - O - O - X (anti-diagonal capture)
    board.place_stone(8, 5, Player::Max);
    board.place_stone(7, 6, Player::Min);
    board.place_stone(6, 7, Player::Min);
    board.place_stone(5, 8, Player::Max);

    // Test capture detection
    let captures = CaptureHandler::detect_captures(&board, 5, 8, Player::Max);
    assert_eq!(captures.len(), 2);
    assert!(captures.contains(&(7, 6)));
    assert!(captures.contains(&(6, 7)));
}

#[test]
fn test_multiple_captures() {
    let mut board = Board::new(19);

    // Set up multiple captures in different directions
    // Horizontal: X - O - O - X
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 6, Player::Min);
    board.place_stone(5, 7, Player::Min);

    // Vertical: X - O - O - X
    board.place_stone(6, 8, Player::Min);
    board.place_stone(7, 8, Player::Min);
    board.place_stone(8, 8, Player::Max);

    // Place the final stone that creates both captures
    board.place_stone(5, 8, Player::Max);
    let captures = CaptureHandler::detect_captures(&board, 5, 8, Player::Max);

    // Should detect both horizontal and vertical captures
    assert_eq!(captures.len(), 4);
    assert!(captures.contains(&(5, 6)));
    assert!(captures.contains(&(5, 7)));
    assert!(captures.contains(&(6, 8)));
    assert!(captures.contains(&(7, 8)));
}

#[test]
fn test_no_capture_empty_space() {
    let mut board = Board::new(19);

    // Set up: X - O - empty - X (no capture)
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 6, Player::Min);
    // Empty space at (5, 7)
    board.place_stone(5, 8, Player::Max);

    let captures = CaptureHandler::detect_captures(&board, 5, 8, Player::Max);
    assert_eq!(captures.len(), 0);
}

#[test]
fn test_no_capture_same_player() {
    let mut board = Board::new(19);

    // Set up: X - X - X - X (no capture)
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 6, Player::Max);
    board.place_stone(5, 7, Player::Max);
    board.place_stone(5, 8, Player::Max);

    let captures = CaptureHandler::detect_captures(&board, 5, 8, Player::Max);
    assert_eq!(captures.len(), 0);
}

#[test]
fn test_execute_captures() {
    let mut board = Board::new(19);

    // Set up a capture scenario
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 6, Player::Min);
    board.place_stone(5, 7, Player::Min);
    board.place_stone(5, 8, Player::Max);

    let captures = CaptureHandler::detect_captures(&board, 5, 8, Player::Max);
    CaptureHandler::execute_captures(&mut board, &captures);

    // Check that captured stones are removed
    assert_eq!(board.get_player(5, 6), None);
    assert_eq!(board.get_player(5, 7), None);

    // Check that other stones remain
    assert_eq!(board.get_player(5, 5), Some(Player::Max));
    assert_eq!(board.get_player(5, 8), Some(Player::Max));
}

#[test]
fn test_edge_case_captures() {
    let mut board = Board::new(19);

    // Test capture at board edge
    board.place_stone(0, 0, Player::Max);
    board.place_stone(0, 1, Player::Min);
    board.place_stone(0, 2, Player::Min);
    board.place_stone(0, 3, Player::Max);

    let captures = CaptureHandler::detect_captures(&board, 0, 3, Player::Max);
    assert_eq!(captures.len(), 2);
    assert!(captures.contains(&(0, 1)));
    assert!(captures.contains(&(0, 2)));
}

// New tests for bitboard-specific scenarios

#[test]
fn test_capture_across_bitboard_chunks() {
    let mut board = Board::new(19);

    // Test capture that spans across different bitboard chunks
    // Position (6, 18) is in chunk 1, position (7, 0) is in chunk 1
    // This tests the bitboard chunk boundary handling
    
    // Set up vertical capture spanning chunks
    board.place_stone(6, 18, Player::Max);  // chunk 1
    board.place_stone(7, 18, Player::Min);  // chunk 1
    board.place_stone(8, 18, Player::Min);  // chunk 1
    board.place_stone(9, 18, Player::Max);  // chunk 1

    let captures = CaptureHandler::detect_captures(&board, 9, 18, Player::Max);
    assert_eq!(captures.len(), 2);
    assert!(captures.contains(&(7, 18)));
    assert!(captures.contains(&(8, 18)));
}

#[test]
fn test_capture_in_high_chunk() {
    let mut board = Board::new(19);

    // Test capture in the highest chunk (chunk 2)
    // Position (18, 18) is at index 360, which is in chunk 2
    board.place_stone(18, 15, Player::Max);
    board.place_stone(18, 16, Player::Min);
    board.place_stone(18, 17, Player::Min);
    board.place_stone(18, 18, Player::Max);

    let captures = CaptureHandler::detect_captures(&board, 18, 18, Player::Max);
    assert_eq!(captures.len(), 2);
    assert!(captures.contains(&(18, 16)));
    assert!(captures.contains(&(18, 17)));
}

#[test]
fn test_capture_execute_bitboard_consistency() {
    let mut board = Board::new(19);

    // Set up multiple captures across different chunks
    board.place_stone(5, 5, Player::Max);    // chunk 0
    board.place_stone(5, 6, Player::Min);    // chunk 0
    board.place_stone(5, 7, Player::Min);    // chunk 0
    board.place_stone(5, 8, Player::Max);    // chunk 0
    
    board.place_stone(15, 15, Player::Max);  // chunk 2
    board.place_stone(16, 15, Player::Min);  // chunk 2
    board.place_stone(17, 15, Player::Min);  // chunk 2
    board.place_stone(18, 15, Player::Max);  // chunk 2

    // Execute captures from first position
    let captures1 = CaptureHandler::detect_captures(&board, 5, 8, Player::Max);
    CaptureHandler::execute_captures(&mut board, &captures1);
    
    // Execute captures from second position
    let captures2 = CaptureHandler::detect_captures(&board, 18, 15, Player::Max);
    CaptureHandler::execute_captures(&mut board, &captures2);

    // Verify all captured stones are removed
    assert_eq!(board.get_player(5, 6), None);
    assert_eq!(board.get_player(5, 7), None);
    assert_eq!(board.get_player(16, 15), None);
    assert_eq!(board.get_player(17, 15), None);
    
    // Verify capturing stones remain
    assert_eq!(board.get_player(5, 5), Some(Player::Max));
    assert_eq!(board.get_player(5, 8), Some(Player::Max));
    assert_eq!(board.get_player(15, 15), Some(Player::Max));
    assert_eq!(board.get_player(18, 15), Some(Player::Max));
}

#[test]
fn test_capture_with_bitboard_bounds() {
    let mut board = Board::new(19);

    // Test capture at the exact boundary of what fits in our bitboards
    // This ensures our bounds checking works correctly
    board.place_stone(18, 16, Player::Max);
    board.place_stone(18, 17, Player::Min);
    board.place_stone(18, 18, Player::Min);
    
    // This would be out of bounds, so no capture should occur
    let captures = CaptureHandler::detect_captures(&board, 18, 16, Player::Max);
    assert_eq!(captures.len(), 0); // No capture because there's no stone at (18, 19)
}

#[test]
fn test_capture_empty_board_state() {
    let mut board = Board::new(19);

    // Test that empty board doesn't produce false captures
    let captures = CaptureHandler::detect_captures(&board, 9, 9, Player::Max);
    assert_eq!(captures.len(), 0);
}

#[test]
fn test_capture_single_stone() {
    let mut board = Board::new(19);

    // Test with only one stone on board
    board.place_stone(9, 9, Player::Max);
    let captures = CaptureHandler::detect_captures(&board, 9, 9, Player::Max);
    assert_eq!(captures.len(), 0);
}
