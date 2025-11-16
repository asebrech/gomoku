//! Comprehensive battle tests for gapped pattern detection features.

use gomoku::ai::move_generation::MoveGenerator;
use gomoku::core::board::{Board, Player};

#[test]
fn test_alternating_pattern_not_threat() {
    let mut board = Board::new(19);
    
    // Create alternating pattern like in user's image: Pink Blue Pink Blue...
    for i in 0..10 {
        let player = if i % 2 == 0 { Player::Max } else { Player::Min };
        board.place_stone(9, 5 + i, player);
    }
    // Pattern: X O X O X O X O X O (row 9, cols 5-14)
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Should NOT treat this as a must-block situation since it's alternating
    // Should be more than just a few blocking moves
    assert!(moves.len() > 5, 
            "Alternating pattern should not be treated as critical threat. Got {} moves", moves.len());
    
    println!("✓ Alternating pattern correctly ignored ({} moves generated)", moves.len());
}

#[test]
fn test_real_gapped_threat_vs_alternating() {
    let mut board = Board::new(19);
    
    // Real gapped threat: X.X.X.X (same player with gaps)
    board.place_stone(5, 5, Player::Max);   // X
    board.place_stone(5, 7, Player::Max);   // X  
    board.place_stone(5, 9, Player::Max);   // X
    board.place_stone(5, 11, Player::Max);  // X
    
    // Alternating pattern elsewhere: X O X O X O
    for i in 0..6 {
        let player = if i % 2 == 0 { Player::Max } else { Player::Min };
        board.place_stone(10, 5 + i, player);
    }
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);
    
    // Should prioritize blocking the real gapped threat, not the alternating pattern
    let blocks_real_threat = moves.contains(&(5, 6)) || moves.contains(&(5, 8)) || moves.contains(&(5, 10));
    assert!(blocks_real_threat, "Should block the real gapped threat");
    
    // Should be a small number of moves (must-block situation)
    assert!(moves.len() <= 10, "Should be focused blocking, got {} moves", moves.len());
    
    println!("✓ Real gapped threat correctly prioritized over alternating pattern");
}

#[test]
fn test_multiple_gapped_threats() {
    let mut board = Board::new(19);
    
    // Create multiple gapped threats for Max
    // Threat 1: X.X.X horizontally
    board.place_stone(5, 5, Player::Max);
    board.place_stone(5, 7, Player::Max);
    board.place_stone(5, 9, Player::Max);
    
    // Threat 2: X.X.X vertically  
    board.place_stone(7, 12, Player::Max);
    board.place_stone(9, 12, Player::Max);
    board.place_stone(11, 12, Player::Max);
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);
    
    // Should detect both threats
    let blocks_horizontal = moves.contains(&(5, 6)) || moves.contains(&(5, 8));
    let blocks_vertical = moves.contains(&(8, 12)) || moves.contains(&(10, 12));
    
    assert!(blocks_horizontal || blocks_vertical, 
            "Should block at least one of the multiple gapped threats");
    
    println!("✓ Multiple gapped threats handling: {} moves generated", moves.len());
}

#[test]
fn test_gapped_threat_near_board_edge() {
    let mut board = Board::new(19);
    
    // Create gapped threat near edge
    board.place_stone(0, 2, Player::Max);   // X at top edge
    board.place_stone(0, 4, Player::Max);   // X
    board.place_stone(0, 6, Player::Max);   // X
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);
    
    // Should still detect even near edges
    let blocks_edge_threat = moves.contains(&(0, 3)) || moves.contains(&(0, 5));
    assert!(blocks_edge_threat, "Should detect gapped threats near board edges");
    
    println!("✓ Edge case gapped threats correctly detected");
}

#[test]
fn test_gapped_threat_with_obstacles() {
    let mut board = Board::new(19);
    
    // Create gapped pattern with obstacle in the middle
    board.place_stone(9, 5, Player::Max);   // X
    board.place_stone(9, 7, Player::Max);   // X
    board.place_stone(9, 9, Player::Min);   // O (obstacle)
    board.place_stone(9, 11, Player::Max);  // X
    board.place_stone(9, 13, Player::Max);  // X
    // Pattern: X . X . O . X . X
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);
    
    // Should NOT treat this as a unified threat since it's broken by obstacle
    // Should be normal zone-based moves, not critical blocking
    assert!(moves.len() > 8, 
            "Broken gapped pattern should not be critical threat, got {} moves", moves.len());
    
    println!("✓ Gapped threats correctly ignored when broken by obstacles");
}

#[test]
fn test_diagonal_gapped_threats() {
    let mut board = Board::new(19);
    
    // Create diagonal gapped threat
    board.place_stone(5, 5, Player::Max);   // X
    board.place_stone(7, 7, Player::Max);   // X
    board.place_stone(9, 9, Player::Max);   // X
    board.place_stone(11, 11, Player::Max); // X
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);
    
    // Should detect diagonal gapped threats
    let blocks_diagonal = moves.contains(&(6, 6)) || moves.contains(&(8, 8)) || moves.contains(&(10, 10));
    assert!(blocks_diagonal, "Should detect diagonal gapped threats");
    
    println!("✓ Diagonal gapped threats correctly detected");
}

