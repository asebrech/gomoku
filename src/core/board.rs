// Type alias for bitboard: 16 u64s to handle up to 32x32 = 1024 bits
type Bitboard = [u64; 16];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    Max,
    Min,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::Max => Player::Min,
            Player::Min => Player::Max,
        }
    }
}

//black = true, white = false
// Represents a Gomoku board using bitboards for efficient storage and operations

#[derive(Clone, Debug)]
pub struct Board {
    black: Bitboard,              // Bitboard for black pieces
    white: Bitboard,              // Bitboard for white pieces
    width: usize,                 // Board width
    height: usize,                // Board height
    masks: Vec<Bitboard>,         // Precomputed five-in-a-row masks
    position_masks: Vec<Vec<usize>>, // Masks that include each position
	directions: [(i32, i32); 8],
	possible_moves: std::collections::HashSet<(usize, usize)>, // Cached possible moves as HashSet
	moves_initialized: bool,      // Flag to track if moves have been initialized
}

impl Board {
    // Initialize a new bitboard for given width and height
    pub fn new(size: usize) -> Self {
        let masks = generate_masks(size, size);
        let position_masks = generate_position_masks(&masks, size, size);
        Self {
            black: [0; 16],
            white: [0; 16],
            width: size,
            height: size,
            masks,
            position_masks,
			directions: [
            (-1, 0), (1, 0),   // Vertical
            (0, -1), (0, 1),   // Horizontal
            (-1, -1), (1, 1),  // Diagonal \
            (-1, 1), (1, -1),  // Diagonal /
         	],
			possible_moves: std::collections::HashSet::new(),
			moves_initialized: false,
        }
    }

	
	#[inline(always)]
	pub fn is_empty(&self) -> bool {
		self.black.iter().all(|&x| x == 0) && self.white.iter().all(|&x| x == 0)
	}

	
	#[inline(always)]
	pub fn center(&self) -> (usize, usize) {
		(self.height / 2, self.width / 2)
	}

	// Check if a position is empty
	#[inline(always)]
	pub fn is_empty_position(&self, row: usize, col: usize) -> bool {
		matches!(self.get_piece(row, col), Ok(None))
	}

	// Get player at position (returns Some(true) for black, Some(false) for white, None for empty)
	pub fn get_player(&self, row: usize, col: usize) -> Option<Player> {
		match self.get_piece(row, col) {
			Ok(Some(true)) => Some(Player::Max),  // black -> Player::Max (matches place_stone)
			Ok(Some(false)) => Some(Player::Min), // white -> Player::Min (matches place_stone)
			Ok(None) => None,
			Err(_) => {
				println!("Error getting player at ({}, {}): out of bounds", row, col);
				None
			},
		}
	}


    // Place a piece for a player (true for black, false for white)
	#[inline(always)]
    pub fn place_stone(&mut self, row: usize, col: usize, player: Player) {
        let total_index = self.pos_to_index(row, col);
        let u64_index = total_index / 64;
        let bit_index = total_index % 64;
        let bit = 1u64 << bit_index;

		if (self.black[u64_index] | self.white[u64_index]) & bit != 0 {
			println!("Position already occupied");
			return;
        }

		if player == Player::Max {
			self.black[u64_index] |= bit;
        } else {
			self.white[u64_index] |= bit;
        };
        
        // Incrementally update possible moves
        self.update_moves_after_placement(row, col, player);
    }

    // Remove a piece from the board (for undo operations)
    #[inline(always)]
    pub fn remove_stone(&mut self, row: usize, col: usize) {
        let total_index = self.pos_to_index(row, col);
        let u64_index = total_index / 64;
        let bit_index = total_index % 64;
        let bit = 1u64 << bit_index;

        // Remove from both boards (one will be no-op)
        self.black[u64_index] &= !bit;
        self.white[u64_index] &= !bit;
        
        // Update possible moves after removal
        self.update_moves_after_capture(row, col);
    }

	// Check if a position is adjacent to any existing stone
	pub fn is_adjacent_to_stone(&self, row: usize, col: usize) -> bool {
		for (dr, dc) in &self.directions {
			let new_row = row as i32 + dr;
			let new_col = col as i32 + dc;
			
			if new_row >= 0 && new_row < self.height as i32 && 
			   new_col >= 0 && new_col < self.width as i32 {
				if !self.is_empty_position(new_row as usize, new_col as usize) {
					return true;
				}
			}
		}
		false
	}

    // Convert (row, col) to bit index
	#[inline(always)]
    fn pos_to_index(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }




