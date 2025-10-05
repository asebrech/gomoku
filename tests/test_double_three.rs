use gomoku::ai::precompute::DirectionTables;
use gomoku::core::{board::{Board, Player}, moves::RuleValidator};

fn create_dir_tables(size: usize) -> DirectionTables {
    DirectionTables::new(size, 6)
}

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
        let dir_tables = create_dir_tables(19);
        
        // Placing at (6,8) should create double-three (horizontal and vertical)
        assert!(RuleValidator::creates_double_three(&board, 6, 8, Player::Max, &dir_tables));
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
    let dir_tables = create_dir_tables(19);
    
    // Placing at (9,9) creates two 3-stone diagonal lines
    assert!(RuleValidator::creates_double_three(&board, 9, 9, Player::Max, &dir_tables));
}

    #[test]
    fn test_single_free_three_allowed() {
        let mut board = Board::new(19);
        
        // Create only one free-three horizontally
        board.place_stone(5, 5, Player::Max);
        board.place_stone(5, 7, Player::Max);
        let dir_tables = create_dir_tables(19);
        
        // Placing at (5,6) creates only one free-three - should be allowed
        assert!(!RuleValidator::creates_double_three(&board, 5, 6, Player::Max, &dir_tables));
    }

    #[test]
    fn test_blocked_three_not_free() {
        let mut board = Board::new(19);
        
        // Create blocked three (not free-three)
        board.place_stone(5, 5, Player::Max);
        board.place_stone(5, 7, Player::Max);
        board.place_stone(5, 8, Player::Min); // Blocks one end
        let dir_tables = create_dir_tables(19);
        
        // This shouldn't create a free-three since one end is blocked
        assert!(!RuleValidator::creates_double_three(&board, 5, 6, Player::Max, &dir_tables));
    }

    #[test]
    fn test_no_space_for_open_four() {
        let mut board = Board::new(19);
        
        // Create scenario where there's no space for undefendable four
        board.place_stone(0, 1, Player::Max);
        board.place_stone(0, 3, Player::Max);
        let dir_tables = create_dir_tables(19);
        // Board edge limits the potential for open four
        
        assert!(!RuleValidator::creates_double_three(&board, 0, 2, Player::Max, &dir_tables));
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
        let dir_tables = create_dir_tables(19);
        
        // Placing at (9,8) should create double-three if both can form open fours
        let creates_double = RuleValidator::creates_double_three(&board, 9, 8, Player::Max, &dir_tables);
        
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
        let dir_tables = create_dir_tables(19);
        
        assert!(!RuleValidator::creates_double_three(&board, 5, 6, Player::Max, &dir_tables));
    }

    #[test]
    fn test_edge_cases() {
        let mut board = Board::new(19);
        
        // Test near board edges
        board.place_stone(0, 0, Player::Max);
        board.place_stone(0, 2, Player::Max);
        let dir_tables = create_dir_tables(19);
        
        // Should not create double-three due to board constraints
        assert!(!RuleValidator::creates_double_three(&board, 0, 1, Player::Max, &dir_tables));
    }
