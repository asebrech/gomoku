
use crate::core::board::Player;
use rand::Rng;
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZobristHash {
    position_keys: Vec<[u64; 2]>,
    player_key: u64,
    board_size: usize,
}

impl ZobristHash {
    pub fn new(board_size: usize) -> Self {
        let total_positions = board_size * board_size;
        let mut rng = ChaCha8Rng::seed_from_u64(0x123456789ABCDEF0);
        
        let mut position_keys = Vec::with_capacity(total_positions);
        
        for _ in 0..total_positions {
            position_keys.push([rng.random::<u64>(), rng.random::<u64>()]);
        }
        
        let player_key = rng.random::<u64>();
        
        Self {
            position_keys,
            player_key,
            board_size,
        }
    }
    
    pub fn board_size(&self) -> usize {
        self.board_size
    }
    
    #[inline]
    fn position_index(&self, row: usize, col: usize) -> usize {
        row * self.board_size + col
    }
    
    #[inline]
    fn player_index(player: Player) -> usize {
        match player {
            Player::Max => 0,
            Player::Min => 1,
        }
    }
    
    pub fn compute_hash(&self, state: &crate::core::state::GameState) -> u64 {
        let mut hash = 0u64;
        
        for row in 0..self.board_size {
            for col in 0..self.board_size {
                if let Some(player) = state.board.get_player(row, col) {
                    let pos_idx = self.position_index(row, col);
                    let player_idx = Self::player_index(player);
                    hash ^= self.position_keys[pos_idx][player_idx];
                }
            }
        }
        
        if state.current_player == Player::Min {
            hash ^= self.player_key;
        }
        
        hash
    }
    
    pub fn update_hash_make_move(&self, current_hash: u64, row: usize, col: usize, player: Player) -> u64 {
        let pos_idx = self.position_index(row, col);
        let player_idx = Self::player_index(player);
        
        current_hash ^ self.position_keys[pos_idx][player_idx] ^ self.player_key
    }
    
    pub fn update_hash_undo_move(&self, current_hash: u64, row: usize, col: usize, player: Player) -> u64 {
        let pos_idx = self.position_index(row, col);
        let player_idx = Self::player_index(player);
        
        current_hash ^ self.position_keys[pos_idx][player_idx] ^ self.player_key
    }
    
    pub fn update_hash_capture(&self, current_hash: u64, captured_positions: &[(usize, usize)], captured_player: Player) -> u64 {
        let mut hash = current_hash;
        let player_idx = Self::player_index(captured_player);
        
        for &(row, col) in captured_positions {
            let pos_idx = self.position_index(row, col);
            hash ^= self.position_keys[pos_idx][player_idx];
        }
        
        hash
    }
}
