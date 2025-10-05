/// Precomputed direction and neighbor tables for efficient board traversal
/// This eliminates the need for repeated bounds checking and direction calculations
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DirectionTables {
    board_size: usize,
    /// For each position (flat index), list of adjacent position indices (8-directional)
    pub adjacent_8: Vec<Vec<usize>>,
    /// For each position, rays in 4 main directions: horizontal, vertical, diagonal1, diagonal2
    /// Each ray goes forward in that direction up to max_length positions
    pub rays_forward: Vec<[Vec<usize>; 4]>,
    /// Same as rays_forward but in backward direction
    pub rays_backward: Vec<[Vec<usize>; 4]>,
    /// For each position and direction (4 directions), pairs of positions at distance 1,2,3
    /// Used for capture detection: position -> direction -> [(pos1, pos2, pos3)]
    pub capture_patterns: Vec<[Vec<(usize, usize, usize)>; 4]>,
}

impl DirectionTables {
    /// Create precomputed tables for a board of given size
    /// max_ray_length: maximum length to precompute for rays (typically win_condition + 1)
    pub fn new(board_size: usize, max_ray_length: usize) -> Self {
        let total_cells = board_size * board_size;
        let mut adjacent_8 = vec![Vec::new(); total_cells];
        let mut rays_forward = vec![[Vec::new(), Vec::new(), Vec::new(), Vec::new()]; total_cells];
        let mut rays_backward = vec![[Vec::new(), Vec::new(), Vec::new(), Vec::new()]; total_cells];
        let mut capture_patterns = vec![[Vec::new(), Vec::new(), Vec::new(), Vec::new()]; total_cells];
        
        // Direction vectors: horizontal, vertical, diagonal1, diagonal2
        let dir_4 = [(1, 0), (0, 1), (1, 1), (1, -1)];
        let dir_8 = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
        
        for row in 0..board_size {
            for col in 0..board_size {
                let idx = row * board_size + col;
                
                // Precompute 8-directional neighbors
                for &(dr, dc) in &dir_8 {
                    if let Some(n_idx) = Self::safe_index(row, col, dr, dc, board_size) {
                        adjacent_8[idx].push(n_idx);
                    }
                }
                
                // Precompute rays for each of the 4 main directions
                for (dir_idx, &(dr, dc)) in dir_4.iter().enumerate() {
                    // Forward ray
                    let mut ray_fwd = Vec::new();
                    for step in 1..=max_ray_length {
                        if let Some(n_idx) = Self::safe_index_multi(row, col, dr * step as isize, dc * step as isize, board_size) {
                            ray_fwd.push(n_idx);
                        } else {
                            break;
                        }
                    }
                    rays_forward[idx][dir_idx] = ray_fwd;
                    
                    // Backward ray
                    let mut ray_bwd = Vec::new();
                    for step in 1..=max_ray_length {
                        if let Some(n_idx) = Self::safe_index_multi(row, col, -dr * step as isize, -dc * step as isize, board_size) {
                            ray_bwd.push(n_idx);
                        } else {
                            break;
                        }
                    }
                    rays_backward[idx][dir_idx] = ray_bwd;
                    
                    // Precompute capture patterns (need 3 consecutive positions for OO capture)
                    let mut patterns = Vec::new();
                    for &multiplier in &[1, -1] {
                        let actual_dr = dr * multiplier;
                        let actual_dc = dc * multiplier;
                        
                        if let Some(pos1) = Self::safe_index_multi(row, col, actual_dr, actual_dc, board_size) {
                            if let Some(pos2) = Self::safe_index_multi(row, col, actual_dr * 2, actual_dc * 2, board_size) {
                                if let Some(pos3) = Self::safe_index_multi(row, col, actual_dr * 3, actual_dc * 3, board_size) {
                                    patterns.push((pos1, pos2, pos3));
                                }
                            }
                        }
                    }
                    capture_patterns[idx][dir_idx] = patterns;
                }
            }
        }
        
        Self {
            board_size,
            adjacent_8,
            rays_forward,
            rays_backward,
            capture_patterns,
        }
    }
    
