use gomoku::core::board::{Board, Player};

#[test]
fn test_board_creation() {
    let board = Board::new(19);
    assert_eq!(board.size, 19);
    assert!(board.is_empty());
    
    for i in 0..19 {
        for j in 0..19 {
            assert!(board.is_empty_position(i, j));
            assert_eq!(board.get_player(i, j), None);
        }
    }
}

#[test]
fn test_board_center() {
    let board = Board::new(19);
    assert_eq!(board.center(), (9, 9));
    
    let board = Board::new(15);
    assert_eq!(board.center(), (7, 7));
}

#[test]
fn test_place_and_remove_stone() {
    let mut board = Board::new(19);
    
    // Place a stone
    board.place_stone(5, 5, Player::Max);
    assert_eq!(board.get_player(5, 5), Some(Player::Max));
    assert!(!board.is_empty_position(5, 5));
    assert!(!board.is_empty());
    
    // Remove a stone
    board.remove_stone(5, 5);
    assert_eq!(board.get_player(5, 5), None);
    assert!(board.is_empty_position(5, 5));
    assert!(board.is_empty());
}

#[test]
fn test_bitboard_operations() {
    let mut bits = vec![0u64; 2];
    
    // Test setting bits across u64 boundaries
    Board::set_bit(&mut bits, 0);    // First bit of first u64
    Board::set_bit(&mut bits, 63);   // Last bit of first u64
    Board::set_bit(&mut bits, 64);   // First bit of second u64
    Board::set_bit(&mut bits, 127);  // Last bit of second u64
    
    // Verify bits are set correctly
    assert!(Board::is_bit_set(&bits, 0));
    assert!(Board::is_bit_set(&bits, 63));
    assert!(Board::is_bit_set(&bits, 64));
    assert!(Board::is_bit_set(&bits, 127));
    
    // Verify other bits are not set
    assert!(!Board::is_bit_set(&bits, 1));
    assert!(!Board::is_bit_set(&bits, 62));
    assert!(!Board::is_bit_set(&bits, 65));
    assert!(!Board::is_bit_set(&bits, 126));
    
    // Test clearing bits
    Board::clear_bit(&mut bits, 63);
    Board::clear_bit(&mut bits, 127);
    
    assert!(!Board::is_bit_set(&bits, 63));
    assert!(!Board::is_bit_set(&bits, 127));
    assert!(Board::is_bit_set(&bits, 0));
    assert!(Board::is_bit_set(&bits, 64));
}

#[test]
fn test_bitboard_bounds_safety() {
    let mut bits = vec![0u64; 2];
    
    // Test operations on out-of-bounds indices
    Board::set_bit(&mut bits, 200);   // Should not panic
    Board::clear_bit(&mut bits, 300); // Should not panic
    
    // Should return false for out-of-bounds
    assert!(!Board::is_bit_set(&bits, 200));
    assert!(!Board::is_bit_set(&bits, 300));
}

#[test]
fn test_board_size_calculations() {
    // Test different board sizes and their u64 requirements
    let board_13 = Board::new(13);
    assert_eq!(board_13.total_cells, 169);
    assert_eq!(board_13.u64_count, 3); // 169 bits requires 3 u64s
    
    let board_15 = Board::new(15);
    assert_eq!(board_15.total_cells, 225);
    assert_eq!(board_15.u64_count, 4); // 225 bits requires 4 u64s
    
    let board_19 = Board::new(19);
    assert_eq!(board_19.total_cells, 361);
    assert_eq!(board_19.u64_count, 6); // 361 bits requires 6 u64s
}

#[test]
fn test_stone_counting() {
    let mut board = Board::new(15);
    
    assert_eq!(board.count_stones(), 0);
    assert_eq!(board.count_player_stones(Player::Max), 0);
    assert_eq!(board.count_player_stones(Player::Min), 0);
    
    // Place some stones
    board.place_stone(7, 7, Player::Max);
    board.place_stone(7, 8, Player::Min);
    board.place_stone(8, 7, Player::Max);
    board.place_stone(6, 7, Player::Min);
    
    assert_eq!(board.count_stones(), 4);
    assert_eq!(board.count_player_stones(Player::Max), 2);
    assert_eq!(board.count_player_stones(Player::Min), 2);
    
    // Remove a stone
    board.remove_stone(7, 8);
    
    assert_eq!(board.count_stones(), 3);
    assert_eq!(board.count_player_stones(Player::Max), 2);
    assert_eq!(board.count_player_stones(Player::Min), 1);
}

