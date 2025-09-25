use crate::core::board::Board;
use std::collections::HashSet;

const CACHE_ZONE_SIZE: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CachedPatternCounts {
    pub five_in_row: u8,
    pub live_four: u8,
    pub half_free_four: u8,
    pub dead_four: u8,
    pub live_three: u8,
    pub half_free_three: u8,
    pub dead_three: u8,
    pub live_two: u8,
    pub half_free_two: u8,
}

impl CachedPatternCounts {
    pub const fn new() -> Self {
        Self {
            five_in_row: 0,
            live_four: 0,
            half_free_four: 0,
            dead_four: 0,
            live_three: 0,
            half_free_three: 0,
            dead_three: 0,
            live_two: 0,
            half_free_two: 0,
        }
    }

    pub fn add(&mut self, other: &CachedPatternCounts) {
        self.five_in_row = self.five_in_row.saturating_add(other.five_in_row);
        self.live_four = self.live_four.saturating_add(other.live_four);
        self.half_free_four = self.half_free_four.saturating_add(other.half_free_four);
        self.dead_four = self.dead_four.saturating_add(other.dead_four);
        self.live_three = self.live_three.saturating_add(other.live_three);
        self.half_free_three = self.half_free_three.saturating_add(other.half_free_three);
        self.dead_three = self.dead_three.saturating_add(other.dead_three);
        self.live_two = self.live_two.saturating_add(other.live_two);
        self.half_free_two = self.half_free_two.saturating_add(other.half_free_two);
    }

