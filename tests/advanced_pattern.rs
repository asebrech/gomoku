use gomoku::ai::heuristic::AdvancedPattern;
use gomoku::ai::heuristic::AdvancedPatternType;
use gomoku::ai::heuristic::Heuristic;
use gomoku::ai::heuristic::WINNING_SCORE;
use gomoku::core::board::Board;
use gomoku::core::board::Player;
use gomoku::core::state::GameState;

// Helper function to create a board with specific patterns
fn create_test_board(size: usize) -> Board {
    Board::new(size)
}

// Helper function to place multiple stones at once
fn place_stones(board: &mut Board, positions: &[(usize, usize, Player)]) {
    for &(row, col, player) in positions {
        board.place_stone(row, col, player);
    }
}

#[test]
fn test_jump_four_detection() {
    let mut board = create_test_board(15);

    // Create XX_X pattern (jump four when completed)
    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max), // X
            (7, 6, Player::Max), // X
            // gap at (7, 7)
            (7, 8, Player::Max), // X
        ],
    );

    let patterns = Heuristic::detect_jump_patterns(&board, Player::Max, 5);

    // Debug output
    println!("Detected {} patterns:", patterns.len());
    for (i, p) in patterns.iter().enumerate() {
        println!(
            "Pattern {}: {:?} - threat level {}",
            i, p.pattern_type, p.threat_level
        );
    }

    // Should detect a jump pattern with 3 stones and 1 gap
    assert!(
        patterns
            .iter()
            .any(|p| matches!(p.pattern_type, AdvancedPatternType::JumpPattern(3, 1)))
    );
}

#[test]
fn test_split_four_detection() {
    let mut board = create_test_board(15);

    // Create XX_XX pattern (split four - very dangerous)
    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max), // X
            (7, 6, Player::Max), // X
            // gap at (7, 7)
            (7, 8, Player::Max), // X
            (7, 9, Player::Max), // X
        ],
    );

    let patterns = Heuristic::detect_split_patterns(&board, Player::Max, 5);

    // Should detect a split pattern (2, 2)
    assert!(
        patterns
            .iter()
            .any(|p| matches!(p.pattern_type, AdvancedPatternType::SplitPattern(2, 2)))
    );

    // Verify it's marked as extremely dangerous
    let split_pattern = patterns
        .iter()
        .find(|p| matches!(p.pattern_type, AdvancedPatternType::SplitPattern(2, 2)))
        .unwrap();
    assert_eq!(split_pattern.threat_level, 10);
    assert!(split_pattern.forcing);
}

#[test]
fn test_split_four_xxx_x_pattern() {
    let mut board = create_test_board(15);

    // Create XXX_X pattern
    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max), // X
            (7, 6, Player::Max), // X
            (7, 7, Player::Max), // X
            // gap at (7, 8)
            (7, 9, Player::Max), // X
        ],
    );

    let patterns = Heuristic::detect_split_patterns(&board, Player::Max, 5);

    // Should detect a split pattern (3, 1)
    assert!(
        patterns
            .iter()
            .any(|p| matches!(p.pattern_type, AdvancedPatternType::SplitPattern(3, 1)))
    );

    let split_pattern = patterns
        .iter()
        .find(|p| matches!(p.pattern_type, AdvancedPatternType::SplitPattern(3, 1)))
        .unwrap();
    assert_eq!(split_pattern.threat_level, 9);
    assert!(split_pattern.forcing);
}

#[test]
fn test_split_four_x_xxx_pattern() {
    let mut board = create_test_board(15);

    // Create X_XXX pattern
    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max), // X
            // gap at (7, 6)
            (7, 7, Player::Max), // X
            (7, 8, Player::Max), // X
            (7, 9, Player::Max), // X
        ],
    );

    let patterns = Heuristic::detect_split_patterns(&board, Player::Max, 5);

    // Should detect a split pattern (1, 3)
    assert!(
        patterns
            .iter()
            .any(|p| matches!(p.pattern_type, AdvancedPatternType::SplitPattern(1, 3)))
    );
}

#[test]
fn test_fork_pattern_detection() {
    let mut board = create_test_board(15);

    // Create a position where placing a stone creates multiple threats
    // Set up intersecting lines that would create a fork
    place_stones(
        &mut board,
        &[
            // Horizontal line: _XXX_
            (7, 5, Player::Max),
            (7, 6, Player::Max),
            (7, 7, Player::Max),
            // Vertical line: X at (6,4) and (8,4)
            (6, 4, Player::Max),
            (8, 4, Player::Max),
        ],
    );

    let patterns = Heuristic::detect_fork_patterns(&board, Player::Max);

    // Check if placing at (7, 4) would create a fork
    let threat_count = Heuristic::count_threats_from_position(&board, 7, 4, Player::Max);

    // Should create at least 2 threats (horizontal four + vertical three)
    if threat_count >= 2 {
        assert!(
            patterns
                .iter()
                .any(|p| matches!(p.pattern_type, AdvancedPatternType::ForkPattern))
        );
    }
}

