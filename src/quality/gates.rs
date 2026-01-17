//! Quality gate checking functionality for Ralph.
//!
//! This module provides the infrastructure for running quality gates
//! against a codebase, including coverage, linting, formatting, and security checks.

// Allow dead_code for now - these types will be used in future stories (US-010+)
#![allow(dead_code)]

use crate::quality::Profile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The result of running a single quality gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Name of the quality gate that was run
    pub gate_name: String,
    /// Whether the gate passed
    pub passed: bool,
    /// Human-readable message describing the result
    pub message: String,
    /// Additional details about the gate result (e.g., specific errors, metrics)
    pub details: Option<String>,
}

impl GateResult {
    /// Create a new passing gate result.
    pub fn pass(gate_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            gate_name: gate_name.into(),
            passed: true,
            message: message.into(),
            details: None,
        }
    }

    /// Create a new failing gate result.
    pub fn fail(
        gate_name: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            gate_name: gate_name.into(),
            passed: false,
            message: message.into(),
            details,
        }
    }

    /// Create a new skipped gate result.
    pub fn skipped(gate_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            gate_name: gate_name.into(),
            passed: true, // Skipped gates count as passed
            message: format!("Skipped: {}", reason.into()),
            details: None,
        }
    }
}

/// A checker that runs quality gates based on a profile configuration.
pub struct QualityGateChecker {
    /// The quality profile to check against
    profile: Profile,
    /// The root directory of the project to check
    project_root: PathBuf,
}

impl QualityGateChecker {
    /// Create a new quality gate checker.
    ///
    /// # Arguments
    ///
    /// * `profile` - The quality profile containing gate configurations
    /// * `project_root` - The root directory of the project to check
    pub fn new(profile: Profile, project_root: impl Into<PathBuf>) -> Self {
        Self {
            profile,
            project_root: project_root.into(),
        }
    }

    /// Get the profile being used for quality checks.
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Get the project root directory.
    pub fn project_root(&self) -> &PathBuf {
        &self.project_root
    }

    /// Run all quality gates configured in the profile.
    ///
    /// Returns a vector of `GateResult` for each gate that was run.
    /// Gates that are not enabled in the profile will be skipped.
    ///
    /// # Returns
    ///
    /// A `Vec<GateResult>` containing the results of all gates.
    pub fn run_all(&self) -> Vec<GateResult> {
        let mut results = Vec::new();

        // Check coverage gate (will be implemented in US-010)
        if self.profile.testing.coverage_threshold > 0 {
            results.push(GateResult::skipped(
                "coverage",
                "Coverage checking not yet implemented",
            ));
        } else {
            results.push(GateResult::skipped(
                "coverage",
                "Coverage threshold is 0 - no check required",
            ));
        }

        // Check lint gate (will be implemented in US-011)
        if self.profile.ci.lint_check {
            results.push(GateResult::skipped(
                "lint",
                "Lint checking not yet implemented",
            ));
        } else {
            results.push(GateResult::skipped(
                "lint",
                "Lint checking not enabled in profile",
            ));
        }

        // Check format gate (will be implemented in US-011)
        if self.profile.ci.format_check {
            results.push(GateResult::skipped(
                "format",
                "Format checking not yet implemented",
            ));
        } else {
            results.push(GateResult::skipped(
                "format",
                "Format checking not enabled in profile",
            ));
        }

        // Check security audit gate (will be implemented in US-012)
        if self.profile.security.cargo_audit {
            results.push(GateResult::skipped(
                "security_audit",
                "Security audit not yet implemented",
            ));
        } else {
            results.push(GateResult::skipped(
                "security_audit",
                "Security audit not enabled in profile",
            ));
        }

        results
    }

    /// Check if all gates passed.
    pub fn all_passed(results: &[GateResult]) -> bool {
        results.iter().all(|r| r.passed)
    }

    /// Get a summary of gate results.
    pub fn summary(results: &[GateResult]) -> String {
        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();
        let failed: Vec<&str> = results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.gate_name.as_str())
            .collect();

        if failed.is_empty() {
            format!("All {total} gates passed")
        } else {
            format!(
                "{passed}/{total} gates passed. Failed: {}",
                failed.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::{CiConfig, Profile, SecurityConfig, TestingConfig};

    fn create_test_profile(coverage: u8, lint: bool, format: bool, audit: bool) -> Profile {
        Profile {
            description: "Test profile".to_string(),
            testing: TestingConfig {
                coverage_threshold: coverage,
                unit_tests: true,
                integration_tests: false,
            },
            ci: CiConfig {
                required: true,
                lint_check: lint,
                format_check: format,
            },
            security: SecurityConfig {
                cargo_audit: audit,
                cargo_deny: false,
                sast: false,
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_gate_result_pass() {
        let result = GateResult::pass("test_gate", "Test passed");
        assert!(result.passed);
        assert_eq!(result.gate_name, "test_gate");
        assert_eq!(result.message, "Test passed");
        assert!(result.details.is_none());
    }

    #[test]
    fn test_gate_result_fail() {
        let result = GateResult::fail(
            "test_gate",
            "Test failed",
            Some("Error details".to_string()),
        );
        assert!(!result.passed);
        assert_eq!(result.gate_name, "test_gate");
        assert_eq!(result.message, "Test failed");
        assert_eq!(result.details, Some("Error details".to_string()));
    }

    #[test]
    fn test_gate_result_skipped() {
        let result = GateResult::skipped("test_gate", "Not enabled");
        assert!(result.passed); // Skipped counts as passed
        assert_eq!(result.gate_name, "test_gate");
        assert!(result.message.contains("Skipped"));
    }

    #[test]
    fn test_checker_run_all_minimal() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let results = checker.run_all();

        assert_eq!(results.len(), 4);
        assert!(QualityGateChecker::all_passed(&results));
    }

    #[test]
    fn test_checker_run_all_comprehensive() {
        let profile = create_test_profile(90, true, true, true);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let results = checker.run_all();

        assert_eq!(results.len(), 4);
        // All skipped for now (not implemented), so all pass
        assert!(QualityGateChecker::all_passed(&results));
    }

    #[test]
    fn test_all_passed_true() {
        let results = vec![
            GateResult::pass("gate1", "Passed"),
            GateResult::pass("gate2", "Passed"),
        ];
        assert!(QualityGateChecker::all_passed(&results));
    }

    #[test]
    fn test_all_passed_false() {
        let results = vec![
            GateResult::pass("gate1", "Passed"),
            GateResult::fail("gate2", "Failed", None),
        ];
        assert!(!QualityGateChecker::all_passed(&results));
    }

    #[test]
    fn test_summary_all_passed() {
        let results = vec![
            GateResult::pass("gate1", "Passed"),
            GateResult::pass("gate2", "Passed"),
        ];
        let summary = QualityGateChecker::summary(&results);
        assert_eq!(summary, "All 2 gates passed");
    }

    #[test]
    fn test_summary_some_failed() {
        let results = vec![
            GateResult::pass("gate1", "Passed"),
            GateResult::fail("gate2", "Failed", None),
            GateResult::fail("gate3", "Failed", None),
        ];
        let summary = QualityGateChecker::summary(&results);
        assert!(summary.contains("1/3 gates passed"));
        assert!(summary.contains("gate2"));
        assert!(summary.contains("gate3"));
    }
}
