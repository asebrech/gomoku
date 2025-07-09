#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player {
    Max,
    Min,
}

pub struct GameState {
    pub board: Vec<Vec<Option<Player>>>,
    pub current_player: Player,
    pub board_size: usize,
    pub win_condition: usize,
}

impl GameState {
    pub fn new(board_size: usize, win_condition: usize) -> Self {
        GameState {
            board: vec![vec![None; board_size]; board_size],
            current_player: Player::Max,
            board_size,
            win_condition,
        }
    }

    pub fn get_possible_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for i in 0..self.board_size {
            for j in 0..self.board_size {
                if self.board[i][j].is_none() {
                    moves.push((i, j));
                }
            }
        }
        moves
    }

    pub fn make_move(&mut self, move_: (usize, usize)) {
        self.board[move_.0][move_.1] = Some(self.current_player);
        self.current_player = if self.current_player == Player::Max {
            Player::Min
        } else {
            Player::Max
        };
    }

    pub fn undo_move(&mut self, move_: (usize, usize)) {
        self.board[move_.0][move_.1] = None;
        self.current_player = if self.current_player == Player::Max {
            Player::Min
        } else {
            Player::Max
        };
    }

    pub fn is_terminal(&self) -> bool {
        self.check_winner().is_some() || self.get_possible_moves().is_empty()
    }

    pub fn check_winner(&self) -> Option<Player> {
        let b = &self.board;

        // Check rows
        for i in 0..self.board_size {
            for j in 0..self.board_size - self.win_condition + 1 {
                if let Some(player) = b[i][j] {
                    if (0..self.win_condition).all(|k| b[i][j + k] == Some(player)) {
                        return Some(player);
                    }
                }
            }
        }

        // Check columns
        for j in 0..self.board_size {
            for i in 0..self.board_size - self.win_condition + 1 {
                if let Some(player) = b[i][j] {
                    if (0..self.win_condition).all(|k| b[i + k][j] == Some(player)) {
                        return Some(player);
                    }
                }
            }
        }

        // Check main diagonals
        for i in 0..self.board_size - self.win_condition + 1 {
            for j in 0..self.board_size - self.win_condition + 1 {
                if let Some(player) = b[i][j] {
                    if (0..self.win_condition).all(|k| b[i + k][j + k] == Some(player)) {
                        return Some(player);
                    }
                }
            }
        }

        // Check anti-diagonals
        for i in 0..self.board_size - self.win_condition + 1 {
            for j in (self.win_condition - 1)..self.board_size {
                if let Some(player) = b[i][j] {
                    if (0..self.win_condition).all(|k| b[i + k][j - k] == Some(player)) {
                        return Some(player);
                    }
                }
            }
        }

        None
    }

    // Evaluate the board state for a player
    pub fn evaluate(&self) -> i32 {
        match self.check_winner() {
            Some(Player::Max) => 1000,
            Some(Player::Min) => -1000,
            None => 0,
        }
    }
}
