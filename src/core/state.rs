use crate::core::board::{Board, Player};
use crate::core::captures::CaptureHandler;
use crate::core::moves::MoveHandler;
use crate::core::rules::WinChecker;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct GameState {
    pub board: Board,
    pub current_player: Player,
    pub win_condition: usize,
    pub winner: Option<Player>,
    pub max_captures: usize,
    pub min_captures: usize,
    pub capture_history: Vec<Vec<(usize, usize)>>,
}

impl GameState {
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

    pub fn get_possible_moves(&self) -> Vec<(usize, usize)> {
        MoveHandler::get_possible_moves(&self.board, self.current_player)
    }

    pub fn make_move(&mut self, mv: (usize, usize)) {
        self.board.place_stone(mv.0, mv.1, self.current_player);

        let captures =
            CaptureHandler::detect_captures(&self.board, mv.0, mv.1, self.current_player);
        self.execute_captures(captures);

        self.check_for_wins(mv);
        self.switch_player();
    }

    pub fn undo_move(&mut self, move_: (usize, usize)) {
        self.current_player = self.current_player.opponent();

        self.board.remove_stone(move_.0, move_.1);
        self.winner = None;

        self.restore_captured_stones();
    }

    pub fn is_terminal(&self) -> bool {
        self.winner.is_some() || self.get_possible_moves().is_empty()
    }

    pub fn check_winner(&self) -> Option<Player> {
        self.winner
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.board.hash().hash(&mut hasher);
        self.current_player.hash(&mut hasher);
        hasher.finish()
    }

    fn switch_player(&mut self) {
        self.current_player = self.current_player.opponent();
    }

    fn check_for_wins(&mut self, mv: (usize, usize)) -> bool {
        if let Some(winner) = self.check_capture_win() {
            self.winner = Some(winner);
            return true;
        }

        if self.check_win_around(mv) && !self.can_break_five_by_capture(self.current_player) {
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

        CaptureHandler::execute_captures(&mut self.board, &captures);

        let pairs_captured = captures.len() / 2;
        match self.current_player {
            Player::Max => self.min_captures += pairs_captured,
            Player::Min => self.max_captures += pairs_captured,
        }

        self.capture_history.push(captures);
    }

    fn restore_captured_stones(&mut self) {
        if let Some(last_captures) = self.capture_history.pop() {
            if !last_captures.is_empty() {
                let opponent = self.current_player.opponent();

                for &(row, col) in &last_captures {
                    self.board.place_stone(row, col, opponent);
                }

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

    fn check_win_around(&self, mv: (usize, usize)) -> bool {
        WinChecker::check_win_around(&self.board, mv.0, mv.1, self.win_condition)
    }

    pub fn check_capture_win(&self) -> Option<Player> {
        WinChecker::check_capture_win(self.max_captures, self.min_captures)
    }

    fn can_break_five_by_capture(&self, player: Player) -> bool {
        WinChecker::can_break_five_by_capture(&self.board, player, self.win_condition)
    }
}
