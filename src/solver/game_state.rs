use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    Max,
    Min,
}

pub struct GameState {
    pub board: Vec<Vec<Option<Player>>>,
    pub current_player: Player,
    pub board_size: usize,
    pub win_condition: usize,
    pub winner: Option<Player>,
}

impl GameState {
    pub fn new(board_size: usize, win_condition: usize) -> Self {
        GameState {
            board: vec![vec![None; board_size]; board_size],
            current_player: Player::Max,
            board_size,
            win_condition,
            winner: None,
        }
    }

    pub fn get_possible_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for i in 0..self.board.len() {
            for j in 0..self.board.len() {
                if self.board[i][j].is_none() {
                    if self.is_board_empty()
                        && (i, j) == (self.board.len() / 2, self.board.len() / 2)
                    {
                        moves.push((i, j));
                    } else if !self.is_board_empty() && self.is_move_adjacent((i, j)) {
                        moves.push((i, j));
                    }
                }
            }
        }
        moves
    }

    pub fn make_move(&mut self, mv: (usize, usize)) -> bool {
        if self.is_board_empty() {
            let center = self.board.len() / 2;
            if mv != (center, center) {
                return false;
            }
        } else if !self.is_move_adjacent(mv) {
            return false;
        }

        self.board[mv.0][mv.1] = Some(self.current_player);

        // Check if this move wins the game
        if self.check_win_around(mv) {
            self.winner = Some(self.current_player);
        }

        self.current_player = match self.current_player {
            Player::Max => Player::Min,
            Player::Min => Player::Max,
        };
        true
    }

    pub fn is_board_empty(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|cell| cell.is_none()))
    }

    pub fn is_move_adjacent(&self, mv: (usize, usize)) -> bool {
        let (i, j) = mv;
        let n = self.board.len();

        let dirs = [-1, 0, 1];

        for di in dirs {
            for dj in dirs {
                if di == 0 && dj == 0 {
                    continue;
                }
                let ni = i as isize + di;
                let nj = j as isize + dj;

                if ni >= 0 && nj >= 0 && ni < n as isize && nj < n as isize {
                    if self.board[ni as usize][nj as usize].is_some() {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn undo_move(&mut self, move_: (usize, usize)) {
        self.board[move_.0][move_.1] = None;
        self.winner = None;
        self.current_player = if self.current_player == Player::Max {
            Player::Min
        } else {
            Player::Max
        };
    }

    pub fn is_terminal(&self) -> bool {
        self.winner.is_some() || self.get_possible_moves().is_empty()
    }

    pub fn check_winner(&self) -> Option<Player> {
        self.winner
    }

    fn check_win_around(&self, mv: (usize, usize)) -> bool {
        let (i, j) = mv;
        let player = self.board[i][j].unwrap();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for &(dx, dy) in directions.iter() {
            let mut count = 1; // Count includes the current cell

            // Check in positive direction
            let mut x = i as isize + dx as isize;
            let mut y = j as isize + dy as isize;
            while x >= 0 && y >= 0 && x < self.board_size as isize && y < self.board_size as isize {
                if self.board[x as usize][y as usize] == Some(player) {
                    count += 1;
                    x += dx as isize;
                    y += dy as isize;
                } else {
                    break;
                }
            }

            // Check in negative direction
            let mut x = i as isize - dx as isize;
            let mut y = j as isize - dy as isize;
            while x >= 0 && y >= 0 && x < self.board_size as isize && y < self.board_size as isize {
                if self.board[x as usize][y as usize] == Some(player) {
                    count += 1;
                    x -= dx as isize;
                    y -= dy as isize;
                } else {
                    break;
                }
            }

            if count >= self.win_condition {
                return true;
            }
        }

        false
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for row in &self.board {
            for cell in row {
                cell.hash(&mut hasher);
            }
        }
        self.current_player.hash(&mut hasher);
        hasher.finish()
    }
}
