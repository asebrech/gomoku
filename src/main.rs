use interface::shell_game::new_game;

mod ai {
    pub mod heuristic;
    pub mod minimax;
    pub mod transposition;
}
mod core {
    pub mod board;
    pub mod captures;
    pub mod moves;
    pub mod rules;
    pub mod state;
}
mod interface {
    pub mod shell_game;
    pub mod utils;
}

fn main() {
    new_game(8, 4, 4);
}
