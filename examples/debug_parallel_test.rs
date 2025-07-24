use gomoku::core::state::GameState;
use gomoku::core::board::Player;
use gomoku::interface::utils::find_best_move_parallel;
use gomoku::ai::transposition::SharedTranspositionTable;

fn main() {
    // Recreate the exact scenario from iteration 1 of the failing test
    let mut state = GameState::new(15, 5);
    let shared_tt = SharedTranspositionTable::new_default();
    
    let base_row = 6 + (1 % 3); // This is 7
    println!("base_row = {}", base_row);
    
    // Place the exact same stones as in the test
    state.board.place_stone(base_row, 6, Player::Max);      // (7, 6)
    state.board.place_stone(base_row, 7, Player::Max);      // (7, 7)
    state.board.place_stone(base_row, 8, Player::Max);      // (7, 8)
    state.board.place_stone(base_row, 9, Player::Max);      // (7, 9)
    
    // Add noise - these are the Min stones
    state.board.place_stone(base_row + 1, 7, Player::Min);  // (8, 7)
    state.board.place_stone(base_row - 1, 8, Player::Min);  // (6, 8)
    
    state.current_player = Player::Max;
    
    // Print the board state
    println!("Board state before move:");
    for row in 5..10 {
        for col in 4..12 {
            match state.board.get_player(row, col) {
                Some(Player::Max) => print!("X "),
                Some(Player::Min) => print!("O "),
                None => print!(". "),
            }
        }
        println!(" <- row {}", row);
    }
    println!("  ^ cols 4-11");
    
    // Find the best move
    let best_move = find_best_move_parallel(&mut state, 6, None, &shared_tt);
    println!("Best move found: {:?}", best_move);
    
    // Check if (6, 5) actually wins
    if let Some((6, 5)) = best_move {
        let mut test_state = state.clone();
        test_state.make_move((6, 5));
        println!("After move (6, 5):");
        for row in 5..10 {
            for col in 4..12 {
                match test_state.board.get_player(row, col) {
                    Some(Player::Max) => print!("X "),
                    Some(Player::Min) => print!("O "),
                    None => print!(". "),
                }
            }
            println!(" <- row {}", row);
        }
        println!("Is terminal: {}", test_state.is_terminal());
        println!("Winner: {:?}", test_state.winner);
    }
    
    // Check the expected winning moves too
    for &(test_row, test_col) in &[(7, 5), (7, 10)] {
        let mut test_state = state.clone();
        test_state.make_move((test_row, test_col));
        println!("After move ({}, {}):", test_row, test_col);
        println!("Is terminal: {}", test_state.is_terminal());
        println!("Winner: {:?}", test_state.winner);
    }
}