    // Detect what captures would occur without executing them (for undo tracking)
    pub fn detect_captures_for_move(&self, row: usize, col: usize, player: Player) -> Vec<(usize, usize)> {
        let mut captured_positions = Vec::new();
        
        let (player_board, opponent_board) = if player == Player::Max {
            (&self.black, &self.white)
        } else {
            (&self.white, &self.black)
        };

        for (dr, dc) in self.directions {
            let r1 = row as i32 + dr;
            let c1 = col as i32 + dc;
            let r2 = row as i32 + 2 * dr;
            let c2 = col as i32 + 2 * dc;
            let r3 = row as i32 + 3 * dr;
            let c3 = col as i32 + 3 * dc;

            if r1 >= 0 && r1 < self.height as i32 && c1 >= 0 && c1 < self.width as i32 &&
               r2 >= 0 && r2 < self.height as i32 && c2 >= 0 && c2 < self.width as i32 &&
               r3 >= 0 && r3 < self.height as i32 && c3 >= 0 && c3 < self.width as i32 {
                let idx1 = (r1 as usize) * self.width + (c1 as usize);
                let idx2 = (r2 as usize) * self.width + (c2 as usize);
                let idx3 = (r3 as usize) * self.width + (c3 as usize);

                let u64_idx1 = idx1 / 64;
                let bit_idx1 = idx1 % 64;
                let u64_idx2 = idx2 / 64;
                let bit_idx2 = idx2 % 64;
                let u64_idx3 = idx3 / 64;
                let bit_idx3 = idx3 % 64;

                // Check for pattern: player-opponent-opponent-player
                if (opponent_board[u64_idx1] & (1u64 << bit_idx1)) != 0 &&
                   (opponent_board[u64_idx2] & (1u64 << bit_idx2)) != 0 &&
                   (player_board[u64_idx3] & (1u64 << bit_idx3)) != 0 {
                    // Record what would be captured
                    captured_positions.push((r1 as usize, c1 as usize));
                    captured_positions.push((r2 as usize, c2 as usize));
                }
            }
        }
        
        captured_positions
    }

    // Check for five-in-a-row win condition using precomputed masks
	#[inline(always)]
    pub fn check_win(&self, player: Player, row: usize, col: usize) -> bool {
        let player_board = if player == Player::Max { &self.black } else { &self.white };
        let pos_index = row * self.width + col;

        // Check only masks that include the last move's position
        for &mask_idx in &self.position_masks[pos_index] {
            let mask = &self.masks[mask_idx];
            let mut is_match = true;
            for i in 0..16 {
                if (player_board[i] & mask[i]) != mask[i] {
                    is_match = false;
                    break;
                }
            }
            if is_match {
                return true;
            }
        }

        false
    }

