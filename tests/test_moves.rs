use gomoku::core::board::{Board, Player};
use gomoku::core::moves::MoveHandler;

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
        (8, 8),
        (8, 9),
        (8, 10),
        (9, 8),
        (9, 10),
        (10, 8),
        (10, 9),
        (10, 10),
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
fn test_edge_case_moves() {
    let mut board = Board::new(19);

    // Place stone at edge
    board.place_stone(0, 0, Player::Max);

    let moves = MoveHandler::get_possible_moves(&board, Player::Min);

    // Should include only valid adjacent positions within board bounds
    let expected_moves = vec![(0, 1), (1, 0), (1, 1)];

    for &pos in &expected_moves {
        assert!(
            moves.contains(&pos),
            "Missing edge adjacent position: {:?}",
            pos
        );
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
        (0, 1),
        (1, 0),
        (1, 1),
        // Top-right corner adjacents
        (0, 17),
        (1, 17),
        (1, 18),
        // Bottom-left corner adjacents
        (17, 0),
        (17, 1),
        (18, 1),
        // Bottom-right corner adjacents
        (17, 17),
        (17, 18),
        (18, 17),
    ];

    for &pos in &expected_corners {
        assert!(
            moves.contains(&pos),
            "Missing corner adjacent position: {:?}",
            pos
        );
    }
}

#[test] 
fn test_double_three_moves_excluded() {
    use gomoku::core::moves::RuleValidator;
    
    let mut board = Board::new(19);
    
    // Create a scenario where placing at a specific position would create double-three
    // Pattern that creates double-three when filled:
    // . . X . X . .
    // . X . ? . X .  (? = target position that would create double-three)
    // . . X . X . .
    
    board.place_stone(5, 7, Player::Max);   // Top
    board.place_stone(5, 9, Player::Max);   // Top  
    board.place_stone(6, 6, Player::Max);   // Left
    board.place_stone(6, 10, Player::Max);  // Right
    board.place_stone(7, 7, Player::Max);   // Bottom
    board.place_stone(7, 9, Player::Max);   // Bottom
    
    // Verify that position (6,8) would create double-three
    assert!(RuleValidator::creates_double_three(&board, 6, 8, Player::Max));
    
    // Now test that MoveHandler excludes this move
    let moves = MoveHandler::get_possible_moves(&board, Player::Max);
    
    // The double-three creating move should NOT be in possible moves
    assert!(!moves.contains(&(6, 8)), "Double-three move should be excluded from possible moves");
    
    // But adjacent valid moves should still be included
    let valid_adjacent_moves = [
        (5, 8), (6, 7), (6, 9), (7, 8)  // Adjacent to existing stones
    ];
    
    for &mv in &valid_adjacent_moves {
        if !RuleValidator::creates_double_three(&board, mv.0, mv.1, Player::Max) {
            assert!(moves.contains(&mv), "Valid adjacent move {:?} should be included", mv);
        }
    }
}
