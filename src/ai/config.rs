use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use bevy::prelude::Resource;

#[derive(Deserialize, Serialize, Clone, Debug, Resource)]
pub struct AIConfig {
    pub heuristic: HeuristicConfig,
    pub move_generation: MoveGenerationConfig,
    pub pattern_history: PatternHistoryConfig,
    pub search: SearchConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HeuristicConfig {
    pub scores: HeuristicScores,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HeuristicScores {
    #[serde(rename = "WINNING_SCORE")]
    pub winning_score: i32,
    #[serde(rename = "FIVE_IN_ROW_SCORE")]
    pub five_in_row_score: i32,
    #[serde(rename = "LIVE_FOUR_MULTIPLE_SCORE")]
    pub live_four_multiple_score: i32,
    #[serde(rename = "LIVE_FOUR_SINGLE_SCORE")]
    pub live_four_single_score: i32,
    #[serde(rename = "CAPTURE_BONUS_MULTIPLIER")]
    pub capture_bonus_multiplier: i32,
    #[serde(rename = "CHECK_PENALTY")]
    pub check_penalty: i32,
    #[serde(rename = "WINNING_THREAT_SCORE")]
    pub winning_threat_score: i32,
    #[serde(rename = "HALF_FREE_FOUR_SCORE")]
    pub half_free_four_score: i32,
    #[serde(rename = "LIVE_THREE_SCORE")]
    pub live_three_score: i32,
    #[serde(rename = "DEAD_FOUR_SCORE")]
    pub dead_four_score: i32,
    #[serde(rename = "HALF_FREE_THREE_SCORE")]
    pub half_free_three_score: i32,
    #[serde(rename = "DEAD_THREE_SCORE")]
    pub dead_three_score: i32,
    #[serde(rename = "LIVE_TWO_SCORE")]
    pub live_two_score: i32,
    #[serde(rename = "HALF_FREE_TWO_SCORE")]
    pub half_free_two_score: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MoveGenerationConfig {
    #[serde(rename = "CENTER_POSITION_BONUS")]
    pub center_position_bonus: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PatternHistoryConfig {
    #[serde(rename = "CAPTURE_MOMENTUM_BONUS")]
    pub capture_momentum_bonus: i32,
    #[serde(rename = "INITIATIVE_BONUS")]
    pub initiative_bonus: i32,
    #[serde(rename = "HISTORY_WINDOW")]
    pub history_window: usize,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SearchConfig {
    pub parallel: ParallelSearchConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ParallelSearchConfig {
    #[serde(rename = "ASPIRATION_OFFSETS")]
    pub aspiration_offsets: Vec<i32>,
    #[serde(rename = "DEPTH_OFFSETS")]
    pub depth_offsets: Vec<i32>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            heuristic: HeuristicConfig {
                scores: HeuristicScores {
                    winning_score: 1_000_000,
                    five_in_row_score: 100_000,
                    live_four_multiple_score: 20_000,
                    live_four_single_score: 15_000,
                    capture_bonus_multiplier: 15_000,
                    check_penalty: 10_000,
                    winning_threat_score: 10_000,
                    half_free_four_score: 3_500,
                    live_three_score: 500,
                    dead_four_score: 400,
                    half_free_three_score: 200,
                    dead_three_score: 50,
                    live_two_score: 50,
                    half_free_two_score: 20,
                },
            },
            move_generation: MoveGenerationConfig {
                center_position_bonus: 10,
            },
            pattern_history: PatternHistoryConfig {
                capture_momentum_bonus: 200,
                initiative_bonus: 100,
                history_window: 4,
            },
            search: SearchConfig {
                parallel: ParallelSearchConfig {
                    aspiration_offsets: vec![0, 25, -25, 75, -75],
                    depth_offsets: vec![0, -1, 1, -2],
                },
            },
        }
    }
}

impl AIConfig {
    /// Load AI configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AIConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load AI configuration with fallback to default
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load_from_file(path).unwrap_or_else(|e| {
            eprintln!("Failed to load AI config: {}. Using defaults.", e);
            Self::default()
        })
    }

    /// Save current configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}