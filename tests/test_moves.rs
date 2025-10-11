use gomoku::core::board::{Board, Player};
use gomoku::ai::move_generation::MoveGenerator;

#[test]
fn test_first_move_center_only() {
    let board = Board::new(19);
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);

    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0], (9, 9)); // Center of 19x19 board
}

#[test]
fn test_first_move_different_board_sizes() {
    let board15 = Board::new(15);
    let moves15 = MoveGenerator::get_candidate_moves(&board15, Player::Max);
    assert_eq!(moves15.len(), 1);
    assert_eq!(moves15[0], (7, 7)); // Center of 15x15 board

    let board13 = Board::new(13);
    let moves13 = MoveGenerator::get_candidate_moves(&board13, Player::Max);
    assert_eq!(moves13.len(), 1);
    assert_eq!(moves13[0], (6, 6)); // Center of 13x13 board
}

#[test]
fn test_adjacent_moves_only() {
    let mut board = Board::new(19);
    board.place_stone(9, 9, Player::Max);

    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);

    // Should include moves within zone (radius 2 in early game, < 10 stones)
    assert!(moves.len() > 0);
    for &(row, col) in &moves {
        // Check that move is within zone radius of 2 from (9,9)
        let distance = ((row as isize - 9).abs().max((col as isize - 9).abs())) as usize;
        assert!(distance <= 2, "Move ({}, {}) is too far from (9, 9)", row, col);
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

    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);

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

    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);

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

    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);

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
    use gomoku::core::rules::GameRules;
    
    let mut board = Board::new(19);
    
    // Create a double-three scenario:
    // A double-three is when a move creates TWO open-ended threes (free threes)
    // 
    // Setup pattern (X = Max, O = Min, ? = target position (8,8)):
    //     6 7 8 9 10
    //  6  . . . . .
    //  7  . . X . .   (vertical pattern: 7,8 - 8,8 - 9,8)
    //  8  . . ? . .   <- Target at (8,8)
    //  9  . X X X .   (horizontal pattern: 9,7 - 9,8 - 9,9 where diagonal contributes)
    // 10  . . O . .   (blocks one direction)
    //
    // Actually, let's use the classic double-three pattern:
    //     7 8 9 10 11
    //  7  . X . . .   (diagonal: 7,8 - 8,9 - 9,10)
    //  8  . . X . .   
    //  9  . . . X .   <- if we play at (8,9), creates:
    // 10  . . . . .       1. Diagonal three: 7,8-8,9-9,10
    //                     2. Horizontal three needs more stones...
    //
    // Let's use a simpler, cleaner pattern:
    // Place stones so that playing at (9,9) creates two free threes:
    //     7 8 9 10 11
    //  7  . X . X .   (diagonal pattern through (8,8), (9,9), (10,10))
    //  8  . . X . .   
    //  9  X . ? . X   (horizontal pattern through (9,7), (9,9), (9,11))
    // 10  . . . X .   
    // 11  . . . . .
    
    // Horizontal setup: X . ? . X at row 9
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 11, Player::Max);
    
    // Diagonal setup: X . ? . X from (7,7) to (11,11)
    board.place_stone(7, 7, Player::Max);
    board.place_stone(11, 11, Player::Max);
    
    // Add one more stone to make both patterns into threes when (9,9) is played
    // For horizontal: need X at (9,8) to make X X ? . X
    board.place_stone(9, 8, Player::Max);  // Now horizontal is X X ? . X
    
    // For diagonal: need X at (8,8) to make X X ? . X  
    board.place_stone(8, 8, Player::Max);  // Now diagonal is X X ? . X
    
    // Both patterns (horizontal and diagonal) will become "three in a row" when we place at (9,9)
    // And both have space to extend to 4, making them "open threes"
    
    // Verify that position (9,9) would create double-three
    assert!(GameRules::creates_double_three(&board, 9, 9, Player::Max),
            "Position (9,9) should create double-three");
    
    // Now test that MoveGenerator excludes this move
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // The double-three creating move should NOT be in candidate moves
    assert!(!moves.contains(&(9, 9)), "Double-three move (9,9) should be excluded from candidate moves");
    
    // Verify that there ARE some valid moves available (not all moves create double-three)
    assert!(!moves.is_empty(), "Should have at least some valid moves available");
    
    // Verify that ALL returned moves are valid (don't create double-three)
    for &mv in &moves {
        assert!(!GameRules::creates_double_three(&board, mv.0, mv.1, Player::Max),
                "Move ({}, {}) should not create double-three but was included", mv.0, mv.1);
    }
    
    // Check that at least one move is reasonably close to the stones
    let has_nearby_move = moves.iter().any(|&(r, c)| {
        // Within radius 2 of the existing stones
        (r as isize - 9).abs() <= 2 && (c as isize - 9).abs() <= 2
    });
    assert!(has_nearby_move, "Should have at least one move near the stone cluster");
}
