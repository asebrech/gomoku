use crate::game::board::{Board, Player};

/// Handles capture detection and execution
pub struct CaptureHandler;

impl CaptureHandler {
    /// Detect captures after placing a stone at the given position
    /// Returns a vector of positions that should be captured
    pub fn detect_captures(board: &Board, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
        let mut captures = Vec::new();
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let opponent = player.opponent();

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
                   pos1_x < board.size as isize && pos1_y < board.size as isize {
                    
                    if board.cells[pos1_x as usize][pos1_y as usize] == Some(opponent) {
                        // Check second position
                        let pos2_x = pos1_x + actual_dx;
                        let pos2_y = pos1_y + actual_dy;
                        
                        if pos2_x >= 0 && pos2_y >= 0 && 
                           pos2_x < board.size as isize && pos2_y < board.size as isize {
                            
                            if board.cells[pos2_x as usize][pos2_y as usize] == Some(opponent) {
                                // Check third position (should be our stone)
                                let pos3_x = pos2_x + actual_dx;
                                let pos3_y = pos2_y + actual_dy;
                                
                                if pos3_x >= 0 && pos3_y >= 0 && 
                                   pos3_x < board.size as isize && pos3_y < board.size as isize {
                                    
                                    if board.cells[pos3_x as usize][pos3_y as usize] == Some(player) {
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

    /// Execute captures by removing stones from the board
    pub fn execute_captures(board: &mut Board, captures: &[(usize, usize)]) {
        for &(row, col) in captures {
            board.remove_stone(row, col);
        }
    }

    /// Check if a stone at the given position can be captured by the opponent
    pub fn can_stone_be_captured(board: &Board, row: usize, col: usize, opponent: Player) -> bool {
        let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let player = board.get_player(row, col).unwrap();

        for &(dx, dy) in &directions {
            // Check pattern: opponent - player - player - ?
            // The '?' position should be empty and capturable
            
            // Check backward direction first
            let back_x = row as isize - dx;
            let back_y = col as isize - dy;
            
            if back_x >= 0 && back_y >= 0 && 
               back_x < board.size as isize && back_y < board.size as isize {
                
                if board.cells[back_x as usize][back_y as usize] == Some(opponent) {
                    // Check if there's another player stone next to us
                    let next_x = row as isize + dx;
                    let next_y = col as isize + dy;
                    
                    if next_x >= 0 && next_y >= 0 && 
                       next_x < board.size as isize && next_y < board.size as isize {
                        
                        if board.cells[next_x as usize][next_y as usize] == Some(player) {
                            // Check if opponent can place a stone to complete capture
                            let capture_x = next_x + dx;
                            let capture_y = next_y + dy;
                            
                            if capture_x >= 0 && capture_y >= 0 && 
                               capture_x < board.size as isize && capture_y < board.size as isize {
                                
                                if board.cells[capture_x as usize][capture_y as usize].is_none() {
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

    /// Check if opponent can capture any stone from a line
    pub fn can_capture_from_line(board: &Board, line: &[(usize, usize)], opponent: Player) -> bool {
        // For each stone in the line, check if it can be captured
        for &(row, col) in line {
            if Self::can_stone_be_captured(board, row, col, opponent) {
                return true;
            }
        }
        false
    }
}

