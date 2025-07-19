use gomoku::core::board::{Board, Player};
use gomoku::core::state::GameState;

#[test]
fn test_first_move_center_only() {
    let mut board = Board::new(19);
    let moves = board.get_possible_moves_vec(Player::Max);

    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0], (9, 9)); // Center of 19x19 board
}

#[test]
fn test_first_move_different_board_sizes() {
    let mut board15 = Board::new(15);
    let moves15 = board15.get_possible_moves_vec(Player::Max);
    assert_eq!(moves15.len(), 1);
    assert_eq!(moves15[0], (7, 7)); // Center of 15x15 board

    let mut board13 = Board::new(13);
    let moves13 = board13.get_possible_moves_vec(Player::Max);
    assert_eq!(moves13.len(), 1);
    assert_eq!(moves13[0], (6, 6)); // Center of 13x13 board
}

#[test]
fn test_adjacent_moves_only() {
    let mut board = Board::new(19);
    board.place_stone(9, 9, Player::Max);

    let moves = board.get_possible_moves_vec(Player::Min);

    // Should only include adjacent positions
    assert!(moves.len() > 0);
    for &(row, col) in moves.iter() {
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

    let moves = board.get_possible_moves_vec(Player::Max);

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

    let moves = board.get_possible_moves_vec(Player::Min);

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

    let moves = board.get_possible_moves_vec(Player::Min);

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
fn test_comprehensive_move_cache_validation() {
    let mut board = Board::new(19);
    
    // Test 1: Initial empty board - should only have center move
    let initial_moves = board.get_possible_moves_vec(Player::Max);
    assert_eq!(initial_moves.len(), 1, "Empty board should have exactly 1 move");
    assert_eq!(initial_moves[0], (9, 9), "First move should be center");
    
    // Test 2: Place first stone and verify adjacent moves
    board.place_stone(9, 9, Player::Max);
    let after_first = board.get_possible_moves_vec(Player::Min);
    assert_eq!(after_first.len(), 8, "After first move should have 8 adjacent moves");
    
    // Verify all 8 adjacent positions are present
    let expected_adjacent = vec![
        (8, 8), (8, 9), (8, 10),
        (9, 8),         (9, 10),
        (10, 8), (10, 9), (10, 10),
    ];
    for &pos in &expected_adjacent {
        assert!(after_first.contains(&pos), "Missing adjacent position: {:?}", pos);
    }
    
    // Test 3: Place second stone and verify move expansion
    board.place_stone(8, 8, Player::Min);
    let after_second = board.get_possible_moves_vec(Player::Max);
    
    // Should have original 7 moves (8 - 1 occupied) plus new adjacent moves around (8,8)
    // New adjacents around (8,8): (7,7), (7,8), (7,9), (8,7), (9,7)
    // But (8,9), (9,8), (9,9) are already in the set
    let expected_new_adjacents = vec![(7, 7), (7, 8), (7, 9), (8, 7), (9, 7)];
    for &pos in &expected_new_adjacents {
        assert!(after_second.contains(&pos), "Missing new adjacent position: {:?}", pos);
    }
    
    // Verify occupied positions are not in moves
    assert!(!after_second.contains(&(9, 9)), "Occupied position should not be in moves");
    assert!(!after_second.contains(&(8, 8)), "Occupied position should not be in moves");
    
    // Test 4: Create capture scenario using GameState (captures only work with GameState)
    let mut capture_state = GameState::new(19, 5);
    
    // Set up horizontal capture: Max-Min-Min-Max
    capture_state.board.place_stone(10, 10, Player::Max);  // Setup for capture
    capture_state.board.place_stone(10, 11, Player::Min);  // First capture target
    capture_state.board.place_stone(10, 12, Player::Min);  // Second capture target
    
    // Execute capture by placing at (10, 13) using make_move which handles captures
    capture_state.make_move((10, 13));  // This should capture (10,11) and (10,12)
    
    let after_capture = capture_state.board.get_possible_moves_vec(Player::Min);
    
    // Verify captured positions are back in possible moves
    assert!(after_capture.contains(&(10, 11)), "Captured position should be available again");
    assert!(after_capture.contains(&(10, 12)), "Captured position should be available again");
    
    // Verify capture occurred
    assert_eq!(capture_state.max_captures, 1, "Should have captured one pair");
    // Capture functionality moved to GameState
    
    // Test 5: Complex scenario with multiple placements
    let positions_to_place = vec![
        (7, 10, Player::Min),
        (8, 11, Player::Max),
        (6, 9, Player::Min),
        (11, 8, Player::Max),
        (12, 7, Player::Min),
    ];
    
    for (row, col, player) in positions_to_place {
        board.place_stone(row, col, player);
        let after_moves = board.get_possible_moves_vec(player.opponent());
        
        // Verify placed position is removed from possible moves
        assert!(!after_moves.contains(&(row, col)), 
                "Placed position ({}, {}) should not be in moves", row, col);
        
        // Verify all moves are adjacent to stones
        for &(r, c) in after_moves.iter() {
            assert!(board.is_empty_position(r, c), 
                   "Position ({}, {}) should be empty", r, c);
            assert!(board.is_adjacent_to_stone(r, c), 
                   "Position ({}, {}) should be adjacent to stones", r, c);
        }
    }
    
    // Test 6: Edge case - fill around center and verify move pruning
    let center_adjacent = vec![
        (8, 9, Player::Max), (8, 10, Player::Min), (9, 10, Player::Max),
        (10, 9, Player::Min), (10, 8, Player::Max), (8, 7, Player::Min),
    ];
    
    for (row, col, player) in center_adjacent {
        if board.is_empty_position(row, col) {
            board.place_stone(row, col, player);
        }
    }
    
    let dense_moves = board.get_possible_moves_vec(Player::Max);
    
    // Verify no isolated moves (not adjacent to stones)
    for &(r, c) in dense_moves.iter() {
        assert!(board.is_adjacent_to_stone(r, c), 
               "All moves should be adjacent to stones in dense area");
    }
    
    // Test 7: Double-three rule enforcement in moves
    let mut test_board = Board::new(19);
    
    // Create potential double-three scenario
    test_board.place_stone(5, 5, Player::Max);
    test_board.place_stone(5, 7, Player::Max);   // Horizontal potential
    test_board.place_stone(7, 6, Player::Max);   // Vertical potential
    test_board.place_stone(3, 6, Player::Max);   // Vertical potential
    
    let double_three_moves = test_board.get_possible_moves_vec(Player::Max);
    
    // Check if position (5, 6) creates double-three and is excluded
    if test_board.creates_double_three(5, 6, Player::Max) {
        assert!(!double_three_moves.contains(&(5, 6)), 
               "Double-three position should be excluded from moves");
    }
    
    // Test 8: Performance validation - ensure moves are cached
    let mut perf_board = Board::new(19);
    
    // Place several stones to create a complex board state
    let perf_positions = vec![
        (9, 9, Player::Max), (8, 8, Player::Min), (10, 10, Player::Max),
        (7, 7, Player::Min), (11, 11, Player::Max), (6, 6, Player::Min),
        (12, 12, Player::Max), (5, 5, Player::Min), (13, 13, Player::Max),
    ];
    
    for (row, col, player) in perf_positions {
        perf_board.place_stone(row, col, player);
    }
    
    // Get moves multiple times - should use cache
    let moves1 = perf_board.get_possible_moves_vec(Player::Min);
    let moves2 = perf_board.get_possible_moves_vec(Player::Min);
    let moves3 = perf_board.get_possible_moves_vec(Player::Min);
    
    // Should be identical (cached)
    assert_eq!(moves1.len(), moves2.len(), "Cached moves should be identical");
    assert_eq!(moves2.len(), moves3.len(), "Cached moves should be identical");
    
    for &pos in &moves1 {
        assert!(moves2.contains(&pos), "Cached moves should contain same positions");
        assert!(moves3.contains(&pos), "Cached moves should contain same positions");
    }
    
    // Test 9: Boundary conditions
    let mut boundary_board = Board::new(19);
    
    // Test corners
    boundary_board.place_stone(0, 0, Player::Max);
    let corner_moves = boundary_board.get_possible_moves_vec(Player::Min);
    
    // Should only have valid adjacent positions (no out-of-bounds)
    for &(r, c) in corner_moves.iter() {
        assert!(r < 19 && c < 19, "All moves should be within board bounds");
        assert!(boundary_board.is_empty_position(r, c), "All moves should be on empty positions");
    }
    
    // Test edges
    boundary_board.place_stone(0, 9, Player::Min);  // Top edge
    boundary_board.place_stone(18, 9, Player::Max); // Bottom edge
    boundary_board.place_stone(9, 0, Player::Min);  // Left edge
    boundary_board.place_stone(9, 18, Player::Max); // Right edge
    
    let edge_moves = boundary_board.get_possible_moves_vec(Player::Min);
    
    // Verify all edge moves are valid
    for &(r, c) in edge_moves.iter() {
        assert!(r < 19 && c < 19, "Edge moves should be within bounds");
        assert!(boundary_board.is_empty_position(r, c), "Edge moves should be on empty positions");
        assert!(boundary_board.is_adjacent_to_stone(r, c), "Edge moves should be adjacent to stones");
    }
    
    println!("Comprehensive move cache validation completed successfully!");
    println!("Final board state has {} possible moves", edge_moves.len());
}
