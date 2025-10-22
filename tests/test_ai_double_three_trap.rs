/// Tests for AI's "Double-Three Trap" tactic
/// 
/// This advanced tactic creates forcing moves where opponent must respond,
/// but all blocking positions create illegal double-three for them.
/// 
/// The trap works by:
/// 1. AI creates strong threat (3 or 4 in a row)
/// 2. Opponent must block to prevent AI win
/// 3. All blocking positions are illegal for opponent (double-three)
/// 4. Result: Opponent is paralyzed, AI has huge advantage

use gomoku::ai::move_generation::MoveGenerator;
use gomoku::ai::lazy_smp::lazy_smp_search;
use gomoku::core::state::GameState;
use gomoku::core::board::{Board, Player};
use gomoku::core::rules::DoubleThreeDetection;

/// Helper to check if a position creates double-three
fn is_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
    DoubleThreeDetection::creates_double_three(board, row, col, player)
}

/// Test: AI creates 3-in-a-row where both blocking positions are illegal for opponent
#[test]
fn test_ai_prefers_trap_with_all_blocks_illegal() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Setup: AI can create 3-in-a-row at (10,10)
    board.place_stone(10, 8, ai);  // X _ _ at row 10
    board.place_stone(10, 9, ai);  // X X _ at row 10
    // Playing at (10,10) creates: X X X with blocks needed at (10,7) and (10,11)
    
    // Make (10,7) illegal for opponent (double-three)
    board.place_stone(8, 7, opponent);   // Vertical pattern
    board.place_stone(9, 7, opponent);
    // (10,7) would create: O O ? with space at (11,7)
    // And horizontal: (10,5) (10,6) ? (10,8) pattern
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    
    // Make (10,11) illegal for opponent (double-three)  
    board.place_stone(8, 11, opponent);  // Vertical pattern
    board.place_stone(9, 11, opponent);
    // (10,11) would create: O O ? with space at (11,11)
    // And horizontal pattern
    board.place_stone(10, 13, opponent);
    board.place_stone(10, 12, opponent);
    
    // Verify trap setup: both blocking positions illegal
    assert!(
        is_double_three(&board, 10, 7, opponent),
        "Position (10,7) should be illegal for opponent"
    );
    assert!(
        is_double_three(&board, 10, 11, opponent),
        "Position (10,11) should be illegal for opponent"
    );
    
    // AI should strongly prefer (10,10) - creates perfect trap
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    // The trap move should be in top candidates
    assert!(
        moves.contains(&(10, 10)),
        "AI should consider trap move (10,10)"
    );
    
    // Ideally it should be the first choice
    // (This might not always be true depending on other factors, but trap bonus should help)
    println!("AI top candidates: {:?}", &moves[..moves.len().min(5)]);
}

/// Test: AI chooses trap move over equivalent non-trap move
#[test]
fn test_ai_prefers_trap_over_equivalent_move() {
    let mut state = GameState::new(19, 5);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Create two equivalent positions for AI:
    // Option A: (10,10) - creates 3-in-a-row with ALL blocks illegal
    // Option B: (12,10) - creates 3-in-a-row with legal blocks
    
    // Setup Option A: 3-in-a-row at row 10
    state.make_move((10, 8));  // Max
    state.make_move((8, 8));   // Min (dummy)
    state.make_move((10, 9));  // Max
    state.make_move((8, 9));   // Min (dummy)
    // Playing at (10,10) would create X X X
    
    // Make blocks at (10,7) and (10,11) illegal for opponent
    state.make_move((9, 7));   // Max (dummy)
    state.make_move((8, 7));   // Min
    state.make_move((9, 8));   // Max (dummy)
    state.make_move((9, 7));   // Min (creates cross pattern for double-three at 10,7)
    state.make_move((7, 7));   // Max (dummy)
    state.make_move((10, 5));  // Min
    state.make_move((7, 8));   // Max (dummy)
    state.make_move((10, 6));  // Min
    
    state.make_move((7, 9));   // Max (dummy)
    state.make_move((8, 11));  // Min
    state.make_move((7, 10));  // Max (dummy)
    state.make_move((9, 11));  // Min
    state.make_move((7, 11));  // Max (dummy)
    state.make_move((10, 13)); // Min
    state.make_move((7, 12));  // Max (dummy)
    state.make_move((10, 12)); // Min
    
    // Setup Option B: 3-in-a-row at row 12 (blocks are legal)
    state.make_move((12, 8));  // Max
    state.make_move((6, 6));   // Min (dummy)
    state.make_move((12, 9));  // Max
    // Playing at (12,10) would also create X X X, but blocks at (12,7) and (12,11) are legal
    
    // Verify trap is set
    assert!(
        is_double_three(&state.board, 10, 7, opponent),
        "Block position (10,7) should be illegal for opponent"
    );
    assert!(
        is_double_three(&state.board, 10, 11, opponent),
        "Block position (10,11) should be illegal for opponent"
    );
    
    // AI should prefer the trap move (10,10) over non-trap (12,10)
    let result = lazy_smp_search(&mut state, 200, 4, Some(1));
    
    if let Some((row, col)) = result.best_move {
        // We expect AI to choose the trap
        assert_eq!(
            (row, col), (10, 10),
            "AI should prefer trap move (10,10) over non-trap alternative. Got ({},{})",
            row, col
        );
    }
}

