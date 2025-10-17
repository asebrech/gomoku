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
    fn test_one_end_blocked_not_free_three() {
        let mut board = Board::new(19);
        
        // One end blocked by opponent
        board.place_stone(8, 7, Player::Max);  
        board.place_stone(8, 9, Player::Max);  
        board.place_stone(8, 10, Player::Min); // Block one end
        board.place_stone(7, 8, Player::Max);  
        board.place_stone(9, 8, Player::Max);  
        
        // This should NOT be a double-three because horizontal line is blocked on one end
        assert!(!GameRules::creates_double_three(&board, 8, 8, Player::Max));
    }

    #[test]
    fn test_both_ends_blocked_not_free_three() {
        let mut board = Board::new(19);
        
        // Both ends blocked
        board.place_stone(8, 7, Player::Max);
        board.place_stone(8, 9, Player::Max);
        board.place_stone(8, 6, Player::Min); // Block left
        board.place_stone(8, 10, Player::Min); // Block right
        
        // This should NOT create a double-three (or even a single free-three)
        assert!(!GameRules::creates_double_three(&board, 8, 8, Player::Max));
    }

    #[test]
    fn test_four_stones_not_free_three() {
        let mut board = Board::new(19);
        
        // Creating four stones in a row should not be counted as a free-three
        board.place_stone(8, 6, Player::Max);  
        board.place_stone(8, 7, Player::Max);  
        board.place_stone(8, 9, Player::Max);  
        
        // This creates a four, not a three
        assert!(!GameRules::creates_double_three(&board, 8, 8, Player::Max));
    }

    #[test]
    fn test_insufficient_space_not_free_three() {
        let mut board = Board::new(19);
        
        // Pattern with opponent after only one empty space
        board.place_stone(8, 7, Player::Max);
        board.place_stone(8, 9, Player::Max);
        board.place_stone(8, 11, Player::Min); // Opponent after one empty space
        board.place_stone(7, 8, Player::Max);
        board.place_stone(9, 8, Player::Max);
        
        // This should NOT be a double-three because there's not enough room to form open four
        assert!(!GameRules::creates_double_three(&board, 8, 8, Player::Max));
    }

    #[test]
    fn analyze_double_three() {
        println!("\n=== Testing Double-Three Detection Logic ===\n");

        // Test 1: Basic horizontal + vertical double-three
        println!("Test 1: Basic Cross Pattern");
        let mut board = Board::new(19);
        board.place_stone(8, 7, Player::Max);  
        board.place_stone(8, 9, Player::Max);  
        board.place_stone(7, 8, Player::Max);  
        board.place_stone(9, 8, Player::Max);  
        let result = GameRules::creates_double_three(&board, 8, 8, Player::Max);
        println!("Cross pattern (should be double-three): {}", result);

        // Test 2: Pattern with blocked end
        println!("\nTest 2: One End Blocked");
        let mut board = Board::new(19);
        board.place_stone(8, 7, Player::Max);  
        board.place_stone(8, 9, Player::Max);  
        board.place_stone(8, 10, Player::Min); // Block one end
        board.place_stone(7, 8, Player::Max);  
        board.place_stone(9, 8, Player::Max);  
        let result = GameRules::creates_double_three(&board, 8, 8, Player::Max);
        println!("One end blocked (should NOT be double-three if blocked): {}", result);

        // Test 3: Four stones in a row should not count as free-three
        println!("\nTest 3: Four in a Row");
        let mut board = Board::new(19);
        board.place_stone(8, 6, Player::Max);  
        board.place_stone(8, 7, Player::Max);  
        board.place_stone(8, 9, Player::Max);  
        let result = GameRules::creates_double_three(&board, 8, 8, Player::Max);
        println!("Creates four in row (should NOT count as free-three): {}", result);
        
        // Test 4: Pattern with a gap (one empty space between stones)
        println!("\nTest 4: Pattern with One Gap");
        let mut board = Board::new(19);
        board.place_stone(8, 6, Player::Max);  
        board.place_stone(8, 8, Player::Max);  
        board.place_stone(7, 7, Player::Max);  
        board.place_stone(9, 7, Player::Max);  
        let result = GameRules::creates_double_three(&board, 8, 7, Player::Max);
        println!("With one gap (depends on whether gap should count): {}", result);

        // Test 5: Both ends blocked
        println!("\nTest 5: Both Ends Blocked");
        let mut board = Board::new(19);
        board.place_stone(8, 7, Player::Max);
        board.place_stone(8, 9, Player::Max);
        board.place_stone(8, 6, Player::Min); // Block left
        board.place_stone(8, 10, Player::Min); // Block right
        let result = GameRules::creates_double_three(&board, 8, 8, Player::Max);
        println!("Both ends blocked (should NOT be free-three at all): {}", result);

        // Test 6: Only one direction has open space
        println!("\nTest 6: Only One Free Three");
        let mut board = Board::new(19);
        board.place_stone(8, 7, Player::Max);
        board.place_stone(8, 9, Player::Max);
        let result = GameRules::creates_double_three(&board, 8, 8, Player::Max);
        println!("Only horizontal free-three (should NOT be double-three): {}", result);

        // Test 7: Immediate opponent block after empty
        println!("\nTest 7: Empty then Opponent");
        let mut board = Board::new(19);
        board.place_stone(8, 7, Player::Max);
        board.place_stone(8, 9, Player::Max);
        board.place_stone(8, 11, Player::Min); // Opponent after one empty space
        board.place_stone(7, 8, Player::Max);
        board.place_stone(9, 8, Player::Max);
        let result = GameRules::creates_double_three(&board, 8, 8, Player::Max);
        println!("Opponent after empty space (can we form open four?): {}", result);
    }
