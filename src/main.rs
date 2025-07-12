use crate::ui::app::GomokuApp;
use gomoku::interface::shell_game::new_game;
mod ui;

fn main() {
	let mut gomoku = GomokuApp::new();
	gomoku.init();
	gomoku.start();
	new_game(8, 4, 4);
}