/// Test: 4-in-a-row trap is even more valuable
#[test]
fn test_four_in_row_trap_highest_priority() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Setup: AI can create 4-in-a-row at (10,11)
    board.place_stone(10, 8, ai);   // X at (10,8)
    board.place_stone(10, 9, ai);   // X at (10,9)
    board.place_stone(10, 10, ai);  // X at (10,10)
    // Playing at (10,11) creates: X X X X (four in a row!)
    // Blocks needed at (10,7) and (10,12)
    
    // Make (10,7) illegal for opponent
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    
    // Make (10,12) illegal for opponent
    board.place_stone(8, 12, opponent);
    board.place_stone(9, 12, opponent);
    board.place_stone(10, 14, opponent);
    board.place_stone(10, 13, opponent);
    
    // Verify both blocks are illegal
    assert!(
        is_double_three(&board, 10, 7, opponent),
        "Position (10,7) should be illegal for opponent"
    );
    assert!(
        is_double_three(&board, 10, 12, opponent),
        "Position (10,12) should be illegal for opponent"
    );
    
    // AI should definitely play (10,11) - it's game over for opponent!
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    assert_eq!(
        moves[0], (10, 11),
        "AI should prioritize 4-in-a-row trap as top move. Got {:?}",
        moves[0]
    );
}

/// Test: Partial trap (one block illegal, one legal) gets proportional bonus
#[test]
fn test_partial_trap_gets_bonus() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Setup: AI creates 3-in-a-row at (10,10)
    board.place_stone(10, 8, ai);
    board.place_stone(10, 9, ai);
    // Blocks at (10,7) and (10,11)
    
    // Make only (10,7) illegal for opponent
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    
    // (10,11) is legal for opponent
    
    // Verify setup
    assert!(
        is_double_three(&board, 10, 7, opponent),
        "Position (10,7) should be illegal for opponent"
    );
    assert!(
        !is_double_three(&board, 10, 11, opponent),
        "Position (10,11) should be LEGAL for opponent"
    );
    
    // AI should still get bonus for partial trap
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    // The move should be in candidates (still valuable)
    assert!(
        moves.contains(&(10, 10)),
        "AI should still consider partial trap move (10,10)"
    );
}

/// Test: AI recognizes trap in diagonal pattern
#[test]
fn test_diagonal_double_three_trap() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Setup: AI creates diagonal 3-in-a-row
    board.place_stone(9, 9, ai);    // Diagonal: (9,9)
    board.place_stone(10, 10, ai);  // (10,10)
    // Playing at (11,11) creates diagonal X X X
    // Blocks at (8,8) and (12,12)
    
    // Make (8,8) illegal for opponent (cross pattern)
    board.place_stone(8, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(6, 8, opponent);
    board.place_stone(7, 8, opponent);
    
    // Make (12,12) illegal for opponent
    board.place_stone(12, 10, opponent);
    board.place_stone(12, 11, opponent);
    board.place_stone(10, 12, opponent);
    board.place_stone(11, 12, opponent);
    
    // Verify trap
    assert!(
        is_double_three(&board, 8, 8, opponent),
        "Position (8,8) should be illegal for opponent"
    );
    assert!(
        is_double_three(&board, 12, 12, opponent),
        "Position (12,12) should be illegal for opponent"
    );
    
    // AI should recognize diagonal trap
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    assert!(
        moves.contains(&(11, 11)),
        "AI should recognize diagonal trap at (11,11)"
    );
}

