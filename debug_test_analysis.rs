use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::find_best_move_parallel;
use gomoku::ai::transposition::SharedTranspositionTable;

fn main() {
    // Recreate the failing scenario from iteration 1
    let mut state = GameState::new(15, 5);
    let shared_tt = SharedTranspositionTable::new_default();
    
    let i = 1;
    let base_row = 6 + (i % 3); // = 7
    
    println!("=== Analyzing Board Position ===");
    println!("base_row = {}", base_row);
    
    // Place Max stones (4 in a row)
    state.board.place_stone(base_row, 6, Player::Max);
    state.board.place_stone(base_row, 7, Player::Max);
    state.board.place_stone(base_row, 8, Player::Max);
    state.board.place_stone(base_row, 9, Player::Max);
    
    // Add noise (Min stones)
    state.board.place_stone(base_row + 1, 7, Player::Min); // (8, 7)
    state.board.place_stone(base_row - 1, 8, Player::Min); // (6, 8)
    
    state.current_player = Player::Max;
    
    println!("\n=== Board State ===");
    print_board(&state);
    
    println!("\n=== Expected Winning Moves ===");
    println!("({}, 5) and ({}, 10)", base_row, base_row);
    
    // Check if these positions are actually valid
    println!("\n=== Checking Move Validity ===");
    println!("Is ({}, 5) empty? {}", base_row, state.board.is_empty(base_row, 5));
    println!("Is ({}, 10) empty? {}", base_row, state.board.is_empty(base_row, 10));
    
    // Test if making these moves would actually win
    println!("\n=== Testing Winning Moves ===");
    let mut test_state = state.clone();
    test_state.make_move((base_row, 5));
    println!("After playing ({}, 5): terminal = {}, winner = {:?}", 
             base_row, test_state.is_terminal(), test_state.winner);
    
    let mut test_state2 = state.clone();
    test_state2.make_move((base_row, 10));
    println!("After playing ({}, 10): terminal = {}, winner = {:?}", 
             base_row, test_state2.is_terminal(), test_state2.winner);
    
    // See what the AI actually chooses
    println!("\n=== AI Decision ===");
    let best_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
    println!("AI chose: {:?}", best_move);
    
    if let Some((row, col)) = best_move {
        // Test if the AI's move is actually winning
        let mut test_state3 = state.clone();
        test_state3.make_move((row, col));
        println!("After playing ({}, {}): terminal = {}, winner = {:?}", 
                 row, col, test_state3.is_terminal(), test_state3.winner);
    }
}

fn print_board(state: &GameState) {
    println!("   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4");
    for row in 0..15 {
        print!("{:2} ", row);
        for col in 0..15 {
            let symbol = if state.board.is_empty(row, col) {
                "."
            } else if state.board.get_stone_at(row, col) == Some(Player::Max) {
                "X"
            } else {
                "O"
            };
            print!("{} ", symbol);
        }
        println!();
    }
}
