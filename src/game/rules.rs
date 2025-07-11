use crate::game::board::{Board, Player};

/// Handles win condition checking
pub struct WinChecker;

impl WinChecker {
    /// Check if there's a win condition around a specific move
    pub fn check_win_around(board: &Board, row: usize, col: usize, win_condition: usize) -> bool {
        let player = board.get_player(row, col).unwrap();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for &(dx, dy) in directions.iter() {
            let mut count = 1; // Count includes the current cell

            // Check in positive direction
            let mut x = row as isize + dx as isize;
            let mut y = col as isize + dy as isize;
            while x >= 0 && y >= 0 && x < board.size as isize && y < board.size as isize {
                if board.get_player(x as usize, y as usize) == Some(player) {
                    count += 1;
                    x += dx as isize;
                    y += dy as isize;
                } else {
                    break;
                }
            }

            // Check in negative direction
            let mut x = row as isize - dx as isize;
            let mut y = col as isize - dy as isize;
            while x >= 0 && y >= 0 && x < board.size as isize && y < board.size as isize {
                if board.get_player(x as usize, y as usize) == Some(player) {
                    count += 1;
                    x -= dx as isize;
                    y -= dy as isize;
                } else {
                    break;
                }
            }

            if count >= win_condition {
                return true;
            }
        }

        false
    }

    /// Check if a player has won by capturing enough stones
    pub fn check_capture_win(max_captures: usize, min_captures: usize) -> Option<Player> {
        if max_captures >= 5 {
            Some(Player::Max)
        } else if min_captures >= 5 {
            Some(Player::Min)
        } else {
            None
        }
    }

    /// Find all five-in-a-row lines for a player
    pub fn find_five_in_a_row_lines(board: &Board, player: Player, win_condition: usize) -> Vec<Vec<(usize, usize)>> {
        let mut lines = Vec::new();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

        for i in 0..board.size {
            for j in 0..board.size {
                if board.get_player(i, j) == Some(player) {
                    for &(dx, dy) in &directions {
                        let line = Self::get_line_from_position(board, i, j, dx, dy, player);
                        if line.len() >= win_condition {
                            lines.push(line);
                        }
                    }
                }
            }
        }

        lines
    }

    /// Get a line of consecutive stones from a starting position
    fn get_line_from_position(board: &Board, start_row: usize, start_col: usize, dx: isize, dy: isize, player: Player) -> Vec<(usize, usize)> {
        let mut line = Vec::new();
        
        // Check if we're at the start of a line (no same-player stone behind us)
        let prev_x = start_row as isize - dx;
        let prev_y = start_col as isize - dy;
        
        if prev_x >= 0 && prev_y >= 0 && 
           prev_x < board.size as isize && prev_y < board.size as isize {
            if board.get_player(prev_x as usize, prev_y as usize) == Some(player) {
                return line; // Not the start of the line
            }
        }

        // Collect consecutive stones in the positive direction
        let mut x = start_row as isize;
        let mut y = start_col as isize;
        
        while x >= 0 && y >= 0 && 
              x < board.size as isize && y < board.size as isize &&
              board.get_player(x as usize, y as usize) == Some(player) {
            line.push((x as usize, y as usize));
            x += dx;
            y += dy;
        }

        line
    }

    /// Check if a five-in-a-row can be broken by capture
    pub fn can_break_five_by_capture(board: &Board, player: Player, win_condition: usize) -> bool {
        let opponent = player.opponent();

        // Find all five-in-a-row lines for the given player
        let five_lines = Self::find_five_in_a_row_lines(board, player, win_condition);
        
        for line in five_lines {
            // Check if opponent can capture any part of this line
            if crate::game::captures::CaptureHandler::can_capture_from_line(board, &line, opponent) {
                return true;
            }
        }

        false
    }

    /// Check if a player is about to lose by capture (has 4 pairs captured already)
    pub fn is_about_to_lose_by_capture(max_captures: usize, min_captures: usize, player: Player) -> bool {
        match player {
            Player::Max => max_captures >= 4,
            Player::Min => min_captures >= 4,
        }
    }

    /// Check if a player can capture to win (opponent has 4 pairs captured and player can capture one more)
    pub fn can_capture_to_win(board: &Board, max_captures: usize, min_captures: usize, player: Player) -> bool {
        let opponent = player.opponent();
        
        // Check if opponent has 4 pairs captured
        if !Self::is_about_to_lose_by_capture(max_captures, min_captures, opponent) {
            return false;
        }

        // Check if current player can make a capture
        for i in 0..board.size {
            for j in 0..board.size {
                if board.is_empty_position(i, j) {
                    // Simulate placing a stone and check if it creates a capture
                    let captures = crate::game::captures::CaptureHandler::detect_captures(board, i, j, player);
                    if !captures.is_empty() {
                        return true;
                    }
                }
            }
        }

        false
    }
}

