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
    pub max_captures: usize,  // Number of pairs captured by Max player
    pub min_captures: usize,  // Number of pairs captured by Min player
    pub capture_history: Vec<Vec<(usize, usize)>>,  // History of captures for undo
}

impl GameState {
    pub fn new(board_size: usize, win_condition: usize) -> Self {
        GameState {
            board: vec![vec![None; board_size]; board_size],
            current_player: Player::Max,
            board_size,
            win_condition,
            winner: None,
            max_captures: 0,
            min_captures: 0,
            capture_history: Vec::new(),
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

        // Check for captures after placing the stone
        let captures = self.detect_captures(mv.0, mv.1, self.current_player);
        self.execute_captures(captures);

        // Check if this move wins by capture (10 stones captured)
        if let Some(winner) = self.check_capture_win() {
            self.winner = Some(winner);
            self.current_player = match self.current_player {
                Player::Max => Player::Min,
                Player::Min => Player::Max,
            };
            return true;
        }

        // Check if this move wins by five-in-a-row
        if self.check_win_around(mv) {
            // Check endgame capture logic: opponent can break this five-in-a-row?
            if !self.can_break_five_by_capture(self.current_player) {
                self.winner = Some(self.current_player);
            }
        }

        // Check if opponent is about to lose by capture (has 4 pairs captured already)
        // and current player can capture one more pair to win
        let opponent = match self.current_player {
            Player::Max => Player::Min,
            Player::Min => Player::Max,
        };
        
        if self.is_about_to_lose_by_capture(opponent) && self.can_capture_to_win(self.current_player) {
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
        // First switch back to the player who made the move being undone
        self.current_player = if self.current_player == Player::Max {
            Player::Min
        } else {
            Player::Max
        };
        
        // Remove the stone from the board
        self.board[move_.0][move_.1] = None;
        self.winner = None;
        
        // Restore captured stones if any
        if let Some(last_captures) = self.capture_history.pop() {
            if !last_captures.is_empty() {
                // Restore the captured stones to the board
                let opponent = match self.current_player {
                    Player::Max => Player::Min,
                    Player::Min => Player::Max,
                };
                for &(row, col) in &last_captures {
                    self.board[row][col] = Some(opponent);
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

    /// Detect captures after placing a stone at the given position
    /// Returns a vector of positions that should be captured
    pub fn detect_captures(&self, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
        let mut captures = Vec::new();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let opponent = match player {
            Player::Max => Player::Min,
            Player::Min => Player::Max,
        };

        for &(dx, dy) in &directions {
            // Check in both positive and negative directions
            for &direction_multiplier in &[1, -1] {
                let actual_dx = dx * direction_multiplier;
                let actual_dy = dy * direction_multiplier;
                
                // Check pattern: [NEW_STONE] -> opponent -> opponent -> player
                let pos1_x = row as isize + actual_dx;
                let pos1_y = col as isize + actual_dy;
                
                // Check if first position is in bounds and has opponent stone
                if pos1_x >= 0 && pos1_y >= 0 && 
                   pos1_x < self.board_size as isize && pos1_y < self.board_size as isize {
                    
                    if self.board[pos1_x as usize][pos1_y as usize] == Some(opponent) {
                        // Check second position
                        let pos2_x = pos1_x + actual_dx;
                        let pos2_y = pos1_y + actual_dy;
                        
                        if pos2_x >= 0 && pos2_y >= 0 && 
                           pos2_x < self.board_size as isize && pos2_y < self.board_size as isize {
                            
                            if self.board[pos2_x as usize][pos2_y as usize] == Some(opponent) {
                                // Check third position (should be our stone)
                                let pos3_x = pos2_x + actual_dx;
                                let pos3_y = pos2_y + actual_dy;
                                
                                if pos3_x >= 0 && pos3_y >= 0 && 
                                   pos3_x < self.board_size as isize && pos3_y < self.board_size as isize {
                                    
                                    if self.board[pos3_x as usize][pos3_y as usize] == Some(player) {
                                        // We have a capture pattern: player - opponent - opponent - player
                                        captures.push((pos1_x as usize, pos1_y as usize));
                                        captures.push((pos2_x as usize, pos2_y as usize));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        captures
    }

    /// Execute captures by removing stones from the board and updating capture counts
    pub fn execute_captures(&mut self, captures: Vec<(usize, usize)>) {
        if captures.is_empty() {
            self.capture_history.push(Vec::new());
            return;
        }

        // Remove captured stones from board
        for &(row, col) in &captures {
            self.board[row][col] = None;
        }

        // Update capture counts (captures come in pairs)
        let pairs_captured = captures.len() / 2;
        match self.current_player {
            Player::Max => self.min_captures += pairs_captured,
            Player::Min => self.max_captures += pairs_captured,
        }

        // Store capture history for undo
        self.capture_history.push(captures);
    }

    /// Check if a player has won by capturing 10 stones (5 pairs)
    pub fn check_capture_win(&self) -> Option<Player> {
        if self.max_captures >= 5 {
            Some(Player::Max)
        } else if self.min_captures >= 5 {
            Some(Player::Min)
        } else {
            None
        }
    }

    /// Check if a five-in-a-row can be broken by capture
    /// This is used for endgame capture logic
    pub fn can_break_five_by_capture(&self, player: Player) -> bool {
        let opponent = match player {
            Player::Max => Player::Min,
            Player::Min => Player::Max,
        };

        // Find all five-in-a-row lines for the given player
        let five_lines = self.find_five_in_a_row_lines(player);
        
        for line in five_lines {
            // Check if opponent can capture any part of this line
            if self.can_capture_from_line(&line, opponent) {
                return true;
            }
        }

        false
    }

    /// Find all five-in-a-row lines for a player
    fn find_five_in_a_row_lines(&self, player: Player) -> Vec<Vec<(usize, usize)>> {
        let mut lines = Vec::new();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for i in 0..self.board_size {
            for j in 0..self.board_size {
                if self.board[i][j] == Some(player) {
                    for &(dx, dy) in &directions {
                        let line = self.get_line_from_position(i, j, dx, dy, player);
                        if line.len() >= 5 {
                            lines.push(line);
                        }
                    }
                }
            }
        }

        lines
    }

    /// Get a line of consecutive stones from a starting position
    fn get_line_from_position(&self, start_row: usize, start_col: usize, dx: isize, dy: isize, player: Player) -> Vec<(usize, usize)> {
        let mut line = Vec::new();
        
        // Check if we're at the start of a line (no same-player stone behind us)
        let prev_x = start_row as isize - dx;
        let prev_y = start_col as isize - dy;
        
        if prev_x >= 0 && prev_y >= 0 && 
           prev_x < self.board_size as isize && prev_y < self.board_size as isize {
            if self.board[prev_x as usize][prev_y as usize] == Some(player) {
                return line; // Not the start of the line
            }
        }

        // Collect consecutive stones in the positive direction
        let mut x = start_row as isize;
        let mut y = start_col as isize;
        
        while x >= 0 && y >= 0 && 
              x < self.board_size as isize && y < self.board_size as isize &&
              self.board[x as usize][y as usize] == Some(player) {
            line.push((x as usize, y as usize));
            x += dx;
            y += dy;
        }

        line
    }

    /// Check if opponent can capture any stone from a line
    fn can_capture_from_line(&self, line: &[(usize, usize)], opponent: Player) -> bool {
        // For each stone in the line, check if it can be captured
        for &(row, col) in line {
            if self.can_stone_be_captured(row, col, opponent) {
                return true;
            }
        }
        false
    }

    /// Check if a stone at the given position can be captured by the opponent
    fn can_stone_be_captured(&self, row: usize, col: usize, opponent: Player) -> bool {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let player = self.board[row][col].unwrap();

        for &(dx, dy) in &directions {
            // Check pattern: opponent - player - player - ?
            // The '?' position should be empty and capturable
            
            // Check backward direction first
            let back_x = row as isize - dx;
            let back_y = col as isize - dy;
            
            if back_x >= 0 && back_y >= 0 && 
               back_x < self.board_size as isize && back_y < self.board_size as isize {
                
                if self.board[back_x as usize][back_y as usize] == Some(opponent) {
                    // Check if there's another player stone next to us
                    let next_x = row as isize + dx;
                    let next_y = col as isize + dy;
                    
                    if next_x >= 0 && next_y >= 0 && 
                       next_x < self.board_size as isize && next_y < self.board_size as isize {
                        
                        if self.board[next_x as usize][next_y as usize] == Some(player) {
                            // Check if opponent can place a stone to complete capture
                            let capture_x = next_x + dx;
                            let capture_y = next_y + dy;
                            
                            if capture_x >= 0 && capture_y >= 0 && 
                               capture_x < self.board_size as isize && capture_y < self.board_size as isize {
                                
                                if self.board[capture_x as usize][capture_y as usize].is_none() {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if a player is about to lose by capture (has 4 pairs captured already)
    pub fn is_about_to_lose_by_capture(&self, player: Player) -> bool {
        match player {
            Player::Max => self.max_captures >= 4,
            Player::Min => self.min_captures >= 4,
        }
    }

    /// Check if a player can capture to win (opponent has 4 pairs captured and player can capture one more)
    pub fn can_capture_to_win(&self, player: Player) -> bool {
        let opponent = match player {
            Player::Max => Player::Min,
            Player::Min => Player::Max,
        };
        
        // Check if opponent has 4 pairs captured
        if !self.is_about_to_lose_by_capture(opponent) {
            return false;
        }

        // Check if current player can make a capture
        for i in 0..self.board_size {
            for j in 0..self.board_size {
                if self.board[i][j].is_none() {
                    // Simulate placing a stone and check if it creates a capture
                    let captures = self.detect_captures(i, j, player);
                    if !captures.is_empty() {
                        return true;
                    }
                }
            }
        }

        false
    }
}
