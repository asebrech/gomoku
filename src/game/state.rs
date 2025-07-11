use std::hash::{DefaultHasher, Hash, Hasher};
use crate::game::board::{Board, Player};
use crate::game::captures::CaptureHandler;
use crate::game::moves::MoveHandler;
use crate::game::rules::WinChecker;

/// Represents the complete game state
pub struct GameState {
    pub board: Board,
    pub current_player: Player,
    pub win_condition: usize,
    pub winner: Option<Player>,
    pub max_captures: usize,  // Number of pairs captured by Max player
    pub min_captures: usize,  // Number of pairs captured by Min player
    pub capture_history: Vec<Vec<(usize, usize)>>,  // History of captures for undo
}

impl GameState {
    // === CREATION AND INITIALIZATION ===
    
    /// Create a new game state
    pub fn new(board_size: usize, win_condition: usize) -> Self {
        GameState {
            board: Board::new(board_size),
            current_player: Player::Max,
            win_condition,
            winner: None,
            max_captures: 0,
            min_captures: 0,
            capture_history: Vec::new(),
        }
    }

    // === CORE GAME MECHANICS ===
    
    /// Get all possible moves for the current player
    pub fn get_possible_moves(&self) -> Vec<(usize, usize)> {
        MoveHandler::get_possible_moves(&self.board, self.current_player)
    }

    /// Make a move and update the game state
    pub fn make_move(&mut self, mv: (usize, usize)) -> bool {
        // Place the stone
        self.board.place_stone(mv.0, mv.1, self.current_player);

        // Handle captures
        let captures = CaptureHandler::detect_captures(&self.board, mv.0, mv.1, self.current_player);
        self.execute_captures(captures);

        // Check for wins
        if self.check_for_wins(mv) {
            self.switch_player();
            return true;
        }

        self.switch_player();
        true
    }

    /// Undo a move and restore the previous game state
    pub fn undo_move(&mut self, move_: (usize, usize)) {
        // Switch back to the player who made the move being undone
        self.current_player = self.current_player.opponent();
        
        // Remove the stone from the board
        self.board.remove_stone(move_.0, move_.1);
        self.winner = None;
        
        // Restore captured stones if any
        self.restore_captured_stones();
    }

    // === GAME STATE QUERIES ===
    
    /// Check if the game is over
    pub fn is_terminal(&self) -> bool {
        self.winner.is_some() || self.get_possible_moves().is_empty()
    }

    /// Get the winner of the game
    pub fn check_winner(&self) -> Option<Player> {
        self.winner
    }

    /// Generate a hash for the current game state
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.board.hash().hash(&mut hasher);
        self.current_player.hash(&mut hasher);
        hasher.finish()
    }

    // === PRIVATE HELPER METHODS ===
    
    fn switch_player(&mut self) {
        self.current_player = self.current_player.opponent();
    }

    fn check_for_wins(&mut self, mv: (usize, usize)) -> bool {
        // Check if this move wins by capture (10 stones captured)
        if let Some(winner) = self.check_capture_win() {
            self.winner = Some(winner);
            return true;
        }

        // Check if this move wins by five-in-a-row
        if self.check_win_around(mv) {
            // Check endgame capture logic: opponent can break this five-in-a-row?
            if !self.can_break_five_by_capture(self.current_player) {
                self.winner = Some(self.current_player);
                return true;
            }
        }

        // Check if opponent is about to lose by capture and current player can capture to win
        let opponent = self.current_player.opponent();
        if self.is_about_to_lose_by_capture(opponent) && self.can_capture_to_win(self.current_player) {
            self.winner = Some(self.current_player);
            return true;
        }

        false
    }

    fn execute_captures(&mut self, captures: Vec<(usize, usize)>) {
        if captures.is_empty() {
            self.capture_history.push(Vec::new());
            return;
        }

        // Remove captured stones from board
        CaptureHandler::execute_captures(&mut self.board, &captures);

        // Update capture counts (captures come in pairs)
        let pairs_captured = captures.len() / 2;
        match self.current_player {
            Player::Max => self.min_captures += pairs_captured,
            Player::Min => self.max_captures += pairs_captured,
        }

        // Store capture history for undo
        self.capture_history.push(captures);
    }

    fn restore_captured_stones(&mut self) {
        if let Some(last_captures) = self.capture_history.pop() {
            if !last_captures.is_empty() {
                let opponent = self.current_player.opponent();
                
                // Restore the captured stones to the board
                for &(row, col) in &last_captures {
                    self.board.place_stone(row, col, opponent);
                }
                
                // Update capture counts (subtract the pairs that were captured)
                let pairs_captured = last_captures.len() / 2;
                match self.current_player {
                    Player::Max => {
                        if self.min_captures >= pairs_captured {
                            self.min_captures -= pairs_captured;
                        }
                    }
                    Player::Min => {
                        if self.max_captures >= pairs_captured {
                            self.max_captures -= pairs_captured;
                        }
                    }
                }
            }
        }
    }

    // === WIN CONDITION CHECKS ===
    
    fn check_win_around(&self, mv: (usize, usize)) -> bool {
        WinChecker::check_win_around(&self.board, mv.0, mv.1, self.win_condition)
    }

    pub fn check_capture_win(&self) -> Option<Player> {
        WinChecker::check_capture_win(self.max_captures, self.min_captures)
    }

    fn can_break_five_by_capture(&self, player: Player) -> bool {
        WinChecker::can_break_five_by_capture(&self.board, player, self.win_condition)
    }

    fn is_about_to_lose_by_capture(&self, player: Player) -> bool {
        WinChecker::is_about_to_lose_by_capture(self.max_captures, self.min_captures, player)
    }

    fn can_capture_to_win(&self, player: Player) -> bool {
        WinChecker::can_capture_to_win(&self.board, self.max_captures, self.min_captures, player)
    }

}

