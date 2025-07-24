use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::find_best_move_parallel;
use gomoku::ai::transposition::SharedTranspositionTable;

fn main() {
    println!("=== Testing Transposition Table Interference ===");
    
    // Test with shared TT (like the failing test)
    let shared_tt = SharedTranspositionTable::new_default();
    
    for i in 0..3 {
        let mut state = GameState::new(15, 5);
        let base_row = 6 + (i % 3);
        
        println!("\n--- Iteration {} (base_row = {}) ---", i, base_row);
        
        // Create the same position as the test
        state.board.place_stone(base_row, 6, Player::Max);
        state.board.place_stone(base_row, 7, Player::Max);
        state.board.place_stone(base_row, 8, Player::Max);
        state.board.place_stone(base_row, 9, Player::Max);
        
        state.board.place_stone(base_row + 1, 7, Player::Min);
        state.board.place_stone(base_row - 1, 8, Player::Min);
        
        state.current_player = Player::Max;
        
        print_small_board(&state, base_row);
        
        let best_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
        println!("With shared TT: {:?}", best_move);
        
        // Test with fresh TT
        let fresh_tt = SharedTranspositionTable::new_default();
        let mut state_copy = state.clone();
        let best_move_fresh = find_best_move_parallel(&mut state_copy, 6, None, &fresh_tt);
        println!("With fresh TT: {:?}", best_move_fresh);
        
        let expected_moves = vec![(base_row, 5), (base_row, 10)];
        println!("Expected: {:?}", expected_moves);
        
        if let Some(mv) = best_move {
            if !expected_moves.contains(&mv) {
                println!("❌ MISMATCH with shared TT!");
            } else {
                println!("✅ Correct with shared TT");
            }
        }
        
        if let Some(mv) = best_move_fresh {
            if !expected_moves.contains(&mv) {
                println!("❌ MISMATCH with fresh TT!");
            } else {
                println!("✅ Correct with fresh TT");
            }
        }
    }
}

fn print_small_board(state: &GameState, center_row: usize) {
    let start = center_row.saturating_sub(2);
    let end = (center_row + 3).min(15);
    
    print!("    ");
    for col in 4..12 {
        print!("{} ", col);
    }
    println!();
    
    for row in start..end {
        print!("{:2}: ", row);
        for col in 4..12 {
            let symbol = if state.board.is_empty_position(row, col) {
                "."
            } else if state.board.get_player(row, col) == Some(Player::Max) {
                "X"
            } else {
                "O"
            };
            print!("{} ", symbol);
        }
        println!();
    }
}