    // Get piece at position (returns None for empty, Some(true) for black, Some(false) for white)
	#[inline(always)]
    pub fn get_piece(&self, row: usize, col: usize) -> Result<Option<bool>, &'static str> {
        let total_index = self.pos_to_index(row, col);
        let u64_index = total_index / 64;
        let bit_index = total_index % 64;
        let bit = 1u64 << bit_index;
        if (self.black[u64_index] & bit) != 0 {
            Ok(Some(true))
        } else if (self.white[u64_index] & bit) != 0 {
            Ok(Some(false))
        } else {
            Ok(None)
        }
    }

	// Get board size (for compatibility with existing code)
	#[inline(always)]
	pub fn size(&self) -> usize {
		self.width.max(self.height)
	}

	#[inline(always)]
	pub fn is_board_full(&self) -> bool {
		// Check if all valid board positions are occupied
		for row in 0..self.height {
			for col in 0..self.width {
				if self.is_empty_position(row, col) {
					return false;
				}
			}
		}
		true
	}

	// Get possible moves for a player (using incremental updates) - returns reference to avoid cloning
	pub fn get_possible_moves(&mut self, player: Player) -> &std::collections::HashSet<(usize, usize)> {
		if !self.moves_initialized {
			self.initialize_possible_moves(player);
		}
		&self.possible_moves
	}

	// Get possible moves as a vector (ONLY use when you need owned data - this is expensive!)
	pub fn get_possible_moves_vec(&mut self, player: Player) -> Vec<(usize, usize)> {
		if !self.moves_initialized {
			self.initialize_possible_moves(player);
		}
		self.possible_moves.iter().cloned().collect()
	}

	// Fast iteration over possible moves without allocation
	pub fn for_each_possible_move<F>(&mut self, player: Player, mut f: F) 
	where 
		F: FnMut((usize, usize))
	{
		if !self.moves_initialized {
			self.initialize_possible_moves(player);
		}
		for &mv in &self.possible_moves {
			f(mv);
		}
	}

	// Get possible moves count without allocation
	pub fn possible_moves_count(&mut self, player: Player) -> usize {
		if !self.moves_initialized {
			self.initialize_possible_moves(player);
		}
		self.possible_moves.len()
	}
	
	// Initialize possible moves (called once for empty board or first time)
	fn initialize_possible_moves(&mut self, player: Player) {
		self.possible_moves.clear();
		
		if self.is_empty() {
			self.possible_moves.insert(self.center());
		} else {
			for row in 0..self.height {
				for col in 0..self.width {
					if self.is_empty_position(row, col)
						&& self.is_adjacent_to_stone(row, col)
						&& !self.creates_double_three(row, col, player)
					{
						self.possible_moves.insert((row, col));
					}
				}
			}
		}
		
		self.moves_initialized = true;
	}

	// Incrementally update moves after placing a piece
	fn update_moves_after_placement(&mut self, row: usize, col: usize, player: Player) {
		// Remove the placed position from possible moves
		self.possible_moves.remove(&(row, col));
		
		// Add new adjacent empty positions as possible moves
		for (dr, dc) in &self.directions {
			let new_row = row as i32 + dr;
			let new_col = col as i32 + dc;
			
			if new_row >= 0 && new_row < self.height as i32 && 
			   new_col >= 0 && new_col < self.width as i32 {
				let adj_row = new_row as usize;
				let adj_col = new_col as usize;
				
				if self.is_empty_position(adj_row, adj_col) 
					&& !self.creates_double_three(adj_row, adj_col, player)
				{
					self.possible_moves.insert((adj_row, adj_col));
				}
			}
		}
		
		// Remove positions that are no longer valid due to double-three rule
		self.revalidate_adjacent_moves(row, col, player);
	}

	// Incrementally update moves after a capture occurs
	fn update_moves_after_capture(&mut self, row: usize, col: usize) {
		// The captured position becomes a valid move again
		self.possible_moves.insert((row, col));
		
		// Check if positions adjacent to the captured piece should be removed
		// (if they're no longer adjacent to any stones)
		for (dr, dc) in &self.directions {
			let new_row = row as i32 + dr;
			let new_col = col as i32 + dc;
			
			if new_row >= 0 && new_row < self.height as i32 && 
			   new_col >= 0 && new_col < self.width as i32 {
				let adj_row = new_row as usize;
				let adj_col = new_col as usize;
				
				if self.is_empty_position(adj_row, adj_col) 
					&& !self.is_adjacent_to_stone(adj_row, adj_col)
				{
					self.possible_moves.remove(&(adj_row, adj_col));
				}
			}
		}
	}

	// Revalidate moves around a position (for double-three rule changes)
	fn revalidate_adjacent_moves(&mut self, row: usize, col: usize, player: Player) {
		let positions_to_check: Vec<(usize, usize)> = self.possible_moves
			.iter()
			.filter(|&&(r, c)| {
				let dr = (r as i32 - row as i32).abs();
				let dc = (c as i32 - col as i32).abs();
				dr <= 2 && dc <= 2 // Check positions within 2 squares
			})
			.cloned()
			.collect();
		
		for (check_row, check_col) in positions_to_check {
			if self.creates_double_three(check_row, check_col, player) {
				self.possible_moves.remove(&(check_row, check_col));
			}
		}
	}

	// Check if placing a piece creates a double three (forbidden in some Gomoku variants)
	pub fn creates_double_three(&self, row: usize, col: usize, player: Player) -> bool {
		const DIRECTIONS: [(i32, i32); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];
		
		DIRECTIONS
			.iter()
			.filter(|&&dir| self.is_free_three_in_direction(row, col, player, dir))
			.count() >= 2
	}

	// Check if there's a free three in a specific direction
	fn is_free_three_in_direction(&self, row: usize, col: usize, player: Player, (dr, dc): (i32, i32)) -> bool {
		let (stones, left_open, right_open) = self.analyze_line(row, col, player, dr, dc);
		stones == 3 && (left_open || right_open)
	}

	// Analyze a line to count stones and check if ends are open
	fn analyze_line(&self, row: usize, col: usize, player: Player, dr: i32, dc: i32) -> (usize, bool, bool) {
		let left_info = self.scan_direction(row, col, player, -dr, -dc);
		let right_info = self.scan_direction(row, col, player, dr, dc);

		let total_stones = 1 + left_info.0 + right_info.0; // +1 for the current stone
		let left_open = left_info.1;
		let right_open = right_info.1;

		(total_stones, left_open, right_open)
	}

	// Scan in a direction to count consecutive stones and check if open
	fn scan_direction(&self, row: usize, col: usize, player: Player, dr: i32, dc: i32) -> (usize, bool) {
		const MAX_SEARCH_DISTANCE: i32 = 4;
		let mut stones = 0;
		let mut empty_found = false;
		let mut is_open = false;

		for i in 1..=MAX_SEARCH_DISTANCE {
			let new_row = row as i32 + dr * i;
			let new_col = col as i32 + dc * i;

			if new_row < 0 || new_row >= self.height as i32 || 
			   new_col < 0 || new_col >= self.width as i32 {
				break;
			}

			match self.get_player(new_row as usize, new_col as usize) {
				Some(p) if p == player => {
					stones += 1;
				},
				Some(_) => {
					if empty_found {
						break; // Gap in stones
					}
				},
				None => {
					if !empty_found {
						empty_found = true;
						is_open = true; // Found an open end
					} else {
						break; // Second empty found, stop scanning
					}
				},
			}
		}

		(stones, is_open)
	}


	// Simple hash function for the board (not for transposition table)
	pub fn hash(&self) -> u64 {
		let mut hash = 0u64;
		
		// Hash the board dimensions first
		hash ^= (self.width as u64).wrapping_mul(0x9e3779b7);
		hash ^= (self.height as u64).wrapping_mul(0x9e3779bb);
		
		// Hash the bitboard contents
		for i in 0..16 {
			hash ^= self.black[i].wrapping_mul(0x9e3779b9);
			hash ^= self.white[i].wrapping_mul(0x9e3779b1);
		}
		
		hash
	}
}

