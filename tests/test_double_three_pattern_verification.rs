/// Standalone verification: Test that we can actually create double-three patterns
/// This ensures our test setups are valid before testing AI behavior

use gomoku::core::board::{Board, Player};
use gomoku::core::rules::DoubleThreeDetection;

#[test]
fn verify_double_three_detection_works() {
    // Test that double-three detection actually works with known valid patterns
    
    let mut board = Board::new(19);
    
    println!("\n=== Testing Double-Three Pattern Creation ===");
    
    // Pattern from the actual double-three tests that we know works
    board.place_stone(5, 7, Player::Max);
    board.place_stone(5, 9, Player::Max);
    board.place_stone(6, 6, Player::Max);
    board.place_stone(6, 10, Player::Max);
    board.place_stone(7, 7, Player::Max);
    board.place_stone(7, 9, Player::Max);
    
    let creates_dt = DoubleThreeDetection::creates_double_three(&board, 6, 8, Player::Max);
    assert!(creates_dt, "Known working pattern should create double-three");
    println!("✓ Basic cross pattern creates double-three at (6, 8)");
    
    // Test simple horizontal + vertical pattern
    let mut board2 = Board::new(19);
    
    // Horizontal: O O _ with space to extend
    board2.place_stone(10, 8, Player::Min);
    board2.place_stone(10, 9, Player::Min);
    // Position (10, 10) would create: O O O with space at (10, 7) and (10, 11)
    
    // Vertical: O O _ with space to extend
    board2.place_stone(8, 10, Player::Min);
    board2.place_stone(9, 10, Player::Min);
    // Position (10, 10) would create: O O O with space at (7, 10) and (11, 10)
    
    let creates_dt2 = DoubleThreeDetection::creates_double_three(&board2, 10, 10, Player::Min);
    
    println!("\nSimple H+V pattern at (10, 10):");
    println!("  Horizontal stones: (10,8), (10,9)");
    println!("  Vertical stones: (8,10), (9,10)");
    println!("  Creates double-three: {}", creates_dt2);
    
    if creates_dt2 {
        println!("✓ Simple horizontal+vertical pattern works!");
    } else {
        println!("✗ Simple pattern does NOT create double-three");
        println!("  This means we need to adjust our test patterns!");
    }
    
    // Test with more spacing
    let mut board3 = Board::new(19);
    
    // Create patterns with more context/space
    // Horizontal: _ O O _ with good extension space
    board3.place_stone(10, 8, Player::Min);
    board3.place_stone(10, 9, Player::Min);
    // Leave (10, 7), (10, 10), (10, 11) empty for extensions
    
    // Vertical: _ O O _ with good extension space
    board3.place_stone(8, 10, Player::Min);
    board3.place_stone(9, 10, Player::Min);
    // Leave (7, 10), (10, 10), (11, 10) empty for extensions
    
    let creates_dt3 = DoubleThreeDetection::creates_double_three(&board3, 10, 10, Player::Min);
    
    println!("\nPattern with extension space at (10, 10):");
    println!("  Creates double-three: {}", creates_dt3);
    
    if !creates_dt2 && !creates_dt3 {
        println!("\n⚠️  WARNING: Neither simple pattern creates double-three!");
        println!("   Our test assumptions about patterns may be wrong.");
        println!("   We need to study actual working double-three patterns!");
    }
    
    assert!(
        creates_dt || creates_dt2 || creates_dt3,
        "At least one pattern should create double-three, or detection is broken"
    );
}

#[test]
fn find_working_double_three_pattern() {
    // Let's systematically find a pattern that DEFINITELY works
    
    let mut board = Board::new(19);
    let center = 10;
    
    println!("\n=== Finding Working Double-Three Pattern ===");
    
    // Try pattern from test_double_three.rs that we know passes
    // Diagonal pattern
    board.place_stone(8, 8, Player::Max);
    board.place_stone(10, 10, Player::Max);
    board.place_stone(8, 10, Player::Max);
    board.place_stone(10, 8, Player::Max);
    
    let diagonal_works = DoubleThreeDetection::creates_double_three(&board, 9, 9, Player::Max);
    println!("Diagonal cross pattern at (9,9): {}", diagonal_works);
    
    if diagonal_works {
        println!("✓ FOUND WORKING PATTERN: Diagonal cross");
        println!("  Stones at: (8,8), (10,10), (8,10), (10,8)");
        println!("  Double-three at: (9,9) for Player::Max");
    }
    
    // Try another known pattern
    let mut board2 = Board::new(19);
    
    // From test_double_three_all_four_directions
    board2.place_stone(center - 1, center - 1, Player::Min);
    board2.place_stone(center - 1, center + 1, Player::Min);
    board2.place_stone(center + 1, center - 1, Player::Min);
    board2.place_stone(center + 1, center + 1, Player::Min);
    
    let quad_works = DoubleThreeDetection::creates_double_three(&board2, center, center, Player::Min);
    println!("\nQuad diagonal pattern at ({},{}): {}", center, center, quad_works);
    
    if quad_works {
        println!("✓ FOUND WORKING PATTERN: Quad diagonal");
        println!("  Stones at corners around center");
        println!("  Double-three at: ({},{}) for Player::Min", center, center);
    }
    
    assert!(
        diagonal_works || quad_works,
        "We must be able to create SOME double-three pattern"
    );
}

#[test]
fn test_exact_pattern_for_ai_tests() {
    // Create the EXACT pattern we'll use in AI tests
    // This must pass for AI tests to be valid
    
    println!("\n=== Pattern for AI Tests ===");
    
    let mut board = Board::new(19);
    let target_row = 10;
    let target_col = 10;
    
    // Pattern attempt 1: Simple adjacent stones
    board.place_stone(target_row, target_col - 2, Player::Min);
    board.place_stone(target_row, target_col - 1, Player::Min);
    board.place_stone(target_row - 2, target_col, Player::Min);
    board.place_stone(target_row - 1, target_col, Player::Min);
    
    let pattern1_works = DoubleThreeDetection::creates_double_three(
        &board, target_row, target_col, Player::Min
    );
    
    println!("Pattern 1 (adjacent horizontal+vertical):");
    println!("  Horizontal: ({},{}) ({},{})", target_row, target_col-2, target_row, target_col-1);
    println!("  Vertical: ({},{}) ({},{})", target_row-2, target_col, target_row-1, target_col);
    println!("  Target: ({},{}) for Player::Min", target_row, target_col);
    println!("  Result: {}", pattern1_works);
    
    if pattern1_works {
        println!("\n✓✓✓ THIS PATTERN WORKS! Use it in AI tests! ✓✓✓");
    } else {
        println!("\n✗ Pattern 1 doesn't work, trying alternatives...");
        
        // Try with gaps
        let mut board2 = Board::new(19);
        board2.place_stone(target_row, target_col - 3, Player::Min);
        board2.place_stone(target_row, target_col - 1, Player::Min);
        board2.place_stone(target_row - 3, target_col, Player::Min);
        board2.place_stone(target_row - 1, target_col, Player::Min);
        
        let pattern2_works = DoubleThreeDetection::creates_double_three(
            &board2, target_row, target_col, Player::Min
        );
        
        println!("\nPattern 2 (with gaps):");
        println!("  Result: {}", pattern2_works);
        
        if !pattern2_works {
            println!("\n⚠️⚠️⚠️  CRITICAL: Cannot create double-three at (10,10) ⚠️⚠️⚠️");
            println!("We need to use a DIFFERENT position for our AI tests!");
        }
    }
    
    assert!(
        pattern1_works,
        "MUST be able to create double-three at target position for AI tests"
    );
}
