use gomoku::core::board::{Board, Player};
use std::time::Instant;

#[test]
fn test_bitboard_different_sizes() {
    // Test various board sizes to ensure bitboard works correctly
    let sizes = [5, 9, 13, 15, 19, 25];
    
    for &size in &sizes {
        let mut board = Board::new(size);
        
        // Test basic operations
        assert!(board.is_empty());
        assert_eq!(board.size, size);
        
        // Place some stones
        let center = board.center();
        board.place_stone(center.0, center.1, Player::Max);
        assert!(!board.is_empty());
        assert_eq!(board.get_player(center.0, center.1), Some(Player::Max));
        
        // Test stone counting
        assert_eq!(board.count_player_stones(Player::Max), 1);
        assert_eq!(board.count_player_stones(Player::Min), 0);
        
        // Test adjacency
        if center.0 > 0 {
            assert!(board.is_adjacent_to_stone(center.0 - 1, center.1));
        }
        if center.1 > 0 {
            assert!(board.is_adjacent_to_stone(center.0, center.1 - 1));
        }
        
        // Remove stone
        board.remove_stone(center.0, center.1);
        assert!(board.is_empty());
    }
}

#[test]
fn test_bitboard_large_board_operations() {
    let mut board = Board::new(25);
    
    // Place stones in a pattern
    for i in 0..5 {
        for j in 0..5 {
            board.place_stone(i, j, if (i + j) % 2 == 0 { Player::Max } else { Player::Min });
        }
    }
    
    // Verify count
    let max_count = board.count_player_stones(Player::Max);
    let min_count = board.count_player_stones(Player::Min);
    assert!(max_count > 0);
    assert!(min_count > 0);
    assert_eq!(max_count + min_count, 25);
    
    // Test occupied positions iterator
    let positions: Vec<_> = board.occupied_positions().collect();
    assert_eq!(positions.len(), 25);
    
    // Verify all positions are in the expected range
    for (row, col, _player) in positions {
        assert!(row < 5);
        assert!(col < 5);
    }
}

#[test]
fn test_bitboard_performance_basic() {
    let board_size = 19;
    let mut board = Board::new(board_size);
    
    let start = Instant::now();
    
    // Perform many operations
    for i in 0..1000 {
        let row = i % board_size;
        let col = (i * 7) % board_size;
        let player = if i % 2 == 0 { Player::Max } else { Player::Min };
        
        if board.is_empty_position(row, col) {
            board.place_stone(row, col, player);
        }
        
        // Test adjacency check (this should be fast with bitboards)
        board.is_adjacent_to_stone(row, col);
        
        // Test player lookup
        board.get_player(row, col);
    }
    
    let duration = start.elapsed();
    println!("Bitboard operations took: {:?}", duration);
    
    // This should complete very quickly with bitboard operations
    assert!(duration.as_millis() < 100, "Bitboard operations should be fast");
}

#[test]
fn test_hash_consistency_bitboard() {
    let mut board1 = Board::new(19);
    let mut board2 = Board::new(19);
    
    // Same operations should produce same hash
    board1.place_stone(9, 9, Player::Max);
    board2.place_stone(9, 9, Player::Max);
    assert_eq!(board1.hash(), board2.hash());
    
    // Different operations should produce different hash
    board1.place_stone(10, 10, Player::Min);
    board2.place_stone(8, 8, Player::Min);
    assert_ne!(board1.hash(), board2.hash());
}
