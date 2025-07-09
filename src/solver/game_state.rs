#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Player {
    Max,
    Min,
}

pub struct GameState {
    pub board: [[Option<Player>; 4]; 4],
    pub current_player: Player,
}

impl GameState {
    pub fn get_possible_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j].is_none() {
                    moves.push((i, j));
                }
            }
        }
        moves
    }

    pub fn make_move(&mut self, move_: (usize, usize)) {
        self.board[move_.0][move_.1] = Some(self.current_player.clone());
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
        for i in 0..4 {
            if let Some(player) = b[i][0] {
                if b[i][1] == Some(player) && b[i][2] == Some(player) && b[i][3] == Some(player) {
                    return Some(player);
                }
            }
        }

        // Check columns
        for j in 0..4 {
            if let Some(player) = b[0][j] {
                if b[1][j] == Some(player) && b[2][j] == Some(player) && b[3][j] == Some(player) {
                    return Some(player);
                }
            }
        }

        // Check main diagonal
        if let Some(player) = b[0][0] {
            if b[1][1] == Some(player) && b[2][2] == Some(player) && b[3][3] == Some(player) {
                return Some(player);
            }
        }

        // Check anti-diagonal
        if let Some(player) = b[0][3] {
            if b[1][2] == Some(player) && b[2][1] == Some(player) && b[3][0] == Some(player) {
                return Some(player);
            }
        }

        None
    }

    pub fn evaluate(&self) -> i32 {
        match self.check_winner() {
            Some(Player::Max) => 1000,
            Some(Player::Min) => -1000,
            None => 0,
        }
    }
}
