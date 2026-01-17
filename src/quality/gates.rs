//! Quality gate checking functionality for Ralph.
//!
//! This module provides the infrastructure for running quality gates
//! against a codebase, including coverage, linting, formatting, and security checks.

// Allow dead_code for now - these types will be used in future stories
#![allow(dead_code)]

use crate::quality::Profile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

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

    /// Check code coverage against the profile threshold.
    ///
    /// This method runs either `cargo llvm-cov` or `cargo tarpaulin` to measure
    /// code coverage and compares it against the threshold configured in the profile.
    ///
    /// # Returns
    ///
    /// A `GateResult` indicating whether the coverage threshold was met.
    /// If coverage tools are not installed, returns a failure with installation instructions.
    pub fn check_coverage(&self) -> GateResult {
        let threshold = self.profile.testing.coverage_threshold;

        // If threshold is 0, skip coverage check
        if threshold == 0 {
            return GateResult::skipped("coverage", "Coverage threshold is 0 - no check required");
        }

        // Try cargo-llvm-cov first (more common in CI environments)
        let llvm_cov_result = self.run_llvm_cov();
        if let Some(result) = llvm_cov_result {
            return result;
        }

        // Fall back to cargo-tarpaulin
        let tarpaulin_result = self.run_tarpaulin();
        if let Some(result) = tarpaulin_result {
            return result;
        }

        // Neither tool is available
        GateResult::fail(
            "coverage",
            "No coverage tool available",
            Some(
                "Install cargo-llvm-cov: cargo install cargo-llvm-cov\n\
                 Or install cargo-tarpaulin: cargo install cargo-tarpaulin"
                    .to_string(),
            ),
        )
    }

    /// Run cargo-llvm-cov and parse the coverage percentage.
    fn run_llvm_cov(&self) -> Option<GateResult> {
        // Check if cargo-llvm-cov is installed
        let check_installed = Command::new("cargo")
            .args(["llvm-cov", "--version"])
            .current_dir(&self.project_root)
            .output();

        if check_installed.is_err() || !check_installed.unwrap().status.success() {
            return None; // Tool not installed
        }

        // Run cargo llvm-cov with JSON output for parsing
        let output = Command::new("cargo")
            .args(["llvm-cov", "--json", "--quiet"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Some(GateResult::fail(
                        "coverage",
                        "cargo llvm-cov failed",
                        Some(format!("stderr: {}", stderr)),
                    ));
                }

                // Parse the JSON output for coverage percentage
                if let Some(coverage) = Self::parse_llvm_cov_json(&stdout) {
                    Some(self.evaluate_coverage(coverage, "cargo-llvm-cov"))
                } else {
                    // If JSON parsing fails, try running with summary output
                    self.run_llvm_cov_summary()
                }
            }
            Err(e) => Some(GateResult::fail(
                "coverage",
                "Failed to run cargo llvm-cov",
                Some(e.to_string()),
            )),
        }
    }

    /// Run cargo-llvm-cov with summary output and parse the percentage.
    fn run_llvm_cov_summary(&self) -> Option<GateResult> {
        let output = Command::new("cargo")
            .args(["llvm-cov", "--quiet"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Some(GateResult::fail(
                        "coverage",
                        "cargo llvm-cov failed",
                        Some(format!("stderr: {}", stderr)),
                    ));
                }

                // Parse the summary output for coverage percentage
                // llvm-cov outputs lines like "TOTAL ... 75.00%"
                if let Some(coverage) = Self::parse_coverage_percentage(&stdout) {
                    Some(self.evaluate_coverage(coverage, "cargo-llvm-cov"))
                } else {
                    Some(GateResult::fail(
                        "coverage",
                        "Failed to parse llvm-cov output",
                        Some(format!("Output: {}", stdout)),
                    ))
                }
            }
            Err(e) => Some(GateResult::fail(
                "coverage",
                "Failed to run cargo llvm-cov",
                Some(e.to_string()),
            )),
        }
    }

    /// Parse llvm-cov JSON output for total coverage percentage.
    fn parse_llvm_cov_json(json_str: &str) -> Option<f64> {
        // llvm-cov JSON has a "data" array with coverage info
        // We need to extract the total line coverage percentage
        // Format: { "data": [{ "totals": { "lines": { "percent": 75.5 } } }] }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
            json.get("data")
                .and_then(|d| d.get(0))
                .and_then(|d| d.get("totals"))
                .and_then(|t| t.get("lines"))
                .and_then(|l| l.get("percent"))
                .and_then(|p| p.as_f64())
        } else {
            None
        }
    }

    /// Run cargo-tarpaulin and parse the coverage percentage.
    fn run_tarpaulin(&self) -> Option<GateResult> {
        // Check if cargo-tarpaulin is installed
        let check_installed = Command::new("cargo")
            .args(["tarpaulin", "--version"])
            .current_dir(&self.project_root)
            .output();

        if check_installed.is_err() || !check_installed.unwrap().status.success() {
            return None; // Tool not installed
        }

        // Run cargo tarpaulin
        let output = Command::new("cargo")
            .args(["tarpaulin", "--skip-clean", "--out", "Stdout"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // tarpaulin returns exit code 0 even on low coverage
                // Parse the output for coverage percentage
                // Format: "XX.XX% coverage"
                if let Some(coverage) = Self::parse_coverage_percentage(&stdout) {
                    Some(self.evaluate_coverage(coverage, "cargo-tarpaulin"))
                } else if let Some(coverage) = Self::parse_coverage_percentage(&stderr) {
                    // Sometimes tarpaulin outputs to stderr
                    Some(self.evaluate_coverage(coverage, "cargo-tarpaulin"))
                } else {
                    Some(GateResult::fail(
                        "coverage",
                        "Failed to parse tarpaulin output",
                        Some(format!("stdout: {}\nstderr: {}", stdout, stderr)),
                    ))
                }
            }
            Err(e) => Some(GateResult::fail(
                "coverage",
                "Failed to run cargo tarpaulin",
                Some(e.to_string()),
            )),
        }
    }

    /// Parse coverage percentage from text output.
    /// Looks for patterns like "75.00%" or "75.00% coverage" or "TOTAL ... 75.00%"
    fn parse_coverage_percentage(output: &str) -> Option<f64> {
        // Look for percentage patterns
        let re_patterns = [
            // Match "XX.XX% coverage" (tarpaulin format)
            r"(\d+(?:\.\d+)?)\s*%\s*coverage",
            // Match "TOTAL ... XX.XX%" (llvm-cov format)
            r"TOTAL\s+.*?(\d+(?:\.\d+)?)\s*%",
            // Match standalone percentage at end of line
            r"(\d+(?:\.\d+)?)\s*%\s*$",
        ];

        for pattern in &re_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(output) {
                    if let Some(match_) = captures.get(1) {
                        if let Ok(coverage) = match_.as_str().parse::<f64>() {
                            return Some(coverage);
                        }
                    }
                }
            }
        }

        None
    }

    /// Evaluate coverage against the threshold and return a GateResult.
    fn evaluate_coverage(&self, coverage: f64, tool_name: &str) -> GateResult {
        let threshold = self.profile.testing.coverage_threshold as f64;
        let coverage_str = format!("{:.2}%", coverage);

        if coverage >= threshold {
            GateResult::pass(
                "coverage",
                format!(
                    "Coverage {coverage_str} meets threshold of {threshold:.0}% (via {tool_name})"
                ),
            )
        } else {
            GateResult::fail(
                "coverage",
                format!("Coverage {coverage_str} is below threshold of {threshold:.0}%"),
                Some(format!(
                    "Measured with {tool_name}. Increase test coverage to meet the threshold."
                )),
            )
        }
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

        // Check coverage gate
        results.push(self.check_coverage());

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
        // Coverage gate may fail if tools not installed, lint/format/security are still skipped
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

    // Coverage gate tests

    #[test]
    fn test_check_coverage_zero_threshold_skipped() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.check_coverage();

        assert!(result.passed);
        assert_eq!(result.gate_name, "coverage");
        assert!(result.message.contains("Skipped"));
        assert!(result.message.contains("threshold is 0"));
    }

    #[test]
    fn test_check_coverage_with_threshold() {
        // This test checks that the coverage gate attempts to run when threshold > 0
        let profile = create_test_profile(70, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.check_coverage();

        // Should either pass (if tools installed) or fail with "no coverage tool available"
        assert_eq!(result.gate_name, "coverage");
        // The result depends on whether coverage tools are installed
        // If not installed, it should fail with a helpful message
        if !result.passed {
            assert!(
                result.message.contains("No coverage tool")
                    || result.message.contains("failed")
                    || result.message.contains("below threshold"),
                "Unexpected failure message: {}",
                result.message
            );
        }
    }

    #[test]
    fn test_parse_coverage_percentage_tarpaulin_format() {
        // Test tarpaulin-style output
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("75.00% coverage"),
            Some(75.0)
        );
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("100% coverage"),
            Some(100.0)
        );
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("0.5% coverage"),
            Some(0.5)
        );
    }

    #[test]
    fn test_parse_coverage_percentage_llvm_cov_format() {
        // Test llvm-cov-style TOTAL line
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("TOTAL 100 50 50.00%"),
            Some(50.0)
        );
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage(
                "Filename   Functions  Lines\nTOTAL      10         75.50%"
            ),
            Some(75.5)
        );
    }

    #[test]
    fn test_parse_coverage_percentage_invalid() {
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("no match here"),
            None
        );
        assert_eq!(QualityGateChecker::parse_coverage_percentage(""), None);
    }

    #[test]
    fn test_parse_llvm_cov_json() {
        let json = r#"{
            "data": [{
                "totals": {
                    "lines": {
                        "percent": 82.5
                    }
                }
            }]
        }"#;
        assert_eq!(QualityGateChecker::parse_llvm_cov_json(json), Some(82.5));
    }

    #[test]
    fn test_parse_llvm_cov_json_invalid() {
        assert_eq!(QualityGateChecker::parse_llvm_cov_json("not json"), None);
        assert_eq!(QualityGateChecker::parse_llvm_cov_json("{}"), None);
        assert_eq!(
            QualityGateChecker::parse_llvm_cov_json(r#"{"data": []}"#),
            None
        );
    }

    #[test]
    fn test_evaluate_coverage_pass() {
        let profile = create_test_profile(70, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.evaluate_coverage(80.0, "test-tool");

        assert!(result.passed);
        assert!(result.message.contains("80.00%"));
        assert!(result.message.contains("meets threshold"));
        assert!(result.message.contains("70%"));
    }

    #[test]
    fn test_evaluate_coverage_fail() {
        let profile = create_test_profile(70, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.evaluate_coverage(50.0, "test-tool");

        assert!(!result.passed);
        assert!(result.message.contains("50.00%"));
        assert!(result.message.contains("below threshold"));
        assert!(result.details.is_some());
        assert!(result.details.unwrap().contains("test-tool"));
    }

    #[test]
    fn test_evaluate_coverage_exact_threshold() {
        let profile = create_test_profile(70, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.evaluate_coverage(70.0, "test-tool");

        assert!(result.passed, "Coverage at exactly threshold should pass");
    }
}
