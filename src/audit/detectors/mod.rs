//! Architecture detectors for identifying gaps and improvements.
//!
//! This module contains detectors that analyze the codebase architecture
//! and identify areas where improvements can be made.

pub mod architecture_gaps;

pub use architecture_gaps::{
    ArchitectureGap, ArchitectureGapType, ArchitectureGapsAnalysis, ArchitectureGapsDetector,
};
