use gomoku::core::board::{Board, Player};

#[test]
fn test_board_creation() {
    let board = Board::new(19);
    assert_eq!(board.size(), 19);
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
    let mut board1 = Board::new(19);
    let mut board2 = Board::new(19);
    
    // Empty boards should have same hash
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
    let board1 = Board::new(19);
    let board2 = Board::new(15);
    
    // Different sized boards should have different hashes
    assert_ne!(board1.hash(), board2.hash());
}

