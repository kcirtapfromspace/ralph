//! Quality profile definitions for Ralph.
//!
//! This module defines the data structures for quality profiles that can be
//! loaded from TOML configuration files.

// Allow dead_code for now - these structs will be used in future stories (US-008+)
#![allow(dead_code)]

use serde::Deserialize;
use std::collections::HashMap;

/// The level of a quality profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileLevel {
    /// Minimal quality gates for rapid prototyping
    Minimal,
    /// Standard quality gates for production-ready features
    Standard,
    /// Comprehensive quality gates for critical features
    Comprehensive,
}

impl Default for ProfileLevel {
    fn default() -> Self {
        Self::Standard
    }
}

/// Documentation requirements for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DocumentationConfig {
    /// Whether documentation is required
    #[serde(default)]
    pub required: bool,
    /// Whether README updates are required
    #[serde(default)]
    pub readme: bool,
    /// Whether inline comments are required
    #[serde(default)]
    pub inline_comments: bool,
}

/// Testing requirements for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TestingConfig {
    /// Whether unit tests are required
    #[serde(default)]
    pub unit_tests: bool,
    /// Whether integration tests are required
    #[serde(default)]
    pub integration_tests: bool,
    /// Minimum code coverage percentage (0-100)
    #[serde(default)]
    pub coverage_threshold: u8,
}

/// CI requirements for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CiConfig {
    /// Whether CI is required
    #[serde(default)]
    pub required: bool,
    /// Whether format checking is required
    #[serde(default)]
    pub format_check: bool,
    /// Whether lint checking is required
    #[serde(default)]
    pub lint_check: bool,
}

/// Security requirements for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SecurityConfig {
    /// Whether cargo-audit is required
    #[serde(default)]
    pub cargo_audit: bool,
    /// Whether cargo-deny is required
    #[serde(default)]
    pub cargo_deny: bool,
    /// Whether SAST (Static Application Security Testing) is required
    #[serde(default)]
    pub sast: bool,
}

/// Blog generation configuration for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BlogConfig {
    /// Whether to generate a blog post
    #[serde(default)]
    pub generate: bool,
    /// Template to use for blog generation
    #[serde(default)]
    pub template: Option<String>,
}

/// A quality profile containing all configuration sections.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Profile {
    /// Human-readable description of this profile
    #[serde(default)]
    pub description: String,
    /// Documentation requirements
    #[serde(default)]
    pub documentation: DocumentationConfig,
    /// Testing requirements
    #[serde(default)]
    pub testing: TestingConfig,
    /// CI requirements
    #[serde(default)]
    pub ci: CiConfig,
    /// Security requirements
    #[serde(default)]
    pub security: SecurityConfig,
    /// Blog generation configuration
    #[serde(default)]
    pub blog: BlogConfig,
}

/// Root configuration structure containing all quality profiles.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct QualityConfig {
    /// Map of profile names to their configurations
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

impl QualityConfig {
    /// Get a profile by its level.
    pub fn get_profile(&self, level: ProfileLevel) -> Option<&Profile> {
        let name = match level {
            ProfileLevel::Minimal => "minimal",
            ProfileLevel::Standard => "standard",
            ProfileLevel::Comprehensive => "comprehensive",
        };
        self.profiles.get(name)
    }

    /// Get a profile by name.
    pub fn get_profile_by_name(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// List all available profile names.
    pub fn profile_names(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_level_default() {
        assert_eq!(ProfileLevel::default(), ProfileLevel::Standard);
    }

    #[test]
    fn test_deserialize_minimal_profile() {
        let toml_str = r#"
            [profiles.minimal]
            description = "Test profile"

            [profiles.minimal.documentation]
            required = false

            [profiles.minimal.testing]
            unit_tests = true
            coverage_threshold = 0

            [profiles.minimal.ci]
            required = false

            [profiles.minimal.security]
            cargo_audit = false

            [profiles.minimal.blog]
            generate = false
        "#;

        let config: QualityConfig = toml::from_str(toml_str).unwrap();
        let profile = config.get_profile(ProfileLevel::Minimal).unwrap();

        assert!(!profile.documentation.required);
        assert!(profile.testing.unit_tests);
        assert_eq!(profile.testing.coverage_threshold, 0);
    }
}
