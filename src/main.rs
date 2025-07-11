mod ai {
    pub mod heuristic;
    pub mod minimax;
    pub mod transposition;
}
mod game {
    pub mod game_state;
    pub mod shell_game;
    pub mod utils;
}

use game::shell_game::new_game;

fn main() {
    new_game(8, 4, 4);
}
