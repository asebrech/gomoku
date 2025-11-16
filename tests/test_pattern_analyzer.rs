use gomoku::core::board::{Board, Player};
use gomoku::core::patterns::{PatternAnalyzer, PatternFreedom};

#[test]
fn test_count_consecutive_horizontal() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    
    let count = PatternAnalyzer::count_consecutive(&board, 9, 10, 0, 1, Player::Max);
    assert_eq!(count, 1);
    
    let count_back = PatternAnalyzer::count_consecutive(&board, 9, 10, 0, -1, Player::Max);
    assert_eq!(count_back, 1);
}

#[test]
fn test_count_consecutive_vertical() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(10, 9, Player::Max);
    board.place_stone(11, 9, Player::Max);
    board.place_stone(12, 9, Player::Max);
    
    let count = PatternAnalyzer::count_consecutive(&board, 10, 9, 1, 0, Player::Max);
    assert_eq!(count, 2);
}

#[test]
fn test_count_consecutive_diagonal() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(10, 10, Player::Max);
    board.place_stone(11, 11, Player::Max);
    
    let count = PatternAnalyzer::count_consecutive(&board, 10, 10, 1, 1, Player::Max);
    assert_eq!(count, 1);
    
    let count_back = PatternAnalyzer::count_consecutive(&board, 10, 10, -1, -1, Player::Max);
    assert_eq!(count_back, 1);
}

#[test]
fn test_count_consecutive_stops_at_opponent() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Min);
    board.place_stone(9, 12, Player::Max);
    
    let count = PatternAnalyzer::count_consecutive(&board, 9, 9, 0, 1, Player::Max);
    assert_eq!(count, 1);
}

#[test]
fn test_count_consecutive_stops_at_empty() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 12, Player::Max);
    
    let count = PatternAnalyzer::count_consecutive(&board, 9, 9, 0, 1, Player::Max);
    assert_eq!(count, 1);
}

#[test]
fn test_is_in_bounds() {
    let board = Board::new(19);
    
    assert!(PatternAnalyzer::is_in_bounds(&board, 0, 0));
    assert!(PatternAnalyzer::is_in_bounds(&board, 18, 18));
    assert!(PatternAnalyzer::is_in_bounds(&board, 9, 9));
    
    assert!(!PatternAnalyzer::is_in_bounds(&board, -1, 0));
    assert!(!PatternAnalyzer::is_in_bounds(&board, 0, -1));
    assert!(!PatternAnalyzer::is_in_bounds(&board, 19, 0));
    assert!(!PatternAnalyzer::is_in_bounds(&board, 0, 19));
}

#[test]
fn test_is_valid_empty() {
    let mut board = Board::new(19);
    
    assert!(PatternAnalyzer::is_valid_empty(&board, 9, 9));
    
    board.place_stone(9, 9, Player::Max);
    assert!(!PatternAnalyzer::is_valid_empty(&board, 9, 9));
    
    assert!(!PatternAnalyzer::is_valid_empty(&board, -1, 0));
    assert!(!PatternAnalyzer::is_valid_empty(&board, 0, -1));
    assert!(!PatternAnalyzer::is_valid_empty(&board, 19, 0));
}

#[test]
fn test_count_consecutive_bidirectional() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    
    let count = PatternAnalyzer::count_consecutive_bidirectional(&board, 9, 10, 0, 1, Player::Max);
    assert_eq!(count, 3);
}

#[test]
fn test_count_consecutive_bidirectional_asymmetric() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 12, Player::Max);
    
    let count = PatternAnalyzer::count_consecutive_bidirectional(&board, 9, 10, 0, 1, Player::Max);
    assert_eq!(count, 5);
}

#[test]
fn test_count_total_space_free_pattern() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    
    let space = PatternAnalyzer::count_total_space(&board, 9, 9, 0, 1, 3);
    
    assert!(space >= 5);
}

#[test]
fn test_count_total_space_blocked() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 8, Player::Min);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 12, Player::Min);
    
    let space = PatternAnalyzer::count_total_space(&board, 9, 9, 0, 1, 3);
    assert_eq!(space, 3);
}

#[test]
fn test_count_empty_in_direction() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 13, Player::Max);
    
    let count = PatternAnalyzer::count_empty_in_direction(&board, 9, 10, 0, 1);
    assert_eq!(count, 3);
}

#[test]
fn test_count_empty_in_direction_stops_at_occupied() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 11, Player::Min);
    
    let count = PatternAnalyzer::count_empty_in_direction(&board, 9, 10, 0, 1);
    assert_eq!(count, 1);
}

