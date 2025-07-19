use crate::core::board::{Board, Player};

const DIRECTIONS: [(isize, isize); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];
const FREE_THREE_LENGTH: usize = 3;
const MAX_SEARCH_DISTANCE: isize = 4;

pub struct MoveHandler;

impl MoveHandler {
    pub fn get_possible_moves(board: &Board, player: Player) -> Vec<(usize, usize)> {
        if board.is_empty() {
            return vec![board.center()];
        }

        (0..board.size)
            .flat_map(|i| (0..board.size).map(move |j| (i, j)))
            .filter(|&(i, j)| {
                board.is_empty_position(i, j)
                    && board.is_adjacent_to_stone(i, j)
                    && !RuleValidator::creates_double_three(board, i, j, player)
            })
            .collect()
    }
}

pub struct RuleValidator;

impl RuleValidator {
    pub fn creates_double_three(board: &Board, row: usize, col: usize, player: Player) -> bool {
        DIRECTIONS
            .iter()
            .filter(|&&dir| Self::is_free_three_in_direction(board, row, col, player, dir))
            .count()
            >= 2
    }

    fn is_free_three_in_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        (dr, dc): (isize, isize),
    ) -> bool {
        let (stones, left_open, right_open) = Self::analyze_line(board, row, col, player, dr, dc);

        stones == FREE_THREE_LENGTH && Self::can_form_open_four(left_open, right_open)
    }

    fn analyze_line(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
    ) -> (usize, bool, bool) {
        let left_info = Self::scan_direction(board, row, col, player, -dr, -dc);
        let right_info = Self::scan_direction(board, row, col, player, dr, dc);

        let total_stones = 1 + left_info.0 + right_info.0; // +1 for the current stone
        let left_open = left_info.1;
        let right_open = right_info.1;

        (total_stones, left_open, right_open)
    }

    fn scan_direction(
        board: &Board,
        row: usize,
        col: usize,
        player: Player,
        dr: isize,
        dc: isize,
    ) -> (usize, bool) {
        let mut stones = 0;
        let mut empty_found = false;
        let mut is_open = false;

        for i in 1..=MAX_SEARCH_DISTANCE {
            let (new_row, new_col) = (row as isize + dr * i, col as isize + dc * i);

            if !Self::is_valid_pos(board, new_row, new_col) {
                break;
            }

            match board.get_player(new_row as usize, new_col as usize) {
                Some(p) if p == player => {
                    if empty_found {
                        break; // Gap in stones
                    }
                    stones += 1;
                }
                None => {
                    if !empty_found && stones > 0 {
                        is_open = true;
                    }
                    empty_found = true;
                    if stones > 0 {
                        break; // Only care about first empty space after stones
                    }
                }
                Some(_) => break, // Opponent stone blocks
            }
        }

        (stones, is_open)
    }

    fn can_form_open_four(left_open: bool, right_open: bool) -> bool {
        left_open || right_open
    }

    fn is_valid_pos(board: &Board, row: isize, col: isize) -> bool {
        (0..board.size as isize).contains(&row) && (0..board.size as isize).contains(&col)
    }
}
