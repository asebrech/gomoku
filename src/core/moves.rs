use crate::core::board::{Board, Player};

pub struct MoveHandler;

impl MoveHandler {
    pub fn get_possible_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();

        for i in 0..board.size {
            for j in 0..board.size {
                if board.is_empty_position(i, j) {
                    if board.is_empty() {
                        let center = board.center();
                        if (i, j) == center {
                            moves.push((i, j));
                        }
                    } else if board.is_adjacent_to_stone(i, j) {
                        if !RuleValidator::creates_double_three(board, i, j, player) {
                            moves.push((i, j));
                        }
                    }
                }
            }
        }

        moves
    }
}

pub struct RuleValidator;

impl RuleValidator {
    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
        // Temporarily place the stone to test
        let mut test_board = board.clone();
        test_board.place_stone(row, col, player);

        let free_threes_count = Self::count_free_threes(&test_board, row, col, player);

        free_threes_count >= 2
    }

    fn count_free_threes(board: &Board, row: usize, col: usize, player: Player) -> usize {
        let directions = [
            (0, 1),  // horizontal
            (1, 0),  // vertical
            (1, 1),  // diagonal /
            (1, -1), // diagonal \
        ];

        let mut free_three_count = 0;

        for &(dr, dc) in &directions {
            if Self::is_free_three_in_direction(board, row, col, player, dr, dc) {
                free_three_count += 1;
            }
        }

        free_three_count
    }

    fn is_free_three_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
    ) -> bool {
        // Count stones in both directions from the placed stone
        let mut stones_count = 1; // Include the stone we just placed
        let mut left_open = false;
        let mut right_open = false;

        // Count stones in the negative direction
        let mut left_empty_spaces = 0;
        for i in 1..=4 {
            let new_row = row as isize - (dr * i);
            let new_col = col as isize - (dc * i);

            if !Self::is_valid_position(board, new_row, new_col) {
                break;
            }

            let pos_row = new_row as usize;
            let pos_col = new_col as usize;

            match board.get_player(pos_row, pos_col) {
                Some(p) if p == player => {
                    if left_empty_spaces == 0 {
                        stones_count += 1;
                    } else {
                        break; // Gap in stones
                    }
                }
                None => {
                    left_empty_spaces += 1;
                    if stones_count == 3 && left_empty_spaces == 1 {
                        left_open = true;
                    }
                    if left_empty_spaces >= 2 {
                        break;
                    }
                }
                Some(_) => break, // Opponent stone
            }
        }

        // Count stones in the positive direction
        let mut right_empty_spaces = 0;
        for i in 1..=4 {
            let new_row = row as isize + (dr * i);
            let new_col = col as isize + (dc * i);

            if !Self::is_valid_position(board, new_row, new_col) {
                break;
            }

            let pos_row = new_row as usize;
            let pos_col = new_col as usize;

            match board.get_player(pos_row, pos_col) {
                Some(p) if p == player => {
                    if right_empty_spaces == 0 {
                        stones_count += 1;
                    } else {
                        break; // Gap in stones
                    }
                }
                None => {
                    right_empty_spaces += 1;
                    if stones_count == 3 && right_empty_spaces == 1 {
                        right_open = true;
                    }
                    if right_empty_spaces >= 2 {
                        break;
                    }
                }
                Some(_) => break, // Opponent stone
            }
        }

        // Check if we have exactly 3 stones and at least one open end
        // that can lead to an undefendable four (both ends open)
        stones_count == 3
            && Self::can_create_open_four(board, row, col, player, dr, dc, left_open, right_open)
    }

    fn can_create_open_four(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
        left_open: bool,
        right_open: bool,
    ) -> bool {
        // A free-three can create an "undefendable alignment of four stones
        // with two unobstructed extremities"

        // Check if adding one stone to either end would create an open four
        if left_open {
            // Check if there's space for open four on the left
            let left_pos = (row as isize - dr, col as isize - dc);
            if Self::is_valid_position(board, left_pos.0, left_pos.1) {
                // Check if placing here would create potential for open four
                let far_left = (row as isize - dr * 2, col as isize - dc * 2);
                let far_right = (row as isize + dr * 4, col as isize + dc * 4);

                if Self::is_valid_position(board, far_left.0, far_left.1)
                    && Self::is_valid_position(board, far_right.0, far_right.1)
                    && board.is_empty_position(far_left.0 as usize, far_left.1 as usize)
                    && board.is_empty_position(far_right.0 as usize, far_right.1 as usize)
                {
                    return true;
                }
            }
        }

        if right_open {
            // Check if there's space for open four on the right
            let right_pos = (row as isize + dr, col as isize + dc);
            if Self::is_valid_position(board, right_pos.0, right_pos.1) {
                // Check if placing here would create potential for open four
                let far_left = (row as isize - dr * 4, col as isize - dc * 4);
                let far_right = (row as isize + dr * 2, col as isize + dc * 2);

                if Self::is_valid_position(board, far_left.0, far_left.1)
                    && Self::is_valid_position(board, far_right.0, far_right.1)
                    && board.is_empty_position(far_left.0 as usize, far_left.1 as usize)
                    && board.is_empty_position(far_right.0 as usize, far_right.1 as usize)
                {
                    return true;
                }
            }
        }

        false
    }

    fn is_valid_position(board: &Board, row: isize, col: isize) -> bool {
        row >= 0 && col >= 0 && row < board.size as isize && col < board.size as isize
    }
}
