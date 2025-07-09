use crate::{
    game::utils::find_best_move,
    solver::game_state::{GameState, Player},
};
use std::io;

fn print_board(board: &[[Option<Player>; 4]; 4]) {
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

pub fn new_game() {
    let mut state = GameState {
        board: [[None; 4]; 4],
        current_player: Player::Max,
    };

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
    let depth = 4;

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
                    .trim()
                    .split_whitespace()
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect();

                if parts.len() == 2 && parts[0] < 4 && parts[1] < 4 {
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
            if let Some(mv) = find_best_move(&mut state, depth) {
                println!("AI chooses: {:?}", mv);
                state.make_move(mv);
            } else {
                println!("AI has no valid moves.");
                break;
            }
        }
    }
}
