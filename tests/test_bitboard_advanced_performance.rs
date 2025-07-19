use gomoku::core::board::{Board, Player};
use std::time::Instant;

#[test]
fn test_bitboard_optimized_operations() {
    let mut board = Board::new(19);
    
    // Fill half the board for more realistic conditions
    for i in 0..19 {
        for j in 0..10 {
            board.place_stone(i, j, if (i + j) % 2 == 0 { Player::Max } else { Player::Min });
        }
    }
    
    let start = Instant::now();
    
    // Test the optimized methods
    for _ in 0..10000 {
        // Test is_empty (optimized with bitwise OR)
        let _ = board.is_empty();
        
        // Test is_full (optimized with precomputed mask)
        let _ = board.is_full();
        
        // Test has_empty_positions (optimized bitwise check)
        let _ = board.has_empty_positions();
        
        // Test count operations (using bit counting)
        let _ = board.count_player_stones(Player::Max);
        let _ = board.count_empty_positions();
        
        // Test player stone presence (optimized with any())
        let _ = board.has_player_stones(Player::Max);
        let _ = board.has_player_stones(Player::Min);
    }
    
    let duration = start.elapsed();
    println!("10,000 optimized bitboard operations took: {:?}", duration);
    
    // Should be very fast with our optimizations
    assert!(duration.as_millis() < 50, "Optimized operations should be very fast");
}

#[test]
fn test_is_full_performance() {
    let mut board = Board::new(19);
    
    // Fill most of the board
    for i in 0..19 {
        for j in 0..19 {
            if i != 18 || j != 18 { // Leave one position empty
                board.place_stone(i, j, if (i + j) % 2 == 0 { Player::Max } else { Player::Min });
            }
        }
    }
    
    let start = Instant::now();
    
    // Test is_full many times
    for _ in 0..100000 {
        let _ = board.is_full();
    }
    
    let duration = start.elapsed();
    println!("100,000 is_full() calls took: {:?}", duration);
    
    // With precomputed mask, this should be very fast
    assert!(duration.as_millis() < 50, "is_full() with precomputed mask should be very fast");
}

#[test]
fn test_adjacency_check_performance() {
    let mut board = Board::new(19);
    
    // Place some stones in various positions
    board.place_stone(9, 9, Player::Max);
    board.place_stone(5, 5, Player::Min);
    board.place_stone(15, 15, Player::Max);
    board.place_stone(0, 0, Player::Min);
    board.place_stone(18, 18, Player::Max);
    
    let start = Instant::now();
    
    // Test adjacency checks for interior positions (optimized path)
    for _ in 0..50000 {
        let _ = board.is_adjacent_to_stone(10, 10); // Interior position
        let _ = board.is_adjacent_to_stone(8, 8);   // Interior position
    }
    
    let duration_interior = start.elapsed();
    
    let start = Instant::now();
    
    // Test adjacency checks for edge positions (slower path)
    for _ in 0..50000 {
        let _ = board.is_adjacent_to_stone(0, 5);   // Edge position
        let _ = board.is_adjacent_to_stone(18, 10); // Edge position
    }
    
    let duration_edge = start.elapsed();
    
    println!("Interior adjacency checks took: {:?}", duration_interior);
    println!("Edge adjacency checks took: {:?}", duration_edge);
    
    // Interior checks should be faster than edge checks
    assert!(duration_interior <= duration_edge, "Interior adjacency checks should be faster or equal");
}

#[test]
fn test_different_board_sizes_performance() {
    let sizes = [9, 13, 15, 19, 25];
    
    for &size in &sizes {
        let mut board = Board::new(size);
        
        // Fill board partially
        for i in 0..size/2 {
            for j in 0..size/2 {
                board.place_stone(i, j, if (i + j) % 2 == 0 { Player::Max } else { Player::Min });
            }
        }
        
        let start = Instant::now();
        
        // Perform various operations
        for _ in 0..1000 {
            let _ = board.is_empty();
            let _ = board.is_full();
            let _ = board.count_player_stones(Player::Max);
            let _ = board.has_empty_positions();
        }
        
        let duration = start.elapsed();
        println!("Board size {}: 4000 operations took {:?}", size, duration);
        
        // Should scale well with board size
        assert!(duration.as_millis() < 100, "Operations should be fast for all board sizes");
    }
}