#[test]
fn test_analyze_pattern_freedom_free() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    
    let freedom = PatternAnalyzer::analyze_pattern_freedom(&board, 9, 9, 0, 1, 3);
    assert_eq!(freedom, PatternFreedom::Free);
}

#[test]
fn test_analyze_pattern_freedom_half_free() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 8, Player::Min);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    
    let freedom = PatternAnalyzer::analyze_pattern_freedom(&board, 9, 9, 0, 1, 3);
    assert_eq!(freedom, PatternFreedom::HalfFree);
}

#[test]
fn test_analyze_pattern_freedom_flanked() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 8, Player::Min);
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 12, Player::Min);
    
    let freedom = PatternAnalyzer::analyze_pattern_freedom(&board, 9, 9, 0, 1, 3);
    assert_eq!(freedom, PatternFreedom::Flanked);
}

#[test]
fn test_analyze_consecutive_from_position() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    
    let result = PatternAnalyzer::analyze_consecutive_from_position(
        &board,
        9,
        10,
        0,
        1,
        Player::Max,
        5,
    );
    
    assert!(result.is_some());
    let (length, start_row, start_col, space, freedom) = result.unwrap();
    assert_eq!(length, 3);
    assert_eq!(start_row, 9);
    assert_eq!(start_col, 9);
    assert!(space >= 5);
    assert_eq!(freedom, PatternFreedom::Free);
}

#[test]
fn test_analyze_consecutive_from_position_too_short() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    
    let result = PatternAnalyzer::analyze_consecutive_from_position(
        &board,
        9,
        9,
        0,
        1,
        Player::Max,
        5,
    );
    
    assert!(result.is_none());
}

#[test]
fn test_analyze_consecutive_from_position_insufficient_space() {
    let mut board = Board::new(19);
    
    board.place_stone(0, 0, Player::Max);
    board.place_stone(0, 1, Player::Max);
    board.place_stone(0, 2, Player::Max);
    board.place_stone(0, 3, Player::Max);
    board.place_stone(0, 4, Player::Min);
    
    let result = PatternAnalyzer::analyze_consecutive_from_position(
        &board,
        0,
        1,
        0,
        -1,
        Player::Max,
        5,
    );
    
    assert!(result.is_none());
}

#[test]
fn test_collect_gapped_stones_simple() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 13, Player::Max);
    
    let stones = PatternAnalyzer::collect_gapped_stones(&board, 9, 11, 0, 1, Player::Max, 6);
    
    assert_eq!(stones.len(), 3);
    assert!(stones.contains(&(9, 9)));
    assert!(stones.contains(&(9, 11)));
    assert!(stones.contains(&(9, 13)));
}

#[test]
fn test_collect_gapped_stones_bidirectional() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 12, Player::Max);
    
    let stones = PatternAnalyzer::collect_gapped_stones(&board, 9, 10, 0, 1, Player::Max, 6);
    
    assert_eq!(stones.len(), 3);
    assert!(stones.contains(&(9, 8)));
    assert!(stones.contains(&(9, 10)));
    assert!(stones.contains(&(9, 12)));
}

#[test]
fn test_collect_gapped_stones_stops_at_opponent() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 12, Player::Min);
    board.place_stone(9, 13, Player::Max);
    
    let stones = PatternAnalyzer::collect_gapped_stones(&board, 9, 9, 0, 1, Player::Max, 6);
    
    assert_eq!(stones.len(), 2);
    assert!(stones.contains(&(9, 9)));
    assert!(stones.contains(&(9, 11)));
    assert!(!stones.contains(&(9, 13)));
}

#[test]
fn test_collect_gapped_stones_max_distance() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 13, Player::Max);
    board.place_stone(9, 16, Player::Max);
    
    let stones = PatternAnalyzer::collect_gapped_stones(&board, 9, 9, 0, 1, Player::Max, 3);
    
    assert_eq!(stones.len(), 2);
    assert!(stones.contains(&(9, 9)));
    assert!(stones.contains(&(9, 11)));
}

#[test]
fn test_analyze_gapped_pattern_valid() {
    let stones = vec![(9, 9), (9, 11), (9, 13)];
    
    let result = PatternAnalyzer::analyze_gapped_pattern(&stones);
    
    assert!(result.is_some());
    let (stone_count, gaps, span) = result.unwrap();
    assert_eq!(stone_count, 3);
    assert_eq!(gaps, 2);
    assert_eq!(span, 5);
}

#[test]
fn test_analyze_gapped_pattern_too_few_stones() {
    let stones = vec![(9, 9)];
    
    let result = PatternAnalyzer::analyze_gapped_pattern(&stones);
    assert!(result.is_none());
}

