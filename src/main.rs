use crate::ui::app::GomokuApp;
mod ui;

fn main() {
	let mut gomoku = GomokuApp::new();
	gomoku.init();
	gomoku.start();
}

