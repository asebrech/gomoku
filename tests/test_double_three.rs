use gomoku::core::board::{Board, Player};
use gomoku::core::moves::{MoveHandler, RuleValidator};
use gomoku::core::state::GameState;

/// Test that placing at (9,9) creates a double three:
/// two simultaneous free three lines — horizontal and vertical.
///
/// Horizontal (row 9): _ X _ X X _
/// Vertical (col 9):   _ X _ X X _
#[test]
fn test_double_three_detection() {
    let mut board = Board::new(19);

    // Horizontal free three parts (cols 8 and 10)
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);

    // Vertical free three parts (rows 8 and 10)
    board.place_stone(8, 9, Player::Max);
    board.place_stone(10, 9, Player::Max);

    // Placing at (9,9) creates two free threes → double three detected
    assert!(RuleValidator::creates_double_three(
        &board,
        9,
        9,
        Player::Max
    ));
}

/// Test detection of a single free three horizontally:
/// Placing at (9,9) completes pattern _ X X X _
#[test]
fn test_free_three_detection() {
    let mut board = Board::new(19);

    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);

    assert!(RuleValidator::is_free_three(
        &board,
        9,
        9,
        Player::Max,
        (0, 1)
    ));
}

/// Test that a blocked three (blocked by opponent stone) is NOT detected as free three
///
/// Pattern: O X X X _
#[test]
fn test_not_free_three_when_blocked() {
    let mut board = Board::new(19);

    board.place_stone(9, 7, Player::Min); // Blocking stone
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);

    assert!(!RuleValidator::is_free_three(
        &board,
        9,
        9,
        Player::Max,
        (0, 1)
    ));
}

/// Test that possible moves exclude those that create a double three
#[test]
fn test_double_three_prevention() {
    let mut state = GameState::new(19, 5);

    state.board.place_stone(9, 8, Player::Max);
    state.board.place_stone(9, 10, Player::Max);
    state.board.place_stone(8, 9, Player::Max);
    state.board.place_stone(10, 9, Player::Max);

    state.current_player = Player::Max;

    let moves = state.get_possible_moves();

    // Move at (9,9) would create double three → must be excluded
    assert!(!moves.contains(&(9, 9)));
}

/// Test that MoveHandler excludes moves that cause double three
#[test]
fn test_moves_exclude_double_three() {
    let mut board = Board::new(19);

    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(8, 9, Player::Max);
    board.place_stone(10, 9, Player::Max);

    let moves = MoveHandler::get_possible_moves(&board, Player::Max);

    assert!(!moves.contains(&(9, 9)));
}

/// Test that placing a stone creating only one free three is NOT double three
#[test]
fn test_no_double_three_single_pattern() {
    let mut board = Board::new(19);

    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);

    // Only one free three → no double three detected
    assert!(!RuleValidator::creates_double_three(
        &board,
        9,
        9,
        Player::Max
    ));
}

/// Test that a blocked three (by opponent stone) is not detected as free three
#[test]
fn test_blocked_three_not_free() {
    let mut board = Board::new(19);

    board.place_stone(9, 7, Player::Min); // Blocking stone
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);

    assert!(!RuleValidator::is_free_three(
        &board,
        9,
        11,
        Player::Max,
        (0, 1)
    ));
}

/// Test complex scenario where multiple stones are placed;
/// ensures get_possible_moves returns moves that don't create double three
#[test]
fn test_complex_double_three_scenario() {
    let mut board = Board::new(19);

    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(10, 9, Player::Max);
    board.place_stone(10, 10, Player::Max);

    let moves = MoveHandler::get_possible_moves(&board, Player::Max);

    assert!(!moves.is_empty());

    for &(row, col) in &moves {
        assert!(!RuleValidator::creates_double_three(
            &board,
            row,
            col,
            Player::Max
        ));
    }
}