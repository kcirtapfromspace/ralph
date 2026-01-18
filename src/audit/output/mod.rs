//! Output formatters for audit reports.
//!
//! This module provides various output formats for audit reports,
//! enabling both human-readable and machine-consumable outputs.

pub mod markdown;
pub mod structured;

pub use markdown::{MarkdownOutputError, MarkdownReportWriter};
pub use structured::{JsonOutputError, JsonReportWriter};