// Implement the BoardHashable trait for the transposition table
impl crate::ai::transposition::BoardHashable for Board {
    fn width(&self) -> usize {
        self.width
    }
    
    fn height(&self) -> usize {
        self.height
    }
    
    fn get_player(&self, row: usize, col: usize) -> Option<Player> {
        self.get_player(row, col)
    }
}

// Generate precomputed masks for five-in-a-row
fn generate_masks(width: usize, height: usize) -> Vec<Bitboard> {
    let mut masks = Vec::new();

    // Horizontal masks
    for row in 0..height {
        for col in 0..=width.saturating_sub(5) {
            let mut mask = [0u64; 16];
            for i in 0..5 {
                let total_index = row * width + col + i;
                let u64_index = total_index / 64;
                let bit_index = total_index % 64;
                mask[u64_index] |= 1u64 << bit_index;
            }
            masks.push(mask);
        }
    }

    // Vertical masks
    for col in 0..width {
        for row in 0..=height.saturating_sub(5) {
            let mut mask = [0u64; 16];
            for i in 0..5 {
                let total_index = (row + i) * width + col;
                let u64_index = total_index / 64;
                let bit_index = total_index % 64;
                mask[u64_index] |= 1u64 << bit_index;
            }
            masks.push(mask);
        }
    }

    // Diagonal \ masks
    for row in 0..=height.saturating_sub(5) {
        for col in 0..=width.saturating_sub(5) {
            let mut mask = [0u64; 16];
            for i in 0..5 {
                let total_index = (row + i) * width + col + i;
                let u64_index = total_index / 64;
                let bit_index = total_index % 64;
                mask[u64_index] |= 1u64 << bit_index;
            }
            masks.push(mask);
        }
    }

    // Diagonal / masks
    for row in 0..=height.saturating_sub(5) {
        for col in (4..width).rev() {
            let mut mask = [0u64; 16];
            for i in 0..5 {
                let total_index = (row + i) * width + col - i;
                let u64_index = total_index / 64;
                let bit_index = total_index % 64;
                mask[u64_index] |= 1u64 << bit_index;
            }
            masks.push(mask);
        }
    }

    masks
}

// Generate position-specific mask indices
fn generate_position_masks(masks: &[Bitboard], width: usize, height: usize) -> Vec<Vec<usize>> {
    let total_positions = width * height;
    let mut position_masks = vec![Vec::new(); total_positions];

    for (mask_idx, mask) in masks.iter().enumerate() {
        for row in 0..height {
            for col in 0..width {
                let total_index = row * width + col;
                let u64_index = total_index / 64;
                let bit_index = total_index % 64;
                if (mask[u64_index] & (1u64 << bit_index)) != 0 {
                    position_masks[total_index].push(mask_idx);
                }
            }
        }
    }

    position_masks
}
