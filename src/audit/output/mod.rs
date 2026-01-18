//! Output formatters for audit reports.
//!
//! This module provides various output formats for audit reports,
//! enabling both human-readable and machine-consumable outputs.

pub mod structured;

pub use structured::{JsonOutputError, JsonReportWriter};
