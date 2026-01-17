//! Quality framework module for Ralph.
//!
//! This module contains quality profiles and gate checking functionality.

pub mod profiles;

// Re-exports for convenience - will be used in future stories (US-009+)
#[allow(unused_imports)]
pub use profiles::{
    BlogConfig, CiConfig, DocumentationConfig, Profile, ProfileLevel, QualityConfig,
    QualityConfigError, SecurityConfig, TestingConfig,
};
