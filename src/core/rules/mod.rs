//! Game rules and validation utilities.
//!
//! This module contains all game rule implementations separated into logical modules:
//! - `win_detection`: Functions for detecting wins and win conditions
//! - `double_three`: Double-three forbidden pattern detection
//! - `capture_breaking`: Rules for breaking five-in-a-row through captures

pub mod win_detection;
pub mod double_three;
pub mod capture_breaking;

pub use win_detection::WinDetection;
pub use double_three::DoubleThreeDetection;
pub use capture_breaking::CaptureBreaking;