use crate::{
    game::utils::{Algorithm, find_best_move},
    solver::game_state::{GameState, Player},
};
use std::io;

fn print_board(board: &Vec<Vec<Option<Player>>>) {
    for row in board {
        for cell in row {
            match cell {
                Some(Player::Max) => print!(" X "),
                Some(Player::Min) => print!(" O "),
                None => print!(" . "),
            }
        }
        println!();
    }
}

pub fn new_game(board_size: usize, winning_condition: usize, depth: i32) {
    let mut state = GameState::new(board_size, winning_condition);
    // Choose sides
    println!("Do you want to play as X (Max) or O (Min)?");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_uppercase();

    let human = if input == "X" {
        Player::Max
    } else {
        Player::Min
    };
    let ai = if human == Player::Max {
        Player::Min
    } else {
        Player::Max
    };

    loop {
        print_board(&state.board);
        if state.is_terminal() {
            match state.check_winner() {
                Some(p) if p == human => println!("🎉 You win!"),
                Some(p) if p == ai => println!("💀 AI wins!"),
                _ => println!("🤝 It's a draw."),
            }
            break;
        }

        if state.current_player == human {
            // Human move
            loop {
                println!("Your move (row and col, e.g. `1 2`): ");
                let mut move_input = String::new();
                io::stdin().read_line(&mut move_input).unwrap();
                let parts: Vec<_> = move_input
                    .split_whitespace()
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect();

                if parts.len() == 2 && parts[0] < board_size && parts[1] < board_size {
                    let mv = (parts[0], parts[1]);
                    if state.board[mv.0][mv.1].is_none() {
                        state.make_move(mv);
                        break;
                    } else {
                        println!("Cell already occupied.");
                    }
                } else {
                    println!("Invalid input. Try again.");
                }
            }
        } else {
            println!("🤖 AI is thinking...");
            if let Some(mv) = find_best_move(&mut state, depth, Algorithm::AlphaBeta) {
                println!("AI chooses: {:?}", mv);
                state.make_move(mv);
            } else {
                println!("AI has no valid moves.");
                break;
            }
        }
    }
}
