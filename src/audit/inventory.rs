//! File inventory and project structure analysis.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Detected project type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    Mixed,
    Unknown,
}

impl Default for ProjectType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Purpose classification for directories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectoryPurpose {
    Source,
    Test,
    Documentation,
    Configuration,
    Build,
    Dependencies,
    Assets,
    Unknown,
}

impl Default for DirectoryPurpose {
    fn default() -> Self {
        Self::Unknown
    }
}

/// A node in the directory tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    /// Directory name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Detected purpose
    pub purpose: DirectoryPurpose,
    /// Child directories
    pub children: Vec<DirectoryNode>,
    /// Number of files in this directory (not recursive)
    pub file_count: usize,
}

/// Key file identified in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFile {
    /// File path relative to project root
    pub path: PathBuf,
    /// File type/purpose
    pub file_type: String,
    /// Why this file is considered key
    pub significance: String,
}

/// Complete file inventory for a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileInventory {
    /// Detected project type
    pub project_type: ProjectType,
    /// Total file count
    pub total_files: usize,
    /// Total lines of code (estimated)
    pub total_loc: usize,
    /// Directory structure
    pub structure: Vec<DirectoryNode>,
    /// Key files identified
    pub key_files: Vec<KeyFile>,
}

/// Scanner for building file inventories
pub struct InventoryScanner {
    root: PathBuf,
}

impl InventoryScanner {
    /// Create a new inventory scanner
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Scan the project and build inventory
    pub fn scan(&self) -> crate::audit::AuditResult<FileInventory> {
        // Stub implementation - will be implemented in future stories
        Ok(FileInventory {
            project_type: ProjectType::Unknown,
            total_files: 0,
            total_loc: 0,
            structure: Vec::new(),
            key_files: Vec::new(),
        })
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
    fn test_project_type_default() {
        assert_eq!(ProjectType::default(), ProjectType::Unknown);
    }

    #[test]
    fn test_directory_purpose_default() {
        assert_eq!(DirectoryPurpose::default(), DirectoryPurpose::Unknown);
    }

    #[test]
    fn test_inventory_scanner_new() {
        let scanner = InventoryScanner::new(PathBuf::from("/test"));
        assert_eq!(scanner.root(), &PathBuf::from("/test"));
    }
}
