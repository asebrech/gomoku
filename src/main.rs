use gomoku::ui::app::GomokuApp;

fn main() {
	let mut gomoku = GomokuApp::new();
	gomoku.init();
	gomoku.start();
}

