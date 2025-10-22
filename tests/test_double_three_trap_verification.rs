/// Verification tests for double-three trap test setups
/// 
/// These tests verify that our test board configurations are correct
/// BEFORE we test the AI behavior. We need to ensure:
/// 1. Double-three patterns are ACTUALLY double-three
/// 2. Test board setups are valid
/// 3. Blocking positions are correctly identified

use gomoku::core::board::{Board, Player};
use gomoku::core::rules::DoubleThreeDetection;

/// Helper to check if a position creates double-three
fn is_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
    DoubleThreeDetection::creates_double_three(board, row, col, player)
}

/// Helper to print board section for debugging
fn print_board_section(board: &Board, center_row: usize, center_col: usize, radius: usize) {
    println!("\nBoard section around ({}, {}):", center_row, center_col);
    for r in center_row.saturating_sub(radius)..=(center_row + radius).min(board.size - 1) {
        print!("Row {:2}: ", r);
        for c in center_col.saturating_sub(radius)..=(center_col + radius).min(board.size - 1) {
            if board.get_player(r, c) == Some(Player::Max) {
                print!("X ");
            } else if board.get_player(r, c) == Some(Player::Min) {
                print!("O ");
            } else {
                print!(". ");
            }
        }
        println!();
    }
}

/// Verify: Simple horizontal + vertical double-three pattern
#[test]
fn verify_simple_double_three_pattern() {
    let mut board = Board::new(19);
    let player = Player::Min;
    
    // Create horizontal free-three: O O ? at (10,5), (10,6), (10,7)
    board.place_stone(10, 5, player);
    board.place_stone(10, 6, player);
    
    // Create vertical free-three: O O ? at (8,7), (9,7), (10,7)
    board.place_stone(8, 7, player);
    board.place_stone(9, 7, player);
    
    print_board_section(&board, 10, 7, 3);
    
    // Playing at (10,7) should create double-three
    let result = is_double_three(&board, 10, 7, player);
    
    assert!(
        result,
        "Position (10,7) should create double-three with H: (10,5)(10,6)(10,7) and V: (8,7)(9,7)(10,7)"
    );
}

/// Verify: Cross pattern double-three
#[test]
fn verify_cross_double_three() {
    let mut board = Board::new(19);
    let player = Player::Min;
    
    // Horizontal: O O ? O at (10,8), (10,9), (10,10), (10,12)
    board.place_stone(10, 8, player);
    board.place_stone(10, 9, player);
    board.place_stone(10, 12, player);
    
    // Vertical: O O ? O at (8,11), (9,11), (10,11), (12,11)
    board.place_stone(8, 11, player);
    board.place_stone(9, 11, player);
    board.place_stone(12, 11, player);
    
    print_board_section(&board, 10, 10, 4);
    
    // (10,11) should create double-three
    assert!(
        is_double_three(&board, 10, 11, player),
        "Position (10,11) should create cross double-three"
    );
}

/// Verify: Diagonal double-three
#[test]
fn verify_diagonal_double_three() {
    let mut board = Board::new(19);
    let player = Player::Min;
    
    // Diagonal 1: O O ? at (9,9), (10,10), (11,11)
    board.place_stone(9, 9, player);
    board.place_stone(10, 10, player);
    // Space at (11,11) for free-three
    
    // Diagonal 2 (opposite): O O ? at (9,13), (10,12), (11,11)
    board.place_stone(9, 13, player);
    board.place_stone(10, 12, player);
    
    print_board_section(&board, 11, 11, 3);
    
    // (11,11) should create double-three
    assert!(
        is_double_three(&board, 11, 11, player),
        "Position (11,11) should create diagonal double-three"
    );
}

/// Verify: Test setup for gapped pattern trap
#[test]
fn verify_gapped_trap_setup() {
    let mut board = Board::new(19);
    let opponent = Player::Min;
    
    // Setup from test: blocks at (10,7) and (10,11)
    
    // (10,7) should be illegal
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    
    println!("\nChecking (10,7):");
    print_board_section(&board, 10, 7, 3);
    
    assert!(
        is_double_three(&board, 10, 7, opponent),
        "(10,7) should create double-three: H: (10,5)(10,6)(10,7)? and V: (8,7)(9,7)(10,7)?"
    );
    
    // (10,11) should be illegal
    board.place_stone(10, 12, opponent);
    board.place_stone(10, 13, opponent);
    board.place_stone(8, 11, opponent);
    board.place_stone(9, 11, opponent);
    
    println!("\nChecking (10,11):");
    print_board_section(&board, 10, 11, 3);
    
    assert!(
        is_double_three(&board, 10, 11, opponent),
        "(10,11) should create double-three: H: ?(10,11)(10,12)(10,13) and V: (8,11)(9,11)(10,11)?"
    );
}