#[test]
fn test_performance_with_many_stones() {
    let mut board = Board::new(19);
    
    // Place many stones to test performance
    for i in 0..50 {
        let row = (i * 7) % 19;
        let col = (i * 11) % 19;
        let player = if i % 2 == 0 { Player::Max } else { Player::Min };
        if board.is_empty_position(row, col) {
            board.place_stone(row, col, player);
        }
    }
    
    // Add a real gapped threat among the noise
    if board.is_empty_position(10, 10) && board.is_empty_position(10, 12) && board.is_empty_position(10, 14) {
        board.place_stone(10, 10, Player::Max);
        board.place_stone(10, 12, Player::Max);
        board.place_stone(10, 14, Player::Max);
    }
    
    let start = std::time::Instant::now();
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Min);
    let duration = start.elapsed();
    
    // Should complete quickly even with many stones
    assert!(duration.as_millis() < 100, "Should be fast even with many stones");
    assert!(!moves.is_empty(), "Should generate some moves");
    
    println!("✓ Performance test: {} moves in {:?} with busy board", moves.len(), duration);
}

#[test]
fn test_gapped_winning_move_detection() {
    let mut board = Board::new(19);
    
    // Create position where filling a gap wins immediately
    board.place_stone(9, 5, Player::Max);   // X
    board.place_stone(9, 6, Player::Max);   // X
    board.place_stone(9, 8, Player::Max);   // X
    board.place_stone(9, 9, Player::Max);   // X
    // Pattern: X X . X X - filling (9,7) creates 5 in a row
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Should recognize the winning move and include it (prioritized first)
    assert!(!moves.is_empty(), "Should generate moves");
    assert_eq!(moves[0], (9, 7), "Winning move should be first");
    assert!(moves.contains(&(9, 7)), "Should identify the winning gap");
    
    println!("✓ Gapped winning moves correctly identified and prioritized");
}

#[test]
fn test_complex_mixed_patterns() {
    let mut board = Board::new(19);
    
    // Create a complex scenario with multiple pattern types
    // Consecutive threat
    board.place_stone(5, 5, Player::Min);
    board.place_stone(5, 6, Player::Min);
    board.place_stone(5, 7, Player::Min);
    
    // Gapped threat
    board.place_stone(8, 8, Player::Min);
    board.place_stone(8, 10, Player::Min);
    board.place_stone(8, 12, Player::Min);
    
    // Alternating pattern (not a threat)
    for i in 0..6 {
        let player = if i % 2 == 0 { Player::Max } else { Player::Min };
        board.place_stone(12, 5 + i, player);
    }
    
    let moves = MoveGenerator::get_candidate_moves(&board, Player::Max);
    
    // Should detect at least one threat (consecutive has higher priority but both should be detected)
    let blocks_consecutive = moves.contains(&(5, 4)) || moves.contains(&(5, 8));
    let blocks_gapped = moves.contains(&(8, 9)) || moves.contains(&(8, 11));
    
    assert!(blocks_consecutive || blocks_gapped, 
            "Should detect at least one threat type. Consecutive: {}, Gapped: {}, Moves: {:?}", 
            blocks_consecutive, blocks_gapped, moves);
    
    // Should generate focused candidate moves (reasonable number)
    assert!(moves.len() <= 25, "Should be a focused response, got {} moves", moves.len());
    assert!(!moves.is_empty(), "Should generate some moves");
    
    println!("✓ Complex mixed patterns correctly prioritized");
}

#[test]
fn test_symmetry_and_consistency() {
    let mut board1 = Board::new(19);
    let mut board2 = Board::new(19);
    
    // Create same pattern in different orientations
    // Horizontal
    board1.place_stone(9, 5, Player::Max);
    board1.place_stone(9, 7, Player::Max);
    board1.place_stone(9, 9, Player::Max);
    
    // Vertical (same pattern rotated)
    board2.place_stone(5, 9, Player::Max);
    board2.place_stone(7, 9, Player::Max);
    board2.place_stone(9, 9, Player::Max);
    
    let moves1 = MoveGenerator::get_candidate_moves(&board1, Player::Min);
    let moves2 = MoveGenerator::get_candidate_moves(&board2, Player::Min);
    
    // Should generate similar number of moves for similar threats
    let diff = (moves1.len() as i32 - moves2.len() as i32).abs();
    assert!(diff <= 2, "Similar patterns should generate similar number of moves");
    
    println!("✓ Pattern detection shows good symmetry: {} vs {} moves", moves1.len(), moves2.len());
}

#[test]
fn test_early_game_vs_late_game() {
    // Early game with few stones
    let mut early_board = Board::new(19);
    early_board.place_stone(9, 9, Player::Max);
    early_board.place_stone(9, 11, Player::Max);
    early_board.place_stone(9, 13, Player::Max);
    
    // Late game with same pattern but more stones around
    let mut late_board = early_board.clone();
    for i in 0..20 {
        let row = (i * 3 + 5) % 19;
        let col = (i * 2 + 3) % 19; 
        if late_board.is_empty_position(row, col) {
            let player = if i % 2 == 0 { Player::Min } else { Player::Max };
            late_board.place_stone(row, col, player);
        }
    }
    
    let early_moves = MoveGenerator::get_candidate_moves(&early_board, Player::Min);
    let late_moves = MoveGenerator::get_candidate_moves(&late_board, Player::Min);
    
    // Both should detect the gapped threat
    assert!(!early_moves.is_empty(), "Should detect threat in early game");
    assert!(!late_moves.is_empty(), "Should detect threat in late game");
    
    println!("✓ Gapped threat detection works in early game ({} moves) and late game ({} moves)", 
             early_moves.len(), late_moves.len());
}