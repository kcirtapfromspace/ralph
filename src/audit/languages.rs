//! Language detection and analysis.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Level of language support in the codebase
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageSupport {
    Primary,
    Secondary,
    Minimal,
}

impl Default for LanguageSupport {
    fn default() -> Self {
        Self::Minimal
    }
}

/// Information about a detected language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    /// Language name
    pub name: String,
    /// File extensions associated
    pub extensions: Vec<String>,
    /// Number of files
    pub file_count: usize,
    /// Lines of code
    pub loc: usize,
    /// Support level in the project
    pub support: LanguageSupport,
}

/// Detector for identifying languages in files
pub struct LanguageDetector;

impl LanguageDetector {
    /// Create a new language detector
    pub fn new() -> Self {
        Self
    }

    /// Detect language from file extension
    pub fn detect_from_extension(&self, extension: &str) -> Option<String> {
        match extension {
            "rs" => Some("Rust".to_string()),
            "js" => Some("JavaScript".to_string()),
            "ts" => Some("TypeScript".to_string()),
            "py" => Some("Python".to_string()),
            "go" => Some("Go".to_string()),
            "java" => Some("Java".to_string()),
            "rb" => Some("Ruby".to_string()),
            "c" | "h" => Some("C".to_string()),
            "cpp" | "hpp" | "cc" | "cxx" => Some("C++".to_string()),
            "cs" => Some("C#".to_string()),
            "swift" => Some("Swift".to_string()),
            "kt" | "kts" => Some("Kotlin".to_string()),
            _ => None,
        }
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzer for aggregating language statistics
pub struct LanguageAnalyzer {
    root: PathBuf,
    detector: LanguageDetector,
}

impl LanguageAnalyzer {
    /// Create a new language analyzer
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            detector: LanguageDetector::new(),
        }
    }

    /// Analyze languages in the project
    pub fn analyze(&self) -> crate::audit::AuditResult<Vec<LanguageInfo>> {
        // Stub implementation - will be implemented in future stories
        Ok(Vec::new())
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get the detector
    pub fn detector(&self) -> &LanguageDetector {
        &self.detector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_support_default() {
        assert_eq!(LanguageSupport::default(), LanguageSupport::Minimal);
    }

    #[test]
    fn test_language_detector_rust() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_extension("rs"),
            Some("Rust".to_string())
        );
    }

    #[test]
    fn test_language_detector_unknown() {
        let detector = LanguageDetector::new();
        assert_eq!(detector.detect_from_extension("xyz"), None);
    }
}