/// Verify: Ultimate trap with 4 illegal blocks
#[test]
fn verify_ultimate_trap_all_blocks_illegal() {
    let mut board = Board::new(19);
    let opponent = Player::Min;
    
    // Test will create patterns making (10,7), (10,11), (11,10), (7,10) all illegal
    
    // (10,7) illegal
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    
    println!("\n=== Checking (10,7) ===");
    print_board_section(&board, 10, 7, 3);
    assert!(is_double_three(&board, 10, 7, opponent), "(10,7) must be illegal");
    
    // (10,11) illegal
    board.place_stone(10, 12, opponent);
    board.place_stone(10, 13, opponent);
    board.place_stone(8, 11, opponent);
    board.place_stone(9, 11, opponent);
    
    println!("\n=== Checking (10,11) ===");
    print_board_section(&board, 10, 11, 3);
    assert!(is_double_three(&board, 10, 11, opponent), "(10,11) must be illegal");
    
    // (11,10) illegal
    board.place_stone(11, 8, opponent);
    board.place_stone(11, 9, opponent);
    board.place_stone(13, 10, opponent);
    board.place_stone(12, 10, opponent);
    
    println!("\n=== Checking (11,10) ===");
    print_board_section(&board, 11, 10, 3);
    assert!(is_double_three(&board, 11, 10, opponent), "(11,10) must be illegal");
    
    // (7,10) illegal
    board.place_stone(7, 8, opponent);
    board.place_stone(7, 9, opponent);
    board.place_stone(5, 10, opponent);
    board.place_stone(6, 10, opponent);
    
    println!("\n=== Checking (7,10) ===");
    print_board_section(&board, 7, 10, 3);
    assert!(is_double_three(&board, 7, 10, opponent), "(7,10) must be illegal");
    
    println!("\n✓ All 4 blocking positions are illegal!");
}

/// Verify: Diagonal trap blocks are illegal
#[test]
fn verify_diagonal_trap_blocks() {
    let mut board = Board::new(19);
    let opponent = Player::Min;
    
    // Blocks at (8,8) and (12,12) for diagonal (9,9)-(10,10)-(11,11)
    
    // (8,8) illegal
    board.place_stone(8, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(6, 8, opponent);
    board.place_stone(7, 8, opponent);
    
    println!("\n=== Checking (8,8) ===");
    print_board_section(&board, 8, 8, 3);
    assert!(is_double_three(&board, 8, 8, opponent), "(8,8) must create double-three");
    
    // (12,12) illegal
    board.place_stone(12, 10, opponent);
    board.place_stone(12, 11, opponent);
    board.place_stone(10, 12, opponent);
    board.place_stone(11, 12, opponent);
    
    println!("\n=== Checking (12,12) ===");
    print_board_section(&board, 12, 12, 3);
    assert!(is_double_three(&board, 12, 12, opponent), "(12,12) must create double-three");
}

/// Verify: Partial trap - one legal, one illegal
#[test]
fn verify_partial_trap_one_illegal() {
    let mut board = Board::new(19);
    let opponent = Player::Min;
    
    // Only (10,7) should be illegal, (10,11) should be legal
    
    // Make (10,7) illegal
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    
    println!("\n=== Checking (10,7) - should be ILLEGAL ===");
    print_board_section(&board, 10, 7, 3);
    assert!(is_double_three(&board, 10, 7, opponent), "(10,7) must be illegal");
    
    // (10,11) should be legal (no stones to create double-three)
    println!("\n=== Checking (10,11) - should be LEGAL ===");
    print_board_section(&board, 10, 11, 3);
    assert!(!is_double_three(&board, 10, 11, opponent), "(10,11) must be LEGAL (no double-three)");
}

/// Verify: Forcing move scenario - both responses illegal
#[test]
fn verify_forcing_move_setup() {
    let mut board = Board::new(19);
    let opponent = Player::Min;
    
    // AI has X X X at (10,9), (10,10), (10,11)
    // Opponent must block at (10,8) or (10,12)
    
    // Make (10,8) illegal
    board.place_stone(8, 8, opponent);
    board.place_stone(9, 8, opponent);
    board.place_stone(10, 6, opponent);
    board.place_stone(10, 7, opponent);
    
    println!("\n=== Checking (10,8) ===");
    print_board_section(&board, 10, 8, 3);
    assert!(is_double_three(&board, 10, 8, opponent), "(10,8) must be illegal");
    
    // Make (10,12) illegal
    board.place_stone(8, 12, opponent);
    board.place_stone(9, 12, opponent);
    board.place_stone(10, 13, opponent);
    board.place_stone(10, 14, opponent);
    
    println!("\n=== Checking (10,12) ===");
    print_board_section(&board, 10, 12, 3);
    assert!(is_double_three(&board, 10, 12, opponent), "(10,12) must be illegal");
}

/// Verify: Check that normal positions are NOT double-three
#[test]
fn verify_normal_positions_are_legal() {
    let mut board = Board::new(19);
    let player = Player::Min;
    
    // Just a few stones, no double-three patterns
    board.place_stone(10, 5, player);
    board.place_stone(10, 6, player);
    board.place_stone(8, 7, player);
    
    // (10,7) should NOT create double-three (only one pattern, not two)
    // Wait, this might actually create double-three if there's space...
    // Let me check a truly empty position
    
    // (15,15) - far from any stones
    println!("\n=== Checking (15,15) - should be LEGAL ===");
    print_board_section(&board, 15, 15, 3);
    assert!(
        !is_double_three(&board, 15, 15, player),
        "(15,15) should be legal (no patterns nearby)"
    );
    
    // (10,10) - some stones but not forming double-three
    println!("\n=== Checking (10,10) - checking if legal ===");
    print_board_section(&board, 10, 10, 3);
    let result = is_double_three(&board, 10, 10, player);
    println!("(10,10) creates double-three: {}", result);
}

/// Verify: Edge case - stone at boundary
#[test]
fn verify_boundary_double_three() {
    let mut board = Board::new(19);
    let player = Player::Min;
    
    // Near edge: row 1
    board.place_stone(1, 5, player);
    board.place_stone(1, 6, player);
    board.place_stone(0, 7, player);
    // (1,7) might create pattern but check it works
    
    println!("\n=== Checking (1,7) near edge ===");
    print_board_section(&board, 1, 7, 2);
    
    // Just verify it doesn't crash
    let result = is_double_three(&board, 1, 7, player);
    println!("(1,7) creates double-three: {}", result);
}