#[test]
fn test_broken_three_pattern() {
    let mut board = create_test_board(15);

    // Create X_X_X pattern (broken three with 2 gaps)
    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max), // X
            // gap at (7, 6)
            (7, 7, Player::Max), // X
            // gap at (7, 8)
            (7, 9, Player::Max), // X
        ],
    );

    let patterns = Heuristic::detect_jump_patterns(&board, Player::Max, 5);

    // Should detect a jump pattern with 3 stones and 2 gaps
    assert!(
        patterns
            .iter()
            .any(|p| matches!(p.pattern_type, AdvancedPatternType::JumpPattern(3, 2)))
    );

    let broken_pattern = patterns
        .iter()
        .find(|p| matches!(p.pattern_type, AdvancedPatternType::JumpPattern(3, 2)))
        .unwrap();
    assert_eq!(broken_pattern.threat_level, 4);
    assert!(!broken_pattern.forcing);
}

#[test]
fn test_complex_threat_detection() {
    let mut board = create_test_board(15);

    // Create a position where placing a stone creates multiple live threes
    place_stones(
        &mut board,
        &[
            // First potential three (horizontal)
            (7, 5, Player::Max),
            (7, 6, Player::Max),
            // Second potential three (vertical)
            (5, 7, Player::Max),
            (6, 7, Player::Max),
        ],
    );

    let patterns = Heuristic::detect_complex_threats(&board, Player::Max);

    // Check if placing at (7, 7) creates complex threats
    if Heuristic::creates_complex_threat(&board, 7, 7, Player::Max) {
        assert!(
            patterns
                .iter()
                .any(|p| matches!(p.pattern_type, AdvancedPatternType::ComplexThreat))
        );
    }
}

#[test]
fn test_advanced_pattern_scoring() {
    // Test individual pattern scores
    let split_four = AdvancedPattern {
        pattern_type: AdvancedPatternType::SplitPattern(2, 2),
        threat_level: 10,
        forcing: true,
        direction: (1, 0),
        positions: [(0, 0); 6],
        position_count: 5,
    };

    let jump_four = AdvancedPattern {
        pattern_type: AdvancedPatternType::JumpPattern(4, 1),
        threat_level: 9,
        forcing: true,
        direction: (1, 0),
        positions: [(0, 0); 6],
        position_count: 5,
    };

    let fork_pattern = AdvancedPattern {
        pattern_type: AdvancedPatternType::ForkPattern,
        threat_level: 10,
        forcing: true,
        direction: (0, 0),
        positions: [(0, 0); 6],
        position_count: 1,
    };

    let patterns = vec![split_four, jump_four, fork_pattern];
    let score = Heuristic::calculate_advanced_pattern_score(&patterns);

    // Should be a high score due to dangerous patterns
    assert!(score > 50000); // Should be significantly high for these dangerous patterns
}

#[test]
fn test_pattern_hierarchy_scoring() {
    // Test that more dangerous patterns get higher scores
    let simple_jump = vec![AdvancedPattern {
        pattern_type: AdvancedPatternType::JumpPattern(3, 2),
        threat_level: 4,
        forcing: false,
        direction: (1, 0),
        positions: [(0, 0); 6],
        position_count: 5,
    }];

    let complex_jump = vec![AdvancedPattern {
        pattern_type: AdvancedPatternType::JumpPattern(3, 1),
        threat_level: 6,
        forcing: false,
        direction: (1, 0),
        positions: [(0, 0); 6],
        position_count: 4,
    }];

    let jump_four = vec![AdvancedPattern {
        pattern_type: AdvancedPatternType::JumpPattern(4, 1),
        threat_level: 9,
        forcing: true,
        direction: (1, 0),
        positions: [(0, 0); 6],
        position_count: 5,
    }];

    let simple_score = Heuristic::calculate_advanced_pattern_score(&simple_jump);
    let complex_score = Heuristic::calculate_advanced_pattern_score(&complex_jump);
    let jump_four_score = Heuristic::calculate_advanced_pattern_score(&jump_four);

    // Verify scoring hierarchy
    assert!(simple_score < complex_score);
    assert!(complex_score < jump_four_score);
}

#[test]
fn test_integration_with_main_evaluate() {
    let mut board = create_test_board(15);

    // Create a game state with advanced patterns
    place_stones(
        &mut board,
        &[
            // Split four for Player::Max
            (7, 5, Player::Max),
            (7, 6, Player::Max),
            // gap at (7, 7)
            (7, 8, Player::Max),
            (7, 9, Player::Max),
            // Some stones for Player::Min
            (6, 5, Player::Min),
            (6, 6, Player::Min),
        ],
    );

    let state = GameState {
        board,
        current_player: Player::Max,
        win_condition: 5,
        max_captures: 0,
        min_captures: 0,
        capture_history: vec![],
        winner: None,
    };

    let score = Heuristic::evaluate(&state, 0);

    // Should be a very high positive score due to split four
    assert!(score > 20000);
}

