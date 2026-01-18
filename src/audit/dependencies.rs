//! Dependency analysis and parsing.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported dependency ecosystems
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyEcosystem {
    Cargo,
    Npm,
    Pip,
    Go,
    Maven,
    Gradle,
    Unknown,
}

impl Default for DependencyEcosystem {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Information about an outdated dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedInfo {
    /// Latest available version
    pub latest_version: String,
    /// Whether this is a major version bump
    pub is_major_bump: bool,
    /// Security advisory if any
    pub security_advisory: Option<String>,
}

/// A single dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Package name
    pub name: String,
    /// Current version
    pub version: String,
    /// Ecosystem this dependency belongs to
    pub ecosystem: DependencyEcosystem,
    /// Whether this is a dev/test dependency
    pub is_dev: bool,
    /// Path to manifest file
    pub manifest_path: PathBuf,
    /// Outdated info if available
    pub outdated: Option<OutdatedInfo>,
}

/// Complete dependency analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// All detected dependencies
    pub dependencies: Vec<Dependency>,
    /// Count by ecosystem
    pub ecosystem_counts: Vec<(DependencyEcosystem, usize)>,
    /// Number of outdated dependencies
    pub outdated_count: usize,
    /// Number of dependencies with security advisories
    pub vulnerable_count: usize,
}

/// Parser for extracting dependencies from manifest files
pub struct DependencyParser {
    root: PathBuf,
}

impl DependencyParser {
    /// Create a new dependency parser
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Parse all dependencies in the project
    pub fn parse(&self) -> crate::audit::AuditResult<DependencyAnalysis> {
        // Stub implementation - will be implemented in future stories
        Ok(DependencyAnalysis::default())
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecosystem_default() {
        assert_eq!(DependencyEcosystem::default(), DependencyEcosystem::Unknown);
    }

    #[test]
    fn test_dependency_parser_new() {
        let parser = DependencyParser::new(PathBuf::from("/test"));
        assert_eq!(parser.root(), &PathBuf::from("/test"));
    }
}
