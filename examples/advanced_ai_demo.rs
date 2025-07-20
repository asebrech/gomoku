use gomoku::core::state::GameState;
use gomoku::ai::transposition::TranspositionTable;
use gomoku::interface::utils::{
    find_best_move, 
    find_best_move_timed, 
    find_best_move_unlimited, 
    find_best_move_quick
};

fn count_pieces(state: &GameState) -> usize {
    let mut count = 0;
    let size = state.board.size();
    for row in 0..size {
        for col in 0..size {
            if state.board.get_player(row, col).is_some() {
                count += 1;
            }
        }
    }
    count
}

fn main() {
    println!("=== ADVANCED AI FEATURES DEMO ===");
    println!("Demonstrating: Killer Moves, History Heuristic, Iterative Deepening");
    
    // Create a game state with some moves
    let mut state = GameState::new(19, 19);
    let mut tt = TranspositionTable::new(19, 19);
    
    // Set up a more complex tactical position
    state.make_move((9, 9));   // Center
    state.make_move((9, 10));  // Block
    state.make_move((9, 8));   // Extend
    state.make_move((10, 9));  // Block
    state.make_move((9, 7));   // Threat
    state.make_move((8, 9));   // Block
    state.make_move((9, 6));   // Continue threat
    state.make_move((7, 9));   // Block
    state.make_move((10, 10)); // Create another line
    state.make_move((11, 11)); // Block
    state.make_move((8, 8));   // Create diagonal threat
    
    println!("Created test position with {} moves", count_pieces(&state));
    println!("Current player: {:?}", state.current_player);
    
    // Test 1: Regular minimax search (baseline)
    println!("\n=== TEST 1: Regular Minimax (Baseline) ===");
    let baseline_start = std::time::Instant::now();
    let baseline_move = find_best_move(&mut state, 4, &mut tt);
    let baseline_time = baseline_start.elapsed();
    println!("Baseline move: {:?}", baseline_move);
    println!("Baseline time: {:?}", baseline_time);
    
    // Clear TT for fair comparison
    tt.clear();
    
    // Test 2: Advanced search without time limit
    println!("\n=== TEST 2: Advanced Search (Depth 6, No Time Limit) ===");
    let advanced_start = std::time::Instant::now();
    let (advanced_move, advanced_depth) = find_best_move_unlimited(&mut state, 6);
    let advanced_time = advanced_start.elapsed();
    println!("Advanced move: {:?}", advanced_move);
    println!("Advanced depth reached: {}", advanced_depth);
    println!("Advanced time: {:?}", advanced_time);
    
    // Test 3: Advanced search with time limit
    println!("\n=== TEST 3: Advanced Search (2 Second Time Limit) ===");
    let timed_start = std::time::Instant::now();
    let (timed_move, timed_depth) = find_best_move_timed(&mut state, 2000);
    let timed_time = timed_start.elapsed();
    println!("Timed move: {:?}", timed_move);
    println!("Timed depth reached: {}", timed_depth);
    println!("Timed time: {:?}", timed_time);
    
    // Test 4: Quick search
    println!("\n=== TEST 4: Quick Search (1 Second Default) ===");
    let quick_start = std::time::Instant::now();
    let (quick_move, quick_depth) = find_best_move_quick(&mut state);
    let quick_time = quick_start.elapsed();
    println!("Quick move: {:?}", quick_move);
    println!("Quick depth reached: {}", quick_depth);
    println!("Quick time: {:?}", quick_time);
    
    // Test 5: Show consistency across different time limits
    println!("\n=== TEST 5: Consistency Check ===");
    let (move1, depth1) = find_best_move_timed(&mut state, 1000);
    let (move2, depth2) = find_best_move_timed(&mut state, 2000);
    
    println!("Move with 1s limit: {:?} (depth {})", move1, depth1);
    println!("Move with 2s limit: {:?} (depth {})", move2, depth2);
    println!("Moves consistent: {}", move1 == move2);
    
    println!("\n=== FEATURE SUMMARY ===");
    println!("✓ Killer Move Heuristic - Prioritizes moves that caused cutoffs");
    println!("✓ History Heuristic - Learns from successful moves across searches");
    println!("✓ Iterative Deepening - Searches incrementally deeper until time runs out");
    println!("✓ Principal Variation Search - More efficient alpha-beta with null windows");
    println!("✓ Enhanced Move Ordering - TT moves > Killers > History > Positional");
    println!("✓ Time Management - Respects time limits and can stop gracefully");
    
    println!("\n=== BENEFITS ===");
    println!("• Better move ordering leads to more alpha-beta cutoffs");
    println!("• Iterative deepening provides anytime results");
    println!("• History heuristic learns patterns across the entire game");
    println!("• Time management allows for tournament play");
    println!("• All features work together synergistically");
}
