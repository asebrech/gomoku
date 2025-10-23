use gomoku::ai::move_generation::MoveGenerator;
use gomoku::core::board::{Board, Player};
use std::collections::HashSet;

/// Helper function to create a board from a pattern string
/// 'X' = Player Max, 'O' = Player Min, '.' = Empty
fn create_board_from_pattern(size: usize, pattern: &[&str]) -> Board {
    let mut board = Board::new(size);
    for (row, line) in pattern.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            match ch {
                'X' => board.place_stone(row, col, Player::Max),
                'O' => board.place_stone(row, col, Player::Min),
                '.' => {}
                ' ' => {}
                _ => panic!("Invalid character in pattern: {}", ch),
            }
        }
    }
    board
}

/// Helper to convert Vec<(usize, usize)> to HashSet for easy comparison
fn to_set(moves: Vec<(usize, usize)>) -> HashSet<(usize, usize)> {
    moves.into_iter().collect()
}

#[test]
fn test_simple_gapped_threat_three_stones_two_gaps() {
    // Pattern: X.X.X (classic gapped threat)
    let pattern = vec![
        "X.X.X.....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should identify positions (0,1) and (0,3) as threats
    assert!(threat_set.contains(&(0, 1)), "Should detect gap at (0,1)");
    assert!(threat_set.contains(&(0, 3)), "Should detect gap at (0,3)");
    assert_eq!(threat_set.len(), 2, "Should find exactly 2 threat positions");
}

#[test]
fn test_gapped_threat_vertical() {
    // Vertical pattern
    let pattern = vec![
        "O.........",
        "..........",
        "O.........",
        "..........",
        "O.........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Min);
    let threat_set = to_set(threats);
    
    // Should identify positions (1,0) and (3,0) as threats
    assert!(threat_set.contains(&(1, 0)), "Should detect gap at (1,0)");
    assert!(threat_set.contains(&(3, 0)), "Should detect gap at (3,0)");
    assert_eq!(threat_set.len(), 2, "Should find exactly 2 threat positions");
}

#[test]
fn test_gapped_threat_diagonal() {
    // Diagonal pattern: X at (0,0), (2,2), (4,4)
    let pattern = vec![
        "X.........",
        "..........",
        "..X.......",
        "..........",
        "....X.....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should identify (1,1) and (3,3) as threats
    assert!(threat_set.contains(&(1, 1)), "Should detect diagonal gap at (1,1)");
    assert!(threat_set.contains(&(3, 3)), "Should detect diagonal gap at (3,3)");
    assert_eq!(threat_set.len(), 2, "Should find exactly 2 threat positions");
}

#[test]
fn test_gapped_threat_anti_diagonal() {
    // Anti-diagonal pattern
    let pattern = vec![
        "....X.....",
        "..........",
        "..X.......",
        "..........",
        "X.........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should identify (1,3) and (3,1) as threats
    assert!(threat_set.contains(&(1, 3)), "Should detect anti-diagonal gap at (1,3)");
    assert!(threat_set.contains(&(3, 1)), "Should detect anti-diagonal gap at (3,1)");
}

#[test]
fn test_gapped_threat_with_single_gap() {
    // Pattern: XX.X (one gap)
    let pattern = vec![
        "XX.X......",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should identify position (0,2) as threat
    assert!(threat_set.contains(&(0, 2)), "Should detect gap at (0,2)");
    assert_eq!(threat_set.len(), 1, "Should find exactly 1 threat position");
}

#[test]
fn test_gapped_threat_four_stones_one_gap() {
    // Pattern: X.XXX (4 stones with 1 gap - very dangerous)
    let pattern = vec![
        "X.XXX.....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should identify position (0,1) as critical threat
    assert!(threat_set.contains(&(0, 1)), "Should detect critical gap at (0,1)");
}

#[test]
fn test_gapped_threat_blocked_by_opponent() {
    // Pattern: X.X.O (blocked by opponent stone)
    let pattern = vec![
        "X.X.O.....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    
    // Should NOT identify this as a threat (blocked by opponent)
    // The pattern only has 2 stones before hitting opponent
    assert!(threats.is_empty() || !threats.contains(&(0, 1)), 
        "Should not detect threat when blocked by opponent");
}

#[test]
fn test_gapped_threat_too_spread_out() {
    // Pattern: X...X...X (spread over 9 spaces - too far)
    let pattern = vec![
        "X...X...X.",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    
    // Should NOT identify as threat (total span > 5)
    assert!(threats.is_empty(), "Should not detect threat when span exceeds 5");
}

#[test]
fn test_gapped_threat_at_board_edge() {
    // Pattern at edge of board
    let pattern = vec![
        ".........X",
        "..........",
        ".........X",
        "..........",
        ".........X",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should still detect gaps even near edge
    assert!(threat_set.contains(&(1, 9)), "Should detect gap at edge (1,9)");
    assert!(threat_set.contains(&(3, 9)), "Should detect gap at edge (3,9)");
}

#[test]
fn test_multiple_gapped_threats_same_direction() {
    // Multiple gapped patterns in same row
    let pattern = vec![
        "X.X.X..O.O.O",
        "............",
        "............",
        "............",
        "............",
        "............",
        "............",
        "............",
        "............",
        "............",
        "............",
        "............",
    ];
    
    let board = create_board_from_pattern(12, &pattern);
    
    // Test Player1's threats
    let threats_p1 = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set_p1 = to_set(threats_p1);
    assert!(threat_set_p1.contains(&(0, 1)), "Should detect X's gap at (0,1)");
    assert!(threat_set_p1.contains(&(0, 3)), "Should detect X's gap at (0,3)");
    
    // Test Player2's threats
    let threats_p2 = MoveGenerator::find_gapped_threats(&board, Player::Min);
    let threat_set_p2 = to_set(threats_p2);
    assert!(threat_set_p2.contains(&(0, 8)), "Should detect O's gap at (0,8)");
    assert!(threat_set_p2.contains(&(0, 10)), "Should detect O's gap at (0,10)");
}

#[test]
fn test_gapped_threat_three_gaps() {
    // Pattern: X..X..X (three gaps - should NOT be detected, too many gaps)
    let pattern = vec![
        "X..X..X...",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    
    // Should NOT detect (max 2 gaps allowed)
    assert!(threats.is_empty(), "Should not detect threat with more than 2 gaps");
}

#[test]
fn test_gapped_threat_exactly_five_spaces() {
    // Pattern that fits exactly in 5 spaces: X.XX.
    let pattern = vec![
        "X.XX......",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should detect the gap
    assert!(threat_set.contains(&(0, 1)), "Should detect gap when pattern fits in 5");
}

#[test]
fn test_gapped_threat_four_stones_scattered() {
    // Pattern: X.X.XX (4 stones, multiple gaps)
    let pattern = vec![
        "X.X.XX....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should detect gaps at (0,1) and (0,3)
    assert!(threat_set.contains(&(0, 1)), "Should detect gap at (0,1)");
    assert!(threat_set.contains(&(0, 3)), "Should detect gap at (0,3)");
}

#[test]
fn test_no_gapped_threat_only_two_stones() {
    // Pattern: X.X (only 2 stones - not enough)
    let pattern = vec![
        "X.X.......",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    
    // Should NOT detect (need at least 3 stones)
    assert!(threats.is_empty(), "Should not detect threat with only 2 stones");
}

#[test]
fn test_gapped_threat_complex_pattern() {
    // Complex board with multiple potential threats
    let pattern = vec![
        "X.X.X.....",
        "..........",
        "X.........",
        "..........",
        "X.........",
        "..........",
        "X.........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Horizontal threat
    assert!(threat_set.contains(&(0, 1)), "Should detect horizontal gap at (0,1)");
    assert!(threat_set.contains(&(0, 3)), "Should detect horizontal gap at (0,3)");
    
    // Vertical threat
    assert!(threat_set.contains(&(1, 0)), "Should detect vertical gap at (1,0)");
    assert!(threat_set.contains(&(3, 0)), "Should detect vertical gap at (3,0)");
    assert!(threat_set.contains(&(5, 0)), "Should detect vertical gap at (5,0)");
}

#[test]
fn test_gapped_threat_mixed_with_consecutive() {
    // Pattern: XXX.X (mostly consecutive with one gap)
    let pattern = vec![
        "XXX.X.....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should detect the gap at (0,3)
    assert!(threat_set.contains(&(0, 3)), "Should detect gap in mostly consecutive pattern");
}

#[test]
fn test_gapped_threat_corner_diagonal() {
    // Diagonal pattern starting from corner
    let pattern = vec![
        "X.........",
        "..........",
        "..X.......",
        "..........",
        "....X.....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Should detect diagonal gaps
    assert!(threat_set.contains(&(1, 1)), "Should detect corner diagonal gap at (1,1)");
    assert!(threat_set.contains(&(3, 3)), "Should detect corner diagonal gap at (3,3)");
}

#[test]
fn test_gapped_threat_five_stones_one_gap() {
    // Pattern: XX.XX (5 stones total with 1 gap - critical!)
    let pattern = vec![
        "XX.XX.....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // This should be detected - filling gap creates win
    assert!(threat_set.contains(&(0, 2)), "Should detect critical winning gap");
}

#[test]
fn test_gapped_threat_not_aligned() {
    // Stones not in a line - should not detect
    let pattern = vec![
        "X.........",
        ".X........",
        "..X.......",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    
    // These form a diagonal, should detect it
    // Actually this IS a valid diagonal pattern
    // No gaps in consecutive diagonal, so should be empty
    assert!(threats.is_empty(), "Consecutive stones without gaps should not be detected by gapped threat finder");
}

#[test]
fn test_gapped_threat_boundary_conditions() {
    // Test near board boundaries with a valid 3-stone pattern
    let pattern = vec![
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "........X.",
        "..........",
        "........X.",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    
    // Add a third stone to make a valid gapped threat
    let mut board = board;
    board.place_stone(4, 8, Player::Max);
    
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threat_set = to_set(threats);
    
    // Vertical pattern: (4,8), (6,8), (8,8) with gaps at (5,8) and (7,8)
    assert!(threat_set.contains(&(5, 8)) || threat_set.contains(&(7, 8)), 
            "Should detect at least one gap in vertical pattern near boundary");
}

#[test]
fn test_empty_board_no_threats() {
    let board = Board::new(10);
    let threats = MoveGenerator::find_gapped_threats(&board, Player::Max);
    
    assert!(threats.is_empty(), "Empty board should have no threats");
}

#[test]
fn test_gapped_threat_both_players() {
    // Both players have gapped patterns
    let pattern = vec![
        "X.X.X.....",
        "..........",
        "O.O.O.....",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
        "..........",
    ];
    
    let board = create_board_from_pattern(10, &pattern);
    
    let threats_p1 = MoveGenerator::find_gapped_threats(&board, Player::Max);
    let threats_p2 = MoveGenerator::find_gapped_threats(&board, Player::Min);
    
    assert!(!threats_p1.is_empty(), "Player 1 should have threats");
    assert!(!threats_p2.is_empty(), "Player 2 should have threats");
    
    let set_p1 = to_set(threats_p1);
    let set_p2 = to_set(threats_p2);
    
    assert!(set_p1.contains(&(0, 1)));
    assert!(set_p2.contains(&(2, 1)));
}
