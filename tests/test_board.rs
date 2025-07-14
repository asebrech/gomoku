use gomoku::core::board::{Board, Player, initialize_zobrist};

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
    initialize_zobrist();
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

#[test]
fn test_board_hash_consistency() {
    initialize_zobrist();
    let mut board1 = Board::new(19);
    let mut board2 = Board::new(19);
    
    // Empty boards should have same hash (both zero)
    assert_eq!(board1.hash(), board2.hash());
    
    // Same moves should produce same hash
    board1.place_stone(5, 5, Player::Max);
    board2.place_stone(5, 5, Player::Max);
    assert_eq!(board1.hash(), board2.hash());
    
    // Different moves should produce different hash
    board1.place_stone(6, 6, Player::Min);
    board2.place_stone(7, 7, Player::Min);
    assert_ne!(board1.hash(), board2.hash());
}

#[test]
fn test_board_hash_different_sizes() {
    initialize_zobrist();
    let board1 = Board::new(19);
    let board2 = Board::new(15);
    
    // Different sized boards should have same hash if empty (both zero)
    assert_eq!(board1.hash(), board2.hash());
}

#[test]
fn test_bitboard_operations() {
    let mut board = Board::new(19);
    
    // Test position to bit conversion
    let (chunk_idx, bit_idx) = Board::position_to_bit_index(0, 0, 19);
    assert_eq!(chunk_idx, 0);
    assert_eq!(bit_idx, 0);
    
    let (chunk_idx, bit_idx) = Board::position_to_bit_index(18, 18, 19);
    assert_eq!(chunk_idx, 2); // 18*19 + 18 = 360, 360/128 = 2
    assert_eq!(bit_idx, 104); // 360 % 128 = 104
    
    // Test placing stones in different chunks
    board.place_stone(0, 0, Player::Max);   // chunk 0
    board.place_stone(7, 0, Player::Min);   // chunk 1 (7*19 = 133, 133/128 = 1)
    board.place_stone(18, 18, Player::Max); // chunk 2
    
    assert_eq!(board.get_player(0, 0), Some(Player::Max));
    assert_eq!(board.get_player(7, 0), Some(Player::Min));
    assert_eq!(board.get_player(18, 18), Some(Player::Max));
}

#[test]
fn test_out_of_bounds() {
    let board = Board::new(19);
    
    // Test out of bounds access
    assert_eq!(board.get_player(19, 0), None);
    assert_eq!(board.get_player(0, 19), None);
    assert_eq!(board.get_player(20, 20), None);
    
    // Test valid position check
    assert!(board.is_valid_position(0, 0));
    assert!(board.is_valid_position(18, 18));
    assert!(!board.is_valid_position(19, 0));
    assert!(!board.is_valid_position(0, 19));
}

#[test]
fn test_multiple_players() {
    let mut board = Board::new(19);
    
    // Place stones for both players
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 6, Player::Min);
    board.place_stone(6, 5, Player::Max);
    board.place_stone(6, 6, Player::Min);
    
    // Verify positions
    assert_eq!(board.get_player(5, 5), Some(Player::Max));
    assert_eq!(board.get_player(5, 6), Some(Player::Min));
    assert_eq!(board.get_player(6, 5), Some(Player::Max));
    assert_eq!(board.get_player(6, 6), Some(Player::Min));
    
    // Verify empty positions
    assert_eq!(board.get_player(5, 7), None);
    assert_eq!(board.get_player(7, 5), None);
}

#[test]
fn test_zobrist_hash_updates() {
    initialize_zobrist();
    let mut board = Board::new(19);
    
    let initial_hash = board.hash();
    
    // Place a stone
    board.place_stone(5, 5, Player::Max);
    let hash_after_place = board.hash();
    assert_ne!(initial_hash, hash_after_place);
    
    // Remove the stone
    board.remove_stone(5, 5);
    let hash_after_remove = board.hash();
    assert_eq!(initial_hash, hash_after_remove);
}

#[test]
fn test_hash_reversibility() {
    initialize_zobrist();
    let mut board = Board::new(19);
    
    let original_hash = board.hash();
    
    // Place and remove multiple stones
    board.place_stone(5, 5, Player::Max);
    board.place_stone(6, 6, Player::Min);
    board.place_stone(7, 7, Player::Max);
    
    board.remove_stone(5, 5);
    board.remove_stone(6, 6);
    board.remove_stone(7, 7);
    
    // Should return to original hash
    assert_eq!(board.hash(), original_hash);
}