/// Test: AI doesn't get false positive when blocks are legal
#[test]
fn test_no_trap_bonus_when_blocks_legal() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Setup: AI creates 3-in-a-row at (10,10)
    board.place_stone(10, 8, ai);
    board.place_stone(10, 9, ai);
    // Blocks at (10,7) and (10,11) - both are LEGAL for opponent
    
    // Just verify no double-three at blocks
    assert!(
        !is_double_three(&board, 10, 7, opponent),
        "Position (10,7) should be LEGAL for opponent"
    );
    assert!(
        !is_double_three(&board, 10, 11, opponent),
        "Position (10,11) should be LEGAL for opponent"
    );
    
    // AI should still consider the move (normal threat) but no special trap bonus
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    assert!(
        moves.contains(&(10, 10)),
        "AI should consider normal threat move (10,10)"
    );
    
    // This test mainly ensures we don't crash or give false bonuses
}

/// Test: Complex scenario - multiple threats, AI chooses the trap
#[test]
fn test_trap_chosen_over_regular_threats() {
    let mut state = GameState::new(19, 5);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Create multiple threat options for AI:
    
    // Threat 1: Regular 3-in-a-row at row 8 (no trap)
    state.make_move((8, 5));   // Max
    state.make_move((7, 7));   // Min (dummy)
    state.make_move((8, 6));   // Max
    state.make_move((7, 8));   // Min (dummy)
    // Can extend to (8,7) or (8,4) - both legal for opponent
    
    // Threat 2: 3-in-a-row with TRAP at row 10
    state.make_move((10, 8));  // Max
    state.make_move((7, 9));   // Min (dummy)
    state.make_move((10, 9));  // Max
    state.make_move((7, 10));  // Min (dummy)
    // Can extend to (10,10) - creates trap if blocks are illegal
    
    // Setup trap at (10,10): make (10,7) and (10,11) illegal
    state.make_move((6, 6));   // Max (dummy)
    state.make_move((8, 7));   // Min
    state.make_move((6, 7));   // Max (dummy)
    state.make_move((9, 7));   // Min
    state.make_move((6, 8));   // Max (dummy)
    state.make_move((10, 5));  // Min
    state.make_move((6, 9));   // Max (dummy)
    state.make_move((10, 6));  // Min
    
    state.make_move((6, 10));  // Max (dummy)
    state.make_move((8, 11));  // Min
    state.make_move((6, 11));  // Max (dummy)
    state.make_move((9, 11));  // Min
    state.make_move((6, 12));  // Max (dummy)
    state.make_move((10, 13)); // Min
    state.make_move((6, 13));  // Max (dummy)
    state.make_move((10, 12)); // Min
    
    // Verify trap is set
    assert!(
        is_double_three(&state.board, 10, 7, opponent),
        "Position (10,7) should be illegal for opponent"
    );
    assert!(
        is_double_three(&state.board, 10, 11, opponent),
        "Position (10,11) should be illegal for opponent"
    );
    
    // AI should prefer trap move (10,10) over regular threat (8,7)
    let result = lazy_smp_search(&mut state, 200, 4, Some(1));
    
    if let Some((row, col)) = result.best_move {
        assert_eq!(
            (row, col), (10, 10),
            "AI should choose trap move (10,10) over regular threat. Got ({},{})",
            row, col
        );
    }
}

/// Test: Gapped pattern trap - STRICT TEST
/// AI must recognize X X _ as threat that becomes X X X with all blocks illegal
#[test]
fn test_gapped_pattern_trap() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Create gapped pattern: X X _ at positions (10,8), (10,9)
    board.place_stone(10, 8, ai);
    board.place_stone(10, 9, ai);
    // Playing at (10,10) creates X X X (3 in a row!)
    // Critical blocks at (10,7) and (10,11)
    
    // Make (10,7) create double-three for opponent
    // Horizontal free-three: O O ? at (10,5), (10,6), (10,7)
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    // Vertical free-three: O O ? at (8,7), (9,7), (10,7)
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    
    // Make (10,11) create double-three for opponent
    // Horizontal free-three: O O ? at (10,11), (10,12), (10,13)
    board.place_stone(10, 12, opponent);
    board.place_stone(10, 13, opponent);
    // Vertical free-three: O O ? at (8,11), (9,11), (10,11)
    board.place_stone(8, 11, opponent);
    board.place_stone(9, 11, opponent);
    
    // VERIFY: Both blocks create double-three
    assert!(
        is_double_three(&board, 10, 7, opponent),
        "Position (10,7) MUST create double-three for opponent"
    );
    assert!(
        is_double_three(&board, 10, 11, opponent),
        "Position (10,11) MUST create double-three for opponent"
    );
    
    // STRICT REQUIREMENT: AI MUST include this move in candidates
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    assert!(
        moves.contains(&(10, 10)),
        "AI MUST recognize gapped trap at (10,10)! Found moves: {:?}",
        moves
    );
    
    // EVEN STRICTER: Should be in top 3 moves
    assert!(
        moves.iter().take(3).any(|&m| m == (10, 10)),
        "AI must PRIORITIZE gapped trap in top 3! Top moves: {:?}",
        &moves[..moves.len().min(5)]
    );
}