    /// Convert (row, col) to flat index
    #[inline]
    pub fn to_index(&self, row: usize, col: usize) -> usize {
        row * self.board_size + col
    }
    
    /// Convert flat index to (row, col)
    #[inline]
    pub fn to_coords(&self, idx: usize) -> (usize, usize) {
        (idx / self.board_size, idx % self.board_size)
    }
    
    /// Get adjacent positions (8-directional) for a given position
    #[inline]
    pub fn get_adjacent(&self, idx: usize) -> &[usize] {
        &self.adjacent_8[idx]
    }
    
    /// Get forward ray for a position in a given direction (0=horiz, 1=vert, 2=diag1, 3=diag2)
    #[inline]
    pub fn get_ray_forward(&self, idx: usize, direction: usize) -> &[usize] {
        &self.rays_forward[idx][direction]
    }
    
    /// Get backward ray for a position in a given direction
    #[inline]
    pub fn get_ray_backward(&self, idx: usize, direction: usize) -> &[usize] {
        &self.rays_backward[idx][direction]
    }
    
    /// Get capture patterns for a position in a given direction
    #[inline]
    pub fn get_capture_patterns(&self, idx: usize, direction: usize) -> &[(usize, usize, usize)] {
        &self.capture_patterns[idx][direction]
    }
    
    /// Helper: safely compute neighbor index with bounds checking
    fn safe_index(row: usize, col: usize, dr: isize, dc: isize, size: usize) -> Option<usize> {
        let nr = row as isize + dr;
        let nc = col as isize + dc;
        if nr >= 0 && nc >= 0 && nr < size as isize && nc < size as isize {
            Some((nr as usize) * size + (nc as usize))
        } else {
            None
        }
    }
    
    /// Helper: safely compute index with multiple steps
    fn safe_index_multi(row: usize, col: usize, dr: isize, dc: isize, size: usize) -> Option<usize> {
        let nr = row as isize + dr;
        let nc = col as isize + dc;
        if nr >= 0 && nc >= 0 && nr < size as isize && nc < size as isize {
            Some((nr as usize) * size + (nc as usize))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_direction_tables_creation() {
        let tables = DirectionTables::new(19, 6);
        assert_eq!(tables.board_size, 19);
        assert_eq!(tables.adjacent_8.len(), 19 * 19);
    }
    
    #[test]
    fn test_adjacent_center() {
        let tables = DirectionTables::new(5, 6);
        let center_idx = tables.to_index(2, 2);
        let adjacent = tables.get_adjacent(center_idx);
        assert_eq!(adjacent.len(), 8); // Center has all 8 neighbors
    }
    
    #[test]
    fn test_adjacent_corner() {
        let tables = DirectionTables::new(5, 6);
        let corner_idx = tables.to_index(0, 0);
        let adjacent = tables.get_adjacent(corner_idx);
        assert_eq!(adjacent.len(), 3); // Corner has only 3 neighbors
    }
    
    #[test]
    fn test_rays() {
        let tables = DirectionTables::new(10, 5);
        let center_idx = tables.to_index(5, 5);
        
        // Horizontal ray forward (direction 0 is (1, 0) which means move to next row)
        let ray = tables.get_ray_forward(center_idx, 0);
        assert_eq!(ray.len(), 4); // Can go 4 steps from (5,5) on 10x10
        
        // First position should be one step in direction (1, 0) from (5,5) = (6,5)
        assert_eq!(tables.to_coords(ray[0]), (6, 5));
    }
    
    #[test]
    fn test_capture_patterns() {
        let tables = DirectionTables::new(10, 5);
        let idx = tables.to_index(5, 5);
        
        // Check horizontal capture patterns
        let patterns = tables.get_capture_patterns(idx, 0);
        assert!(patterns.len() > 0);
    }
    
    #[test]
    fn test_to_index_to_coords() {
        let tables = DirectionTables::new(19, 6);
        let idx = tables.to_index(10, 15);
        let (row, col) = tables.to_coords(idx);
        assert_eq!(row, 10);
        assert_eq!(col, 15);
    }
}
