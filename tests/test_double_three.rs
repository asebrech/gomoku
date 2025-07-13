use gomoku::core::board::{Board, Player};
use gomoku::core::moves::{MoveHandler, RuleValidator};
use gomoku::core::state::GameState;

// Schema:
// Horizontal: (row 9)
//  . . . . . . . X X _ X . . . . . . .
//  (col 7)      (col 8) (col 9) (col 10)
// Vertical: (col 9)
//  . . . . . . .
//  X           (row 7)
//  X           (row 8)
//  _           (row 9)
//  X           (row 10)
// Placing at (9,9) creates double three (horizontal & vertical)
// TODO :
// #[test]
// fn test_double_three_detection() {
//     let mut board = Board::new(19);
//
//     // Set up a scenario where placing at (9,9) would create double three
//     // Pattern 1: horizontal three
//     board.place_stone(9, 7, Player::Max);
//     board.place_stone(9, 8, Player::Max);
//     // (9, 9) would complete horizontal three
//     board.place_stone(9, 10, Player::Max);
//
//     // Pattern 2: vertical three
//     board.place_stone(7, 9, Player::Max);
//     board.place_stone(8, 9, Player::Max);
//     // (9, 9) would complete vertical three
//     board.place_stone(10, 9, Player::Max);
//
//     // This should detect double three
//     assert!(RuleValidator::creates_double_three(
//         &board,
//         9,
//         9,
//         Player::Max
//     ));
// }

// Schema:
// Horizontal: (row 9)
//  . . . . . . . _ X X X _ . . . . . .
//                (7)(8)(9)(10)(11)
// Free three pattern (_ X X X _), tests both ends
// TODO :
// #[test]
// fn test_free_three_detection() {
//     let mut board = Board::new(19);
//
//     // Set up a free three pattern: _ X X X _
//     board.place_stone(9, 8, Player::Max);
//     board.place_stone(9, 9, Player::Max);
//     board.place_stone(9, 10, Player::Max);
//
//     // Check if placing at (9, 7) or (9, 11) would create a free three
//     assert!(RuleValidator::is_free_three(
//         &board,
//         9,
//         7,
//         Player::Max,
//         (0, 1)
//     ));
//     assert!(RuleValidator::is_free_three(
//         &board,
//         9,
//         11,
//         Player::Max,
//         (0, 1)
//     ));
// }

// TODO :
// #[test]
// fn test_double_three_prevention() {
//     let mut state = GameState::new(19, 5);
//
//     // Set up potential double three
//     state.board.place_stone(9, 7, Player::Max);
//     state.board.place_stone(9, 8, Player::Max);
//     state.board.place_stone(9, 10, Player::Max);
//     state.board.place_stone(7, 9, Player::Max);
//     state.board.place_stone(8, 9, Player::Max);
//     state.board.place_stone(10, 9, Player::Max);
//     state.current_player = Player::Max;
//
//     // Get possible moves
//     let moves = state.get_possible_moves();
//
//     // Should not include the double three move
//     assert!(!moves.contains(&(9, 9)));
// }

// Schema:
// Horizontal: (row 9)
//  . . . . . . . X X _ X . . . . . . .
//                (7)(8)(9)(10)
// Vertical: (col 9)
//   X (7)
//   X (8)
//   _ (9)
//   X (10)
// MoveHandler should not propose (9, 9) due to double three
// TODO :
// #[test]
// fn test_moves_exclude_double_three() {
//     let mut board = Board::new(19);
//
//     // Set up potential double three scenario
//     board.place_stone(9, 7, Player::Max);
//     board.place_stone(9, 8, Player::Max);
//     board.place_stone(9, 10, Player::Max);
//     board.place_stone(7, 9, Player::Max);
//     board.place_stone(8, 9, Player::Max);
//     board.place_stone(10, 9, Player::Max);
//
//     let moves = MoveHandler::get_possible_moves(&board, Player::Max);
//
//     // Should not include the double three move
//     assert!(!moves.contains(&(9, 9)));
// }

// Schema:
// Horizontal: (row 9)
//  . . . . . . . X X _ X . . . . . . .
//                (col 7)(col 8)(col 9)(col 10)
// Only one horizontal three, no vertical pattern
#[test]
fn test_no_double_three_single_pattern() {
    let mut board = Board::new(19);

    // Set up only one three pattern
    board.place_stone(9, 7, Player::Max);
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);

    // This should not detect double three (only one pattern)
    assert!(!RuleValidator::creates_double_three(
        &board,
        9,
        9,
        Player::Max
    ));
}

// Schema:
// Horizontal: (row 9)
//  . . . . . . . O X X X _ . . . . . .
//                (7)(8)(9)(10)(11)
// Blocked at left end (O), should not be free three
#[test]
fn test_blocked_three_not_free() {
    let mut board = Board::new(19);

    // Set up a blocked three pattern: O X X X _
    board.place_stone(9, 7, Player::Min); // Blocking stone
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);

    // This should not be detected as a free three
    assert!(!RuleValidator::is_free_three(
        &board,
        9,
        11,
        Player::Max,
        (0, 1)
    ));
}

// Schema:
// Intersecting stones bottom right:
// (rows 9,10), (cols 9,10)
//  . . . . . . X X
//  . . . . . . X X
//   (9, 9)(9,10)
//   (10,9)(10,10)
// Tests multiple placements, ensures no move creates double three
#[test]
fn test_complex_double_three_scenario() {
    let mut board = Board::new(19);

    // Create a more complex board state
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(10, 9, Player::Max);
    board.place_stone(10, 10, Player::Max);

    // Test various positions for double three
    let moves = MoveHandler::get_possible_moves(&board, Player::Max);

    // Should include valid moves that don't create double three
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
