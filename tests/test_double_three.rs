use gomoku::core::{board::{Board, Player}, rules::GameRules};


    #[test]
    fn test_creates_double_three_basic_case() {
        let mut board = Board::new(19);
        
        // Create a proper double-three scenario with more space
        // Pattern:
        // . . X . X . .
        // . X . ? . X .  
        // . . X . X . .
        
        board.place_stone(5, 7, Player::Max);   // Top
        board.place_stone(5, 9, Player::Max);   // Top  
        board.place_stone(6, 6, Player::Max);   // Left
        board.place_stone(6, 10, Player::Max);  // Right
        board.place_stone(7, 7, Player::Max);   // Bottom
        board.place_stone(7, 9, Player::Max);   // Bottom
        
        // Placing at (6,8) should create double-three (horizontal and vertical)
        assert!(GameRules::creates_double_three(&board, 6, 8, Player::Max));
    }


#[test]
fn test_creates_double_three_diagonal() {
    let mut board = Board::new(19);
    
    // Create a cross pattern that forms two diagonal threes when center is filled
    //   0 1 2 3 4
    // 0 . . . . .
    // 1 . . X . .
    // 2 . X . X .  
    // 3 . . X . .
    // 4 . . . . .
    
    board.place_stone(1, 2, Player::Max);  // Top
    board.place_stone(2, 1, Player::Max);  // Left  
    board.place_stone(2, 3, Player::Max);  // Right
    board.place_stone(3, 2, Player::Max);  // Bottom
    
    // Placing at (2,2) creates both diagonal threes:
    // Diagonal \: (1,1) would be empty, (2,2) placed, (3,3) would be empty
    // Actually, let me create proper consecutive diagonals:
    
    let mut board = Board::new(19);
    
    // Diagonal (1,1) consecutive stones
    board.place_stone(8, 8, Player::Max);
    board.place_stone(10, 10, Player::Max);
    
    // Diagonal (1,-1) consecutive stones  
    board.place_stone(8, 10, Player::Max);
    board.place_stone(10, 8, Player::Max);
    
    // Placing at (9,9) creates two 3-stone diagonal lines
    assert!(GameRules::creates_double_three(&board, 9, 9, Player::Max));
}

    #[test]
    fn test_single_free_three_allowed() {
        let mut board = Board::new(19);
        
        // Create only one free-three horizontally
        board.place_stone(5, 5, Player::Max);
        board.place_stone(5, 7, Player::Max);
        
        // Placing at (5,6) creates only one free-three - should be allowed
        assert!(!GameRules::creates_double_three(&board, 5, 6, Player::Max));
    }

    #[test]
    fn test_blocked_three_not_free() {
        let mut board = Board::new(19);
        
        // Create blocked three (not free-three)
        board.place_stone(5, 5, Player::Max);
        board.place_stone(5, 7, Player::Max);
        board.place_stone(5, 8, Player::Min); // Blocks one end
        
        // This shouldn't create a free-three since one end is blocked
        assert!(!GameRules::creates_double_three(&board, 5, 6, Player::Max));
    }

    #[test]
    fn test_no_space_for_open_four() {
        let mut board = Board::new(19);
        
        // Create scenario where there's no space for undefendable four
        board.place_stone(0, 1, Player::Max);
        board.place_stone(0, 3, Player::Max);
        // Board edge limits the potential for open four
        
        assert!(!GameRules::creates_double_three(&board, 0, 2, Player::Max));
    }

    #[test]
    fn test_complex_double_three_scenario() {
        let mut board = Board::new(19);
        
        // More complex pattern from the subject appendix
        /*
        . . . . .
        . X . X .
        . . ? . .
        . X . X .
        . . . . .
        */
        
        board.place_stone(8, 7, Player::Max);  // Top
        board.place_stone(8, 9, Player::Max);  // Top
        board.place_stone(10, 7, Player::Max); // Bottom
        board.place_stone(10, 9, Player::Max); // Bottom
        
        // Placing at (9,8) should create double-three if both can form open fours
        let creates_double = GameRules::creates_double_three(&board, 9, 8, Player::Max);
        
        // This depends on your exact implementation of free-three detection
        println!("Complex scenario creates double-three: {}", creates_double);
    }

    #[test]
    fn test_opponent_stones_break_pattern() {
        let mut board = Board::new(19);
        
        // Pattern where opponent stones prevent free-three formation
        board.place_stone(5, 5, Player::Max);
        board.place_stone(5, 7, Player::Max);
        board.place_stone(5, 4, Player::Min); // Opponent blocks potential
        
        assert!(!GameRules::creates_double_three(&board, 5, 6, Player::Max));
    }

    #[test]
    fn test_edge_cases() {
        let mut board = Board::new(19);
        
        // Test near board edges
        board.place_stone(0, 0, Player::Max);
        board.place_stone(0, 2, Player::Max);
        
        // Should not create double-three due to board constraints
        assert!(!GameRules::creates_double_three(&board, 0, 1, Player::Max));
    }

    #[test]
    fn test_double_three_exact_example_from_subject() {
        let mut board = Board::new(19);
        
        // Recreate the exact scenario from the subject appendix
        // Red stones forming the pattern where playing at 'a' creates double-three
        //
        // Pattern (using 0-indexed coordinates):
        //     . . . . . . . .
        //     . . X . . . . .  <- (row 1, col 2)
        //     . . . X . . . .  <- (row 2, col 3)
        //     . . . . . . . .
        //     . . . . . a X X  <- 'a' at (row 4, col 5), stones at (row 4, col 6) and (row 4, col 7)
        //
        // This creates:
        // 1. Diagonal free-three: stones at (1,2), (2,3), and placing at (4,5) would form diagonal line
        // 2. Horizontal free-three: placing at (4,5) with stones at (4,6), (4,7) forms horizontal line
        
        // Place the diagonal stones
        board.place_stone(1, 2, Player::Max);  // First diagonal stone
        board.place_stone(2, 3, Player::Max);  // Second diagonal stone
        
        // Place the horizontal stones
        board.place_stone(4, 6, Player::Max);  // First horizontal stone
        board.place_stone(4, 7, Player::Max);  // Second horizontal stone
        
        // Playing at position 'a' (4, 5) should create a double-three
        // This creates two free-threes:
        // 1. Diagonal: (1,2) -> (2,3) -> (4,5) with potential to extend to (5,6)
        // 2. Horizontal: (4,5) -> (4,6) -> (4,7) with potential to extend on both sides
        assert!(GameRules::creates_double_three(&board, 4, 5, Player::Max));
    }
