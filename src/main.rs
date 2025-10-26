#[cfg(feature = "native")]
use gomoku::ui::app::GomokuApp;

#[cfg(feature = "native")]
fn main() {
	let mut gomoku = GomokuApp::new();
	gomoku.init();
	gomoku.start();
}

#[cfg(not(feature = "native"))]
fn main() {
	eprintln!("This binary requires the 'native' feature to run the Bevy UI.");
	eprintln!("Use the Svelte frontend instead, or build with: cargo build --features native");
	std::process::exit(1);
}