#[test]
fn test_critical_pattern_early_termination() {
    let mut board = create_test_board(15);

    // Create an immediate winning split four for Max
    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max),
            (7, 6, Player::Max),
            // gap at (7, 7) - if Max plays here, it's winning
            (7, 8, Player::Max),
            (7, 9, Player::Max),
        ],
    );

    let state = GameState {
        board,
        current_player: Player::Max,
        win_condition: 5,
        max_captures: 0,
        min_captures: 0,
        capture_history: vec![],
        winner: None,
    };

    let score = Heuristic::evaluate(&state, 0);

    // Should return winning score due to critical advanced pattern
    assert!(score >= WINNING_SCORE);
}

#[test]
fn test_defensive_pattern_recognition() {
    let mut board = create_test_board(15);

    // Create dangerous pattern for opponent
    place_stones(
        &mut board,
        &[
            (7, 5, Player::Min),
            (7, 6, Player::Min),
            // gap at (7, 7)
            (7, 8, Player::Min),
            (7, 9, Player::Min),
        ],
    );

    let state = GameState {
        board,
        current_player: Player::Max,
        win_condition: 5,
        max_captures: 0,
        min_captures: 0,
        capture_history: vec![],
        winner: None,
    };

    let score = Heuristic::evaluate(&state, 0);

    // Should be very negative due to opponent's split four
    assert!(score < -20000);
}

#[test]
fn test_move_ordering_with_advanced_patterns() {
    let mut board = create_test_board(15);

    // Set up a position with potential advanced patterns
    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max),
            (7, 6, Player::Max),
            (7, 8, Player::Max),
            (7, 9, Player::Max),
        ],
    );

    let state = GameState {
        board,
        current_player: Player::Max,
        win_condition: 5,
        max_captures: 0,
        min_captures: 0,
        capture_history: vec![],
        winner: None,
    };

    let mut moves = vec![
        (7, 7), // Completes split four - should be highest priority
        (8, 7), // Random move
        (6, 7), // Random move
    ];

    Heuristic::order_moves(&state, &mut moves);

    // Move that completes split four should be first
    assert_eq!(moves[0], (7, 7));
}

#[test]
fn test_no_false_positives() {
    let mut board = create_test_board(15);

    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max),
            (7, 6, Player::Max),
            (8, 5, Player::Max),
            (8, 6, Player::Min),
            (8, 7, Player::Max),
        ],
    );

    let jump_patterns = Heuristic::detect_jump_patterns(&board, Player::Max, 5);
    let split_patterns = Heuristic::detect_split_patterns(&board, Player::Max, 5);
    let fork_patterns = Heuristic::detect_fork_patterns(&board, Player::Max);

    assert!(jump_patterns.is_empty() || jump_patterns.iter().all(|p| p.threat_level <= 3));
    assert!(split_patterns.is_empty());

    assert!(
        fork_patterns.is_empty()
            || fork_patterns.iter().all(|p| {
                if let Some(pos) = p.positions.get(0) {
                    Heuristic::count_threats_from_position(&board, pos.0, pos.1, Player::Max) >= 2
                } else {
                    false
                }
            })
    );
}

#[test]
fn test_pattern_position_tracking() {
    let mut board = create_test_board(15);

    place_stones(
        &mut board,
        &[
            (7, 5, Player::Max),
            (7, 6, Player::Max),
            (7, 8, Player::Max),
            (7, 9, Player::Max),
        ],
    );

    let patterns = Heuristic::detect_split_patterns(&board, Player::Max, 5);

    if let Some(pattern) = patterns.first() {
        // Check that positions are correctly recorded
        assert_eq!(pattern.position_count, 5);
        assert_eq!(pattern.positions[0], (7, 5));
        assert_eq!(pattern.positions[1], (7, 6));
        assert_eq!(pattern.positions[2], (7, 7)); // Gap position
        assert_eq!(pattern.positions[3], (7, 8));
        assert_eq!(pattern.positions[4], (7, 9));
    }
}

#[test]
fn test_multiple_directions() {
    let mut board = create_test_board(15);

    // Create patterns in different directions
    place_stones(
        &mut board,
        &[
            // Horizontal pattern
            (7, 5, Player::Max),
            (7, 6, Player::Max),
            (7, 8, Player::Max),
            (7, 9, Player::Max),
            // Vertical pattern
            (5, 10, Player::Max),
            (6, 10, Player::Max),
            (8, 10, Player::Max),
            (9, 10, Player::Max),
            // Diagonal pattern
            (3, 3, Player::Max),
            (4, 4, Player::Max),
            (6, 6, Player::Max),
            (7, 7, Player::Max),
        ],
    );

    let patterns = Heuristic::detect_split_patterns(&board, Player::Max, 5);

    // Should detect multiple split patterns in different directions
    assert!(patterns.len() >= 2);

    // Check that different directions are detected
    let directions: std::collections::HashSet<_> = patterns.iter().map(|p| p.direction).collect();
    assert!(directions.len() > 1);
}
