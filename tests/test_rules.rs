use gomoku::core::board::{Board, Player, initialize_zobrist};
use gomoku::core::rules::WinChecker;

#[test]
fn test_horizontal_win() {
    initialize_zobrist();
    let mut board = Board::new(19);

    // Place 5 stones horizontally
    for i in 0..5 {
        board.place_stone(9, 5 + i, Player::Max);
    }

    // Check win detection
    assert!(WinChecker::check_win_around(&board, 9, 7, 5));
    assert!(WinChecker::check_win_around(&board, 9, 5, 5));
    assert!(WinChecker::check_win_around(&board, 9, 9, 5));
}

#[test]
fn test_vertical_win() {
    initialize_zobrist();
    let mut board = Board::new(19);

    // Place 5 stones vertically
    for i in 0..5 {
        board.place_stone(5 + i, 9, Player::Max);
    }

    // Check win detection
    assert!(WinChecker::check_win_around(&board, 7, 9, 5));
    assert!(WinChecker::check_win_around(&board, 5, 9, 5));
    assert!(WinChecker::check_win_around(&board, 9, 9, 5));
}

#[test]
fn test_diagonal_win() {
    initialize_zobrist();
    let mut board = Board::new(19);

    // Place 5 stones diagonally
    for i in 0..5 {
        board.place_stone(5 + i, 5 + i, Player::Max);
    }

    // Check win detection
    assert!(WinChecker::check_win_around(&board, 7, 7, 5));
    assert!(WinChecker::check_win_around(&board, 5, 5, 5));
    assert!(WinChecker::check_win_around(&board, 9, 9, 5));
}

#[test]
fn test_anti_diagonal_win() {
    initialize_zobrist();
    let mut board = Board::new(19);

    // Place 5 stones anti-diagonally
    for i in 0..5 {
        board.place_stone(5 + i, 9 - i, Player::Max);
    }

    // Check win detection
    assert!(WinChecker::check_win_around(&board, 7, 7, 5));
    assert!(WinChecker::check_win_around(&board, 5, 9, 5));
    assert!(WinChecker::check_win_around(&board, 9, 5, 5));
}

#[test]
fn test_no_win_four_in_row() {
    initialize_zobrist();
    let mut board = Board::new(19);

    // Place only 4 stones horizontally
    for i in 0..4 {
        board.place_stone(9, 5 + i, Player::Max);
    }

    // Should not detect win
    assert!(!WinChecker::check_win_around(&board, 9, 7, 5));
}

#[test]
fn test_blocked_line_no_win() {
    initialize_zobrist();
    let mut board = Board::new(19);

    // Place stones with opponent blocking
    board.place_stone(9, 5, Player::Max);
    board.place_stone(9, 6, Player::Max);
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Min); // Blocking stone

    // Should not detect win
    assert!(!WinChecker::check_win_around(&board, 9, 7, 5));
}

#[test]
fn test_capture_win_max() {
    assert_eq!(WinChecker::check_capture_win(5, 0), Some(Player::Max));
    assert_eq!(WinChecker::check_capture_win(10, 0), Some(Player::Max));
}

#[test]
fn test_capture_win_min() {
    assert_eq!(WinChecker::check_capture_win(0, 5), Some(Player::Min));
    assert_eq!(WinChecker::check_capture_win(0, 10), Some(Player::Min));
}

#[test]
fn test_no_capture_win() {
    assert_eq!(WinChecker::check_capture_win(4, 4), None);
    assert_eq!(WinChecker::check_capture_win(0, 0), None);
    assert_eq!(WinChecker::check_capture_win(3, 2), None);
}

#[test]
fn test_edge_case_wins() {
    initialize_zobrist();
    let mut board = Board::new(19);

    // Create a 5-in-a-row at the edge (vertical)
    for i in 0..5 {
        board.place_stone(i, 0, Player::Max);
    }

    // Check win detection - should work for any of the 5 positions
    let mut win_detected = false;
    for i in 0..5 {
        if WinChecker::check_win_around(&board, i, 0, 5) {
            win_detected = true;
            break;
        }
    }
    assert!(win_detected, "Should detect win at edge");
}

#[test]
fn test_win_different_conditions() {
    initialize_zobrist();
    let mut board = Board::new(19);

    // Test different win conditions
    for i in 0..3 {
        board.place_stone(9, 5 + i, Player::Max);
    }

    // Should win with condition 3
    assert!(WinChecker::check_win_around(&board, 9, 6, 3));

    // Should not win with condition 5
    assert!(!WinChecker::check_win_around(&board, 9, 6, 5));
}
