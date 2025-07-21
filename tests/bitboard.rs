use gomoku::core::board::{Board, Player};

fn create_test_board(size: usize, stones: Vec<((usize, usize), Player)>) -> Board {
    let mut board = Board::new(size);
    for ((row, col), player) in stones {
        board.place_stone(row, col, player);
    }
    board
}

// Tests for index
#[test]
fn test_index_basic() {
    let board = create_test_board(5, vec![]);
    assert_eq!(board.index(0, 0), 0);
    assert_eq!(board.index(1, 2), 7); // 1*5 + 2 = 7
    assert_eq!(board.index(4, 4), 24); // Last position
}

#[test]
fn test_index_edge() {
    let board = create_test_board(3, vec![]);
    assert_eq!(board.index(2, 2), 8);
}

// Tests for set_bit
#[test]
fn test_set_bit_first() {
    let mut bits = vec![0u64; 1];
    Board::set_bit(&mut bits, 0);
    assert_eq!(bits[0], 1u64);
    Board::set_bit(&mut bits, 5);
    assert_eq!(bits[0], 1u64 | (1u64 << 5));
}

#[test]
fn test_set_bit_boundary() {
    let mut bits = vec![0u64; 2];
    Board::set_bit(&mut bits, 63);
    assert_eq!(bits[0], 1u64 << 63);
    Board::set_bit(&mut bits, 64);
    assert_eq!(bits[1], 1u64);
}

// Tests for clear_bit
#[test]
fn test_clear_bit() {
    let mut bits = vec![1u64 << 3, 0u64];
    Board::clear_bit(&mut bits, 3);
    assert_eq!(bits[0], 0u64);
}

#[test]
fn test_clear_bit_unset() {
    let mut bits = vec![0u64; 1];
    Board::clear_bit(&mut bits, 0);
    assert_eq!(bits[0], 0u64);
}

// Tests for is_bit_set
#[test]
fn test_is_bit_set_true() {
    let bits = vec![1u64 << 10];
    assert!(Board::is_bit_set(&bits, 10));
}

#[test]
fn test_is_bit_set_false() {
    let bits = vec![0u64];
    assert!(!Board::is_bit_set(&bits, 0));
}

#[test]
fn test_is_bit_set_boundary() {
    let bits = vec![0u64, 1u64];
    assert!(Board::is_bit_set(&bits, 64));
}

// Tests for occupied updates
#[test]
fn test_occupied_on_place() {
    let mut board = Board::new(5);
    board.place_stone(2, 2, Player::Max);
    let idx = board.index(2, 2);
    assert!(Board::is_bit_set(&board.occupied, idx));
    assert!(Board::is_bit_set(&board.max_bits, idx));
    assert!(!Board::is_bit_set(&board.min_bits, idx));
}

#[test]
fn test_occupied_on_remove() {
    let mut board = create_test_board(5, vec![((2, 2), Player::Max)]);
    board.remove_stone(2, 2);
    let idx = board.index(2, 2);
    assert!(!Board::is_bit_set(&board.occupied, idx));
    assert!(!Board::is_bit_set(&board.max_bits, idx));
}

// Tests for is_adjacent_to_stone
#[test]
fn test_is_adjacent_with_neighbors() {
    let board = create_test_board(5, vec![((2, 2), Player::Max), ((2, 3), Player::Min)]);
    assert!(board.is_adjacent_to_stone(2, 1)); // Adjacent to (2,2)
    assert!(board.is_adjacent_to_stone(1, 2)); // Adjacent vertically
}

#[test]
fn test_no_adjacency_isolated() {
    let board = create_test_board(5, vec![]);
    assert!(!board.is_adjacent_to_stone(0, 0));
}

#[test]
fn test_boundary_adjacency() {
    let board = create_test_board(5, vec![((0, 0), Player::Max)]);
    assert!(board.is_adjacent_to_stone(0, 1)); // Edge case
    assert!(!board.is_adjacent_to_stone(4, 4)); // Far corner
}

// Tests for is_full
#[test]
fn test_is_full_empty() {
    let board = Board::new(5);
    assert!(!board.is_full());
}

#[test]
fn test_is_full_complete() {
    let mut board = Board::new(5);
    for row in 0..5 {
        for col in 0..5 {
            board.place_stone(row, col, Player::Max);
        }
    }
    assert!(board.is_full());
}

#[test]
fn test_is_full_partial() {
    let board = create_test_board(5, vec![((0, 0), Player::Max), ((1, 1), Player::Min)]);
    assert!(!board.is_full());
}

// Tests for count_in_line
#[test]
fn test_count_in_line_horizontal() {
    let board = create_test_board(
        5,
        vec![
            ((2, 2), Player::Max),
            ((2, 3), Player::Max),
            ((2, 4), Player::Max),
        ],
    );
    assert_eq!(board.count_in_line(2, 2, Player::Max, (0, 1), 5), 3);
}

#[test]
fn test_count_in_line_diagonal_boundary() {
    let board = create_test_board(5, vec![((0, 0), Player::Min), ((1, 1), Player::Min)]);
    assert_eq!(board.count_in_line(0, 0, Player::Min, (1, 1), 3), 2);
    assert_eq!(board.count_in_line(2, 2, Player::Min, (1, 1), 3), 0); // No stones
}

#[test]
fn test_count_in_line_zero() {
    let board = Board::new(5);
    assert_eq!(board.count_in_line(2, 2, Player::Max, (1, 0), 5), 0);
}

// Tests for get_empty_positions
#[test]
fn test_get_empty_positions_all() {
    let board = Board::new(3); // 9 cells
    let empties = board.get_empty_positions();
    assert_eq!(empties.len(), 9);
    assert!(empties.contains(&(0, 0)));
    assert!(empties.contains(&(2, 2)));
}

#[test]
fn test_get_empty_positions_partial() {
    let board = create_test_board(3, vec![((0, 0), Player::Max), ((1, 1), Player::Min)]);
    let empties = board.get_empty_positions();
    assert_eq!(empties.len(), 7); // 9 - 2
    assert!(!empties.contains(&(0, 0)));
    assert!(empties.contains(&(2, 2)));
}

#[test]
fn test_get_empty_positions_none() {
    let mut board = Board::new(2); // 4 cells
    for row in 0..2 {
        for col in 0..2 {
            board.place_stone(row, col, Player::Max);
        }
    }
    let empties = board.get_empty_positions();
    assert_eq!(empties.len(), 0);
}
