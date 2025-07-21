use gomoku::core::board::{Board, Player};
use gomoku::core::rules::WinChecker;

#[test]
fn test_horizontal_win() {
    let mut board = Board::new(19);

    // Place 5 stones horizontally
    for i in 0..5 {
        board.place_stone(9, 5 + i, Player::Max);
    }

    // Test win detection from different positions
    assert!(WinChecker::check_win_around(&board, 9, 5, 5));
    assert!(WinChecker::check_win_around(&board, 9, 7, 5));
    assert!(WinChecker::check_win_around(&board, 9, 9, 5));
}

#[test]
fn test_vertical_win() {
    let mut board = Board::new(19);

    // Place 5 stones vertically
    for i in 0..5 {
        board.place_stone(5 + i, 9, Player::Max);
    }

    // Test win detection
    assert!(WinChecker::check_win_around(&board, 5, 9, 5));
    assert!(WinChecker::check_win_around(&board, 7, 9, 5));
    assert!(WinChecker::check_win_around(&board, 9, 9, 5));
}

#[test]
fn test_diagonal_win() {
    let mut board = Board::new(19);

    // Place 5 stones diagonally
    for i in 0..5 {
        board.place_stone(5 + i, 5 + i, Player::Max);
    }

    // Test win detection
    assert!(WinChecker::check_win_around(&board, 5, 5, 5));
    assert!(WinChecker::check_win_around(&board, 7, 7, 5));
    assert!(WinChecker::check_win_around(&board, 9, 9, 5));
}

#[test]
fn test_anti_diagonal_win() {
    let mut board = Board::new(19);

    // Place 5 stones anti-diagonally
    for i in 0..5 {
        board.place_stone(5 + i, 9 - i, Player::Max);
    }

    // Test win detection
    assert!(WinChecker::check_win_around(&board, 5, 9, 5));
    assert!(WinChecker::check_win_around(&board, 7, 7, 5));
    assert!(WinChecker::check_win_around(&board, 9, 5, 5));
}

#[test]
fn test_no_win_four_in_row() {
    let mut board = Board::new(19);

    // Place only 4 stones horizontally
    for i in 0..4 {
        board.place_stone(9, 5 + i, Player::Max);
    }

    // Should not detect win
    assert!(!WinChecker::check_win_around(&board, 9, 5, 5));
    assert!(!WinChecker::check_win_around(&board, 9, 8, 5));
}

#[test]
fn test_blocked_line_no_win() {
    let mut board = Board::new(19);

    // Place 4 stones with opponent stone blocking
    board.place_stone(9, 5, Player::Max);
    board.place_stone(9, 6, Player::Max);
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Min); // Blocking stone

    // Should not detect win
    assert!(!WinChecker::check_win_around(&board, 9, 5, 5));
    assert!(!WinChecker::check_win_around(&board, 9, 8, 5));
}

#[test]
fn test_capture_win_max() {
    // Test capture win for Max player (5 pairs = 10 captures)
    assert_eq!(WinChecker::check_capture_win(5, 0), Some(Player::Max));
    assert_eq!(WinChecker::check_capture_win(6, 0), Some(Player::Max));
    assert_eq!(WinChecker::check_capture_win(4, 0), None);
}

#[test]
fn test_capture_win_min() {
    // Test capture win for Min player (5 pairs = 10 captures)
    assert_eq!(WinChecker::check_capture_win(0, 5), Some(Player::Min));
    assert_eq!(WinChecker::check_capture_win(0, 6), Some(Player::Min));
    assert_eq!(WinChecker::check_capture_win(0, 4), None);
}

#[test]
fn test_no_capture_win() {
    // Test no capture win
    assert_eq!(WinChecker::check_capture_win(4, 4), None);
    assert_eq!(WinChecker::check_capture_win(3, 2), None);
    assert_eq!(WinChecker::check_capture_win(0, 0), None);
}

#[test]
fn test_win_different_conditions() {
    let mut board = Board::new(19);

    // Test with 4-in-a-row win condition
    for i in 0..4 {
        board.place_stone(9, 5 + i, Player::Max);
    }

    assert!(WinChecker::check_win_around(&board, 9, 5, 4));
    assert!(!WinChecker::check_win_around(&board, 9, 5, 5));

    // Test with 6-in-a-row win condition
    for i in 4..6 {
        board.place_stone(9, 5 + i, Player::Max);
    }

    assert!(WinChecker::check_win_around(&board, 9, 5, 6));
    assert!(WinChecker::check_win_around(&board, 9, 5, 5));
}

#[test]
fn test_edge_case_wins() {
    let mut board = Board::new(19);

    // Test win at board edge
    for i in 0..5 {
        board.place_stone(0, i, Player::Max);
    }

    assert!(WinChecker::check_win_around(&board, 0, 0, 5));
    assert!(WinChecker::check_win_around(&board, 0, 4, 5));

    for i in 0..5 {
        board.remove_stone(0, i);
    }
    for i in 0..5 {
        board.place_stone(i, 0, Player::Min);
    }

    assert!(WinChecker::check_win_around(&board, 0, 0, 5));
    assert!(WinChecker::check_win_around(&board, 4, 0, 5));
}
