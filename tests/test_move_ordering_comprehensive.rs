use gomoku::core::board::{Board, Player};
use gomoku::ai::move_ordering::MoveGenerator;
use gomoku::core::rules::DoubleThreeDetection;

#[test]
fn test_empty_board_returns_center() {
    let board = Board::new(19);
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0], (9, 9));
}

#[test]
fn test_move_ordering_prioritizes_live_four() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 10, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    board.place_stone( 9, 12, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves[0] == (9, 8) || moves[0] == (9, 13));
}

#[test]
fn test_move_ordering_defensive_blocks_opponent_live_four() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Min);
    board.place_stone( 9, 10, Player::Min);
    board.place_stone( 9, 11, Player::Min);
    board.place_stone( 9, 12, Player::Min);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves[0] == (9, 8) || moves[0] == (9, 13));
}

#[test]
fn test_move_ordering_prioritizes_live_three() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 10, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    let top_move = moves[0];
    assert!(top_move == (9, 8) || top_move == (9, 12));
}

#[test]
fn test_move_ordering_includes_gapped_threats() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    board.place_stone( 9, 13, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(9, 10)));
    assert!(moves.contains(&(9, 12)));
}

#[test]
fn test_move_ordering_capture_bonus() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 10, Player::Min);
    board.place_stone( 9, 11, Player::Min);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(9, 12)));
}

#[test]
fn test_move_ordering_filters_double_three() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    board.place_stone( 11, 10, Player::Max);
    board.place_stone( 13, 10, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    for (row, col) in moves {
        assert!(!DoubleThreeDetection::creates_double_three(&board, row, col, Player::Max));
    }
}

#[test]
fn test_move_ordering_zone_fallback_when_no_threats() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    for (row, col) in &moves {
        let distance = ((*row as isize - 9).abs().max((*col as isize - 9).abs())) as usize;
        assert!(distance <= 2);
    }
}

#[test]
fn test_move_ordering_handles_multiple_patterns_same_position() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 10, Player::Max);
    board.place_stone( 10, 9, Player::Max);
    board.place_stone( 11, 9, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(9, 11)) || moves.contains(&(12, 9)));
}

#[test]
fn test_move_ordering_half_free_patterns() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Min);
    board.place_stone( 9, 10, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    board.place_stone( 9, 12, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(9, 13)));
}

#[test]
fn test_move_ordering_flanked_patterns_ignored() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 8, Player::Min);
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 10, Player::Max);
    board.place_stone( 9, 11, Player::Min);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
}

#[test]
fn test_move_ordering_all_directions() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 10, 9, Player::Max);
    board.place_stone( 11, 9, Player::Max);
    
    board.place_stone( 9, 10, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    board.place_stone( 9, 12, Player::Max);
    
    board.place_stone( 10, 10, Player::Max);
    board.place_stone( 11, 11, Player::Max);
    board.place_stone( 12, 12, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(12, 9)) || moves.contains(&(9, 13)) || moves.contains(&(13, 13)));
}

#[test]
fn test_move_ordering_edge_cases_near_border() {
    let mut board = Board::new(19);
    
    board.place_stone( 0, 0, Player::Max);
    board.place_stone( 0, 1, Player::Max);
    board.place_stone( 0, 2, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
}

#[test]
fn test_move_ordering_almost_full_board() {
    let mut board = Board::new(19);
    
    for row in 0..19 {
        for col in 0..19 {
            if row == 9 && col == 9 {
                continue;
            }
            board.place_stone( row, col, if (row + col) % 2 == 0 { Player::Max } else { Player::Min });
        }
    }
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
}

#[test]
fn test_move_ordering_complex_gapped_pattern() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 8, Player::Max);
    board.place_stone( 9, 10, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    board.place_stone( 9, 13, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(9, 9)) || moves.contains(&(9, 12)));
}

#[test]
fn test_move_ordering_multiple_gapped_patterns() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    board.place_stone( 9, 13, Player::Max);
    
    board.place_stone( 10, 9, Player::Max);
    board.place_stone( 12, 9, Player::Max);
    board.place_stone( 14, 9, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(9, 10)) || moves.contains(&(11, 9)));
}

#[test]
fn test_move_ordering_offensive_defensive_balance() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 9, 10, Player::Max);
    board.place_stone( 9, 11, Player::Max);
    
    board.place_stone( 10, 9, Player::Min);
    board.place_stone( 10, 10, Player::Min);
    board.place_stone( 10, 11, Player::Min);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    let top_moves: Vec<_> = moves.iter().take(4).collect();
    assert!(top_moves.contains(&&(9, 8)) || top_moves.contains(&&(9, 12)) || 
            top_moves.contains(&&(10, 8)) || top_moves.contains(&&(10, 12)));
}

#[test]
fn test_move_ordering_diagonal_threats() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    board.place_stone( 10, 10, Player::Max);
    board.place_stone( 11, 11, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(8, 8)) || moves.contains(&(12, 12)));
}

#[test]
fn test_move_ordering_anti_diagonal_threats() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 11, Player::Max);
    board.place_stone( 10, 10, Player::Max);
    board.place_stone( 11, 9, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
    assert!(moves.contains(&(8, 12)) || moves.contains(&(12, 8)));
}

#[test]
fn test_move_ordering_no_legal_moves_returns_something() {
    let mut board = Board::new(19);
    
    board.place_stone( 9, 9, Player::Max);
    
    let moves = MoveGenerator::order_moves(&board, Player::Max);
    
    assert!(!moves.is_empty());
}