#[test]
fn test_analyze_gapped_pattern_no_gaps() {
    let stones = vec![(9, 9), (9, 10), (9, 11)];
    
    let result = PatternAnalyzer::analyze_gapped_pattern(&stones);
    assert!(result.is_none());
}

#[test]
fn test_analyze_gapped_pattern_span_too_large() {
    let stones = vec![(9, 9), (9, 17)];
    
    let result = PatternAnalyzer::analyze_gapped_pattern(&stones);
    assert!(result.is_none());
}

#[test]
fn test_analyze_gapped_pattern_too_many_gaps() {
    let stones = vec![(9, 9), (9, 14)];
    
    let result = PatternAnalyzer::analyze_gapped_pattern(&stones);
    assert!(result.is_none());
}

#[test]
fn test_extract_gap_positions() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 13, Player::Max);
    
    let stones = vec![(9, 9), (9, 11), (9, 13)];
    let gaps = PatternAnalyzer::extract_gap_positions(&board, &stones, 0, 1);
    
    assert_eq!(gaps.len(), 2);
    assert!(gaps.contains(&(9, 10)));
    assert!(gaps.contains(&(9, 12)));
}

#[test]
fn test_extract_gap_positions_occupied_position() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(9, 10, Player::Min);
    board.place_stone(9, 11, Player::Max);
    
    let stones = vec![(9, 9), (9, 11)];
    let gaps = PatternAnalyzer::extract_gap_positions(&board, &stones, 0, 1);
    
    assert_eq!(gaps.len(), 0);
}

#[test]
fn test_extract_gap_positions_diagonal() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(11, 11, Player::Max);
    
    let stones = vec![(9, 9), (11, 11)];
    let gaps = PatternAnalyzer::extract_gap_positions(&board, &stones, 1, 1);
    
    assert_eq!(gaps.len(), 1);
    assert!(gaps.contains(&(10, 10)));
}

#[test]
fn test_pattern_freedom_partial_eq() {
    assert_eq!(PatternFreedom::Free, PatternFreedom::Free);
    assert_eq!(PatternFreedom::HalfFree, PatternFreedom::HalfFree);
    assert_eq!(PatternFreedom::Flanked, PatternFreedom::Flanked);
    
    assert_ne!(PatternFreedom::Free, PatternFreedom::HalfFree);
    assert_ne!(PatternFreedom::Free, PatternFreedom::Flanked);
    assert_ne!(PatternFreedom::HalfFree, PatternFreedom::Flanked);
}

#[test]
fn test_analyze_consecutive_from_position_at_edges() {
    let mut board = Board::new(19);
    
    board.place_stone(0, 0, Player::Max);
    board.place_stone(0, 1, Player::Max);
    board.place_stone(0, 2, Player::Max);
    
    let result = PatternAnalyzer::analyze_consecutive_from_position(
        &board,
        0,
        1,
        0,
        1,
        Player::Max,
        5,
    );
    
    assert!(result.is_some());
}

#[test]
fn test_collect_gapped_stones_diagonal() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    board.place_stone(10, 10, Player::Max);
    board.place_stone(12, 12, Player::Max);
    
    let stones = PatternAnalyzer::collect_gapped_stones(&board, 9, 9, 1, 1, Player::Max, 6);
    
    assert_eq!(stones.len(), 3);
    assert!(stones.contains(&(9, 9)));
    assert!(stones.contains(&(10, 10)));
    assert!(stones.contains(&(12, 12)));
}

#[test]
fn test_complex_gapped_pattern_with_multiple_gaps() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 8, Player::Max);
    board.place_stone(9, 10, Player::Max);
    board.place_stone(9, 11, Player::Max);
    board.place_stone(9, 13, Player::Max);
    
    let stones = PatternAnalyzer::collect_gapped_stones(&board, 9, 8, 0, 1, Player::Max, 6);
    assert!(stones.len() >= 3);
    
    let result = PatternAnalyzer::analyze_gapped_pattern(&stones);
    assert!(result.is_some());
}

#[test]
fn test_count_consecutive_bidirectional_single_stone() {
    let mut board = Board::new(19);
    
    board.place_stone(9, 9, Player::Max);
    
    let count = PatternAnalyzer::count_consecutive_bidirectional(&board, 9, 9, 0, 1, Player::Max);
    assert_eq!(count, 1);
}

#[test]
fn test_analyze_pattern_freedom_at_board_edge() {
    let mut board = Board::new(19);
    
    board.place_stone(0, 0, Player::Max);
    board.place_stone(0, 1, Player::Max);
    board.place_stone(0, 2, Player::Max);
    
    let freedom = PatternAnalyzer::analyze_pattern_freedom(&board, 0, 0, 0, 1, 3);
    assert_eq!(freedom, PatternFreedom::HalfFree);
}