#[test]
fn test_bitboard_consistency() {
    let mut board = Board::new(15);
    
    // Place stones and verify bitboard consistency
    let positions = vec![
        (0, 0, Player::Max),
        (14, 14, Player::Min),
        (7, 7, Player::Max),
        (0, 14, Player::Min),
        (14, 0, Player::Max),
    ];
    
    for (row, col, player) in positions {
        board.place_stone(row, col, player);
        
        let idx = board.index(row, col);
        
        // Verify player-specific bitboard is set
        match player {
            Player::Max => assert!(Board::is_bit_set(&board.max_bits, idx)),
            Player::Min => assert!(Board::is_bit_set(&board.min_bits, idx)),
        }
        
        // Verify occupied bitboard is set
        assert!(Board::is_bit_set(&board.occupied, idx));
        
        // Verify get_player returns correct player
        assert_eq!(board.get_player(row, col), Some(player));
        
        // Verify position is not empty
        assert!(!board.is_empty_position(row, col));
    }
}

#[test]
fn test_large_board_bitboard_operations() {
    let mut board = Board::new(19);
    
    // Test operations near the end of bitboard arrays
    let last_row = 18;
    let last_col = 18;
    
    board.place_stone(last_row, last_col, Player::Max);
    assert_eq!(board.get_player(last_row, last_col), Some(Player::Max));
    
    // Test index calculation for last position
    let last_idx = board.index(last_row, last_col);
    assert_eq!(last_idx, 18 * 19 + 18); // 360
    
    // Verify bitboard operations work correctly for high indices
    assert!(Board::is_bit_set(&board.max_bits, last_idx));
    assert!(Board::is_bit_set(&board.occupied, last_idx));
    assert!(!Board::is_bit_set(&board.min_bits, last_idx));
}

#[test]
fn test_is_adjacent_to_stone() {
    let mut board = Board::new(19);
    
    // Place a stone at center
    board.place_stone(9, 9, Player::Max);
    
    // Test adjacent positions
    assert!(board.is_adjacent_to_stone(8, 8)); // diagonal
    assert!(board.is_adjacent_to_stone(8, 9)); // vertical
    assert!(board.is_adjacent_to_stone(9, 8)); // horizontal
    assert!(board.is_adjacent_to_stone(10, 10)); // diagonal
    assert!(board.is_adjacent_to_stone(10, 9)); // vertical
    assert!(board.is_adjacent_to_stone(9, 10)); // horizontal
    
    // Test non-adjacent positions
    assert!(!board.is_adjacent_to_stone(7, 7));
    assert!(!board.is_adjacent_to_stone(11, 11));
    assert!(!board.is_adjacent_to_stone(0, 0));
}

#[test]
fn test_board_edges() {
    let mut board = Board::new(19);
    
    // Test corner positions
    board.place_stone(0, 0, Player::Max);
    assert!(board.is_adjacent_to_stone(0, 1));
    assert!(board.is_adjacent_to_stone(1, 0));
    assert!(board.is_adjacent_to_stone(1, 1));
    assert!(!board.is_adjacent_to_stone(2, 2));
    
    // Test edge positions
    board.place_stone(0, 5, Player::Min);
    assert!(board.is_adjacent_to_stone(0, 4));
    assert!(board.is_adjacent_to_stone(0, 6));
    assert!(board.is_adjacent_to_stone(1, 5));
    assert!(board.is_adjacent_to_stone(1, 4));
    assert!(board.is_adjacent_to_stone(1, 6));
}

#[test]
fn test_player_opponent() {
    assert_eq!(Player::Max.opponent(), Player::Min);
    assert_eq!(Player::Min.opponent(), Player::Max);
}

