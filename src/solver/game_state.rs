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

    pub fn evaluate(&self) -> i32 {
        // Terminal state evaluation: check for wins or draws
        if let Some(winner) = self.check_winner() {
            return match winner {
                Player::Max => 1_000_000,  // Max wins
                Player::Min => -1_000_000, // Min wins
            };
        }
        if self.get_possible_moves().is_empty() {
            return 0; // Draw
        }

        let mut total_score = 0;
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for i in 0..self.board_size {
            for j in 0..self.board_size {
                if let Some(player) = self.board[i][j] {
                    for &(dx, dy) in directions.iter() {
                        // Check if current cell is start of a run
                        let prev_i = i as isize - dx as isize;
                        let prev_j = j as isize - dy as isize;

                        // Skip if previous cell in run exists
                        if prev_i >= 0
                            && prev_j >= 0
                            && prev_i < self.board_size as isize
                            && prev_j < self.board_size as isize
                        {
                            if let Some(prev_player) = self.board[prev_i as usize][prev_j as usize]
                            {
                                if prev_player == player {
                                    continue;
                                }
                            }
                        }

                        // Count consecutive stones in direction
                        let mut count = 1;
                        let mut cur_i = i as isize + dx as isize;
                        let mut cur_j = j as isize + dy as isize;

                        while cur_i >= 0
                            && cur_j >= 0
                            && cur_i < self.board_size as isize
                            && cur_j < self.board_size as isize
                        {
                            match self.board[cur_i as usize][cur_j as usize] {
                                Some(cell_player) if cell_player == player => {
                                    count += 1;
                                    cur_i += dx as isize;
                                    cur_j += dy as isize;
                                }
                                _ => break,
                            }
                        }

                        // Only consider runs of length 2 or more
                        if count < 2 {
                            continue;
                        }

                        // Skip runs that meet/exceed win condition (should be terminal)
                        if count >= self.win_condition {
                            continue;
                        }

                        // Check start end (behind the run)
                        let start_open = if prev_i >= 0
                            && prev_j >= 0
                            && prev_i < self.board_size as isize
                            && prev_j < self.board_size as isize
                        {
                            self.board[prev_i as usize][prev_j as usize].is_none()
                        } else {
                            false // Blocked by board edge
                        };

                        // Check end end (after the run)
                        let end_open = if cur_i >= 0
                            && cur_j >= 0
                            && cur_i < self.board_size as isize
                            && cur_j < self.board_size as isize
                        {
                            self.board[cur_i as usize][cur_j as usize].is_none()
                        } else {
                            false // Blocked by board edge
                        };

                        // Calculate run score based on type
                        let score = match (start_open, end_open) {
                            (true, true) => 2 * 10i32.pow(count as u32 - 1), // Open run
                            (true, false) | (false, true) => 10i32.pow(count as u32 - 1), // Semi-open
                            _ => 0, // Closed run (no score)
                        };

                        // Add score for Max, subtract for Min
                        match player {
                            Player::Max => total_score += score,
                            Player::Min => total_score -= score,
                        }
                    }
                }
            }
        }

        total_score
    }
}