    pub fn subtract(&mut self, other: &CachedPatternCounts) {
        self.five_in_row = self.five_in_row.saturating_sub(other.five_in_row);
        self.live_four = self.live_four.saturating_sub(other.live_four);
        self.half_free_four = self.half_free_four.saturating_sub(other.half_free_four);
        self.dead_four = self.dead_four.saturating_sub(other.dead_four);
        self.live_three = self.live_three.saturating_sub(other.live_three);
        self.half_free_three = self.half_free_three.saturating_sub(other.half_free_three);
        self.dead_three = self.dead_three.saturating_sub(other.dead_three);
        self.live_two = self.live_two.saturating_sub(other.live_two);
        self.half_free_two = self.half_free_two.saturating_sub(other.half_free_two);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZoneCoord {
    pub zone_row: usize,
    pub zone_col: usize,
}

#[derive(Debug, Clone)]
pub struct ZoneCache {
    pub max_patterns: CachedPatternCounts,
    pub min_patterns: CachedPatternCounts,
    pub is_valid: bool,
}

impl ZoneCache {
    pub fn new() -> Self {
        Self {
            max_patterns: CachedPatternCounts::new(),
            min_patterns: CachedPatternCounts::new(),
            is_valid: false,
        }
    }

    pub fn invalidate(&mut self) {
        self.is_valid = false;
        self.max_patterns = CachedPatternCounts::new();
        self.min_patterns = CachedPatternCounts::new();
    }
}

#[derive(Debug, Clone)]
pub struct IncrementalHeuristicCache {
    zones: Vec<Vec<ZoneCache>>,
    zone_rows: usize,
    zone_cols: usize,
    board_size: usize,
    pub dirty_zones: HashSet<ZoneCoord>,
    pub cached_max_total: CachedPatternCounts,
    pub cached_min_total: CachedPatternCounts,
    pub cache_valid: bool,
}

impl IncrementalHeuristicCache {
    pub fn new(board_size: usize) -> Self {
        let zone_rows = (board_size + CACHE_ZONE_SIZE - 1) / CACHE_ZONE_SIZE;
        let zone_cols = (board_size + CACHE_ZONE_SIZE - 1) / CACHE_ZONE_SIZE;
        
        let mut zones = Vec::with_capacity(zone_rows);
        for _ in 0..zone_rows {
            let mut row = Vec::with_capacity(zone_cols);
            for _ in 0..zone_cols {
                row.push(ZoneCache::new());
            }
            zones.push(row);
        }

        Self {
            zones,
            zone_rows,
            zone_cols,
            board_size,
            dirty_zones: HashSet::new(),
            cached_max_total: CachedPatternCounts::new(),
            cached_min_total: CachedPatternCounts::new(),
            cache_valid: false,
        }
    }

    pub fn get_zone_coord(&self, row: usize, col: usize) -> ZoneCoord {
        ZoneCoord {
            zone_row: row / CACHE_ZONE_SIZE,
            zone_col: col / CACHE_ZONE_SIZE,
        }
    }

    pub fn get_affected_zones(&self, row: usize, col: usize) -> Vec<ZoneCoord> {
        let mut zones = Vec::with_capacity(4);
        
        let zone_coord = self.get_zone_coord(row, col);
        zones.push(zone_coord);
        
        let zone_start_row = zone_coord.zone_row * CACHE_ZONE_SIZE;
        let zone_start_col = zone_coord.zone_col * CACHE_ZONE_SIZE;
        let zone_end_row = (zone_coord.zone_row + 1) * CACHE_ZONE_SIZE;
        let zone_end_col = (zone_coord.zone_col + 1) * CACHE_ZONE_SIZE;
        
        if row <= zone_start_row + 2 && zone_coord.zone_row > 0 {
            zones.push(ZoneCoord { zone_row: zone_coord.zone_row - 1, zone_col: zone_coord.zone_col });
        }
        
        if row >= zone_end_row.saturating_sub(3) && zone_coord.zone_row + 1 < self.zone_rows {
            zones.push(ZoneCoord { zone_row: zone_coord.zone_row + 1, zone_col: zone_coord.zone_col });
        }
        
        if col <= zone_start_col + 2 && zone_coord.zone_col > 0 {
            zones.push(ZoneCoord { zone_row: zone_coord.zone_row, zone_col: zone_coord.zone_col - 1 });
        }
        
        if col >= zone_end_col.saturating_sub(3) && zone_coord.zone_col + 1 < self.zone_cols {
            zones.push(ZoneCoord { zone_row: zone_coord.zone_row, zone_col: zone_coord.zone_col + 1 });
        }

        zones
    }

    pub fn invalidate_zones(&mut self, zones: &[ZoneCoord]) {
        for &zone_coord in zones {
            if zone_coord.zone_row < self.zone_rows && zone_coord.zone_col < self.zone_cols {
                self.zones[zone_coord.zone_row][zone_coord.zone_col].invalidate();
                self.dirty_zones.insert(zone_coord);
            }
        }
        self.cache_valid = false;
    }

    pub fn invalidate_position(&mut self, row: usize, col: usize) {
        let affected_zones = self.get_affected_zones(row, col);
        self.invalidate_zones(&affected_zones);
    }

    pub fn get_zone_bounds(&self, zone_coord: ZoneCoord) -> (usize, usize, usize, usize) {
        let start_row = zone_coord.zone_row * CACHE_ZONE_SIZE;
        let end_row = ((zone_coord.zone_row + 1) * CACHE_ZONE_SIZE).min(self.board_size);
        let start_col = zone_coord.zone_col * CACHE_ZONE_SIZE;
        let end_col = ((zone_coord.zone_col + 1) * CACHE_ZONE_SIZE).min(self.board_size);
        
        (start_row, end_row, start_col, end_col)
    }

    pub fn update_zone_cache(&mut self, zone_coord: ZoneCoord, board: &Board, win_condition: usize) {
        if zone_coord.zone_row >= self.zone_rows || zone_coord.zone_col >= self.zone_cols {
            return;
        }

        let (start_row, end_row, start_col, end_col) = self.get_zone_bounds(zone_coord);
        
        let (max_counts, min_counts) = self.analyze_zone(board, start_row, end_row, start_col, end_col, win_condition);
        
        let zone_cache = &mut self.zones[zone_coord.zone_row][zone_coord.zone_col];
        zone_cache.max_patterns = max_counts;
        zone_cache.min_patterns = min_counts;
        zone_cache.is_valid = true;

        self.dirty_zones.remove(&zone_coord);
    }

    pub fn get_total_counts(&mut self, board: &Board, win_condition: usize) -> (CachedPatternCounts, CachedPatternCounts) {
        let dirty_zones: Vec<_> = self.dirty_zones.iter().cloned().collect();
        for zone_coord in dirty_zones {
            self.update_zone_cache(zone_coord, board, win_condition);
        }

        if !self.cache_valid {
            self.cached_max_total = CachedPatternCounts::new();
            self.cached_min_total = CachedPatternCounts::new();

            for zone_row in 0..self.zone_rows {
                for zone_col in 0..self.zone_cols {
                    let zone_cache = &self.zones[zone_row][zone_col];
                    if zone_cache.is_valid {
                        self.cached_max_total.add(&zone_cache.max_patterns);
                        self.cached_min_total.add(&zone_cache.min_patterns);
                    }
                }
            }
            self.cache_valid = true;
        }

        (self.cached_max_total, self.cached_min_total)
    }

    fn analyze_zone(&self, board: &Board, start_row: usize, end_row: usize, start_col: usize, end_col: usize, win_condition: usize) -> (CachedPatternCounts, CachedPatternCounts) {
        use crate::ai::heuristic::Heuristic;
        Heuristic::analyze_zone_patterns(board, start_row, end_row, start_col, end_col, win_condition)
    }

    pub fn force_rebuild_cache(&mut self, board: &Board, win_condition: usize) {
        for zone_row in 0..self.zone_rows {
            for zone_col in 0..self.zone_cols {
                self.zones[zone_row][zone_col].invalidate();
                self.dirty_zones.insert(ZoneCoord { zone_row, zone_col });
            }
        }
        self.cache_valid = false;
        
        let _ = self.get_total_counts(board, win_condition);
    }

    pub fn clear(&mut self) {
        for zone_row in 0..self.zone_rows {
            for zone_col in 0..self.zone_cols {
                self.zones[zone_row][zone_col].invalidate();
            }
        }
        self.dirty_zones.clear();
        self.cached_max_total = CachedPatternCounts::new();
        self.cached_min_total = CachedPatternCounts::new();
        self.cache_valid = false;
    }
}