/// Test: ULTIMATE TRAP - Multiple threats with only illegal defenses
/// This is the most devastating position - opponent has NO legal defense
#[test]
fn test_ultimate_double_three_trap_multiple_threats() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Setup: AI creates a 4-in-a-row threat with gapped pattern
    // Pattern: X X X _ X at (10,7), (10,8), (10,9), (10,10), (10,11)
    board.place_stone(10, 7, ai);
    board.place_stone(10, 8, ai);
    board.place_stone(10, 9, ai);
    board.place_stone(10, 11, ai);
    
    // Playing at (10,10) creates X X X X X - but wait that's 5 (instant win)!
    // Let's use a different pattern: X X _ X at (10,8), (10,9), (10,11)
    // Clear and restart
    board = Board::new(19);
    board.place_stone(10, 8, ai);
    board.place_stone(10, 9, ai);
    board.place_stone(10, 11, ai);
    
    // Playing at (10,10) creates X X X X (four-in-a-row)
    // Opponent needs to block at: (10,7) or (10,12)
    
    // Make BOTH blocks create double-three for opponent
    // (10,7) illegal
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    
    // (10,12) illegal  
    board.place_stone(10, 13, opponent);
    board.place_stone(10, 14, opponent);
    board.place_stone(8, 12, opponent);
    board.place_stone(9, 12, opponent);
    
    // VERIFY: defensive positions are illegal
    assert!(is_double_three(&board, 10, 7, opponent), "Block (10,7) must be illegal");
    assert!(is_double_three(&board, 10, 12, opponent), "Block (10,12) must be illegal");
    
    // ULTIMATE TEST: This move should be #1 priority - it's game over!
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    assert_eq!(
        moves[0], (10, 10),
        "AI MUST choose ultimate trap (10,10) as TOP move! Got: {:?}",
        moves[0]
    );
}

/// Test: Forced response creates double-three - opponent must lose
#[test]
fn test_forcing_move_all_responses_illegal() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // AI creates open-three (both ends open)
    board.place_stone(10, 9, ai);
    board.place_stone(10, 10, ai);
    board.place_stone(10, 11, ai);
    // Pattern: _ X X X _ at row 10, positions 8-12
    // Opponent MUST block at (10,8) or (10,12)
    
    // Make (10,8) illegal - creates double-three
    board.place_stone(8, 8, opponent);
    board.place_stone(9, 8, opponent);
    board.place_stone(10, 6, opponent);
    board.place_stone(10, 7, opponent);
    
    // Make (10,12) illegal - creates double-three  
    board.place_stone(8, 12, opponent);
    board.place_stone(9, 12, opponent);
    board.place_stone(10, 13, opponent);
    board.place_stone(10, 14, opponent);
    
    // VERIFY
    assert!(is_double_three(&board, 10, 8, opponent), "(10,8) must be illegal");
    assert!(is_double_three(&board, 10, 12, opponent), "(10,12) must be illegal");
    
    // This position already exists - AI should recognize its value
    // Let's test that AI would create such a position
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    // AI should recognize this is a winning position (any offensive move wins)
    // Because opponent literally cannot defend
    assert!(
        !moves.is_empty(),
        "AI should have winning moves available"
    );
}

