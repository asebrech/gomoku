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
                        // Check if this move would create a double-three (forbidden)
                        if !self.creates_double_three(i, j, self.current_player) {
                            moves.push((i, j));
                        }
                    }
                }
            }
        }
        moves
    }

    pub fn make_move(&mut self, mv: (usize, usize)) -> bool {
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

    /// Check if a move would create a free-three pattern
    /// A free-three is a pattern of 3 stones that can become an open-four if not blocked
    pub fn is_free_three(
        &self,
        row: usize,
        col: usize,
        player: Player,
        direction: (isize, isize),
    ) -> bool {
        let (dx, dy) = direction;

        // Check if we can form a free-three in this direction
        // We need to check patterns like _XXX_, _X_XX_, _XX_X_

        // First, let's get the line of 7 positions centered on our move
        let mut line = Vec::new();
        for i in -3..=3 {
            let new_row = row as isize + i * dx;
            let new_col = col as isize + i * dy;

            if new_row >= 0
                && new_row < self.board_size as isize
                && new_col >= 0
                && new_col < self.board_size as isize
            {
                if new_row as usize == row && new_col as usize == col {
                    // This is our hypothetical move
                    line.push(Some(player));
                } else {
                    line.push(self.board[new_row as usize][new_col as usize]);
                }
            } else {
                line.push(Some(Player::Max)); // Treat board edges as blocked
            }
        }

        // Now check if this line contains a free-three pattern
        // A free-three must be able to extend to _XXXX_ pattern
        self.contains_free_three_pattern(&line, player)
    }

    fn contains_free_three_pattern(&self, line: &[Option<Player>], player: Player) -> bool {
        if line.len() < 6 {
            return false;
        }

        // Check all possible positions for a free-three that could become _XXXX_
        for start in 0..=(line.len() - 6) {
            let segment = &line[start..start + 6];

            // Check if this segment can form _XXXX_ pattern
            if self.can_form_open_four(segment, player) {
                return true;
            }
        }

        false
    }

    fn can_form_open_four(&self, segment: &[Option<Player>], player: Player) -> bool {
        if segment.len() != 6 {
            return false;
        }

        // Pattern: _XXXX_ (positions 0 and 5 must be empty, positions 1-4 must be player)
        let mut player_count = 0;
        let mut player_positions = Vec::new();

        // Check if ends are open
        if segment[0].is_some() || segment[5].is_some() {
            return false;
        }

        // Count player stones in middle 4 positions
        for i in 1..5 {
            match segment[i] {
                Some(p) if p == player => {
                    player_count += 1;
                    player_positions.push(i);
                }
                Some(_) => return false, // Opponent stone blocks the pattern
                None => {}               // Empty space
            }
        }

        // For a free-three, we need exactly 3 player stones in the middle 4 positions
        // and they should be able to form a continuous line of 4 with one more move
        if player_count == 3 {
            // Check if the 3 stones can form a line of 4 with one empty space
            let empty_positions: Vec<usize> = (1..5).filter(|&i| segment[i].is_none()).collect();

            if empty_positions.len() == 1 {
                // All 3 stones are consecutive, this is a free-three
                return true;
            }
        }

        false
    }

    /// Check if a move would create a double-three (two free-threes simultaneously)
    pub fn creates_double_three(&self, row: usize, col: usize, player: Player) -> bool {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let mut free_three_count = 0;

        for &direction in &directions {
            if self.is_free_three(row, col, player, direction) {
                free_three_count += 1;
                if free_three_count >= 2 {
                    return true;
                }
            }
        }

        false
    }
}
