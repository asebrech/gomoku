mod solver {
    pub mod game_state;
    pub mod minimax;
}
mod game {
    pub mod shell_game;
    pub mod utils;
}

use game::shell_game::new_game;

fn main() {
    new_game();
}