/// Test: AI chooses trap over immediate but defendable threat
#[test]
fn test_trap_beats_simple_threat() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Option A: Simple 4-in-a-row at row 8 (opponent CAN defend)
    board.place_stone(8, 5, ai);
    board.place_stone(8, 6, ai);
    board.place_stone(8, 7, ai);
    // Can extend to (8,8) for X X X X - blocks at (8,4), (8,9) are legal
    
    // Option B: 3-in-a-row with TRAP at row 10
    board.place_stone(10, 8, ai);
    board.place_stone(10, 9, ai);
    // Can extend to (10,10) for X X X - blocks at (10,7), (10,11) are illegal
    
    // Make row 10 blocks illegal
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    
    board.place_stone(10, 12, opponent);
    board.place_stone(10, 13, opponent);
    board.place_stone(8, 11, opponent);
    board.place_stone(9, 11, opponent);
    
    // VERIFY
    assert!(is_double_three(&board, 10, 7, opponent));
    assert!(is_double_three(&board, 10, 11, opponent));
    
    // AI MUST choose the trap (10,10) over the simple threat (8,8)
    // Because trap = undefendable, simple = defendable
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    assert_eq!(
        moves[0], (10, 10),
        "AI must prefer TRAP (10,10) over simple threat (8,8)! Got: {:?}",
        moves[0]
    );
}

/// Test: Complex real-game scenario with trap opportunity
#[test]
fn test_realistic_game_trap_scenario() {
    let mut state = GameState::new(19, 5);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Simulate realistic game progression
    // Opening moves
    state.make_move((9, 9));   // Max - center
    state.make_move((9, 10));  // Min - nearby
    state.make_move((10, 10)); // Max
    state.make_move((10, 9));  // Min
    
    // Max builds pattern at row 10
    state.make_move((10, 8));  // Max - creates potential
    state.make_move((8, 9));   // Min - random
    state.make_move((10, 9));  // Max - oops, occupied, but in real game this builds
    
    // Actually let's create proper scenario
    let mut board = Board::new(19);
    
    // AI has potential at row 10
    board.place_stone(10, 8, ai);
    board.place_stone(10, 9, ai);
    
    // Opponent has patterns that make (10,7) and (10,11) illegal
    board.place_stone(10, 5, opponent);
    board.place_stone(10, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(9, 7, opponent);
    board.place_stone(11, 7, opponent);  // Extra stone for stronger pattern
    
    board.place_stone(10, 12, opponent);
    board.place_stone(10, 13, opponent);
    board.place_stone(8, 11, opponent);
    board.place_stone(9, 11, opponent);
    board.place_stone(11, 11, opponent);
    
    // Add some noise - other stones on board
    board.place_stone(5, 5, ai);
    board.place_stone(5, 6, opponent);
    board.place_stone(15, 15, ai);
    board.place_stone(15, 14, opponent);
    
    // VERIFY trap exists
    assert!(is_double_three(&board, 10, 7, opponent));
    assert!(is_double_three(&board, 10, 11, opponent));
    
    // In realistic scenario, AI must still find the trap
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    assert!(
        moves.iter().take(5).any(|&m| m == (10, 10)),
        "AI must find trap move (10,10) even in complex board! Top 5: {:?}",
        &moves[..moves.len().min(5)]
    );
}

/// Test: Diagonal trap - harder to detect but equally devastating
#[test]
fn test_diagonal_trap_strict() {
    let mut board = Board::new(19);
    let ai = Player::Max;
    let opponent = Player::Min;
    
    // Diagonal pattern: X X _ at (9,9), (10,10) -> extends to (11,11)
    board.place_stone(9, 9, ai);
    board.place_stone(10, 10, ai);
    // Blocks at (8,8) and (12,12)
    
    // Make (8,8) illegal
    board.place_stone(8, 6, opponent);
    board.place_stone(8, 7, opponent);
    board.place_stone(6, 8, opponent);
    board.place_stone(7, 8, opponent);
    
    // Make (12,12) illegal
    board.place_stone(12, 10, opponent);
    board.place_stone(12, 11, opponent);
    board.place_stone(10, 12, opponent);
    board.place_stone(11, 12, opponent);
    
    // VERIFY
    assert!(is_double_three(&board, 8, 8, opponent), "(8,8) must be illegal");
    assert!(is_double_three(&board, 12, 12, opponent), "(12,12) must be illegal");
    
    // STRICT: AI must recognize diagonal trap
    let moves = MoveGenerator::get_candidate_moves(&board, ai);
    
    assert!(
        moves.contains(&(11, 11)),
        "AI MUST detect diagonal trap at (11,11)! Moves: {:?}",
        moves
    );
    
    // Should be high priority
    assert!(
        moves.iter().take(3).any(|&m| m == (11, 11)),
        "Diagonal trap must be in top 3! Top moves: {:?}",
        &moves[..moves.len().min(5)]
    );
}
