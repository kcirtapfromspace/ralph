// start_audit MCP tool implementation
// This tool starts a codebase audit via MCP for programmatic analysis

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global audit ID counter for generating unique audit IDs.
static AUDIT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Supported output formats for audit reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuditOutputFormat {
    /// JSON format - machine-readable structured output
    #[default]
    Json,
    /// Markdown format - human-readable report
    Markdown,
    /// Agent context format - optimized for AI agents
    AgentContext,
}

impl std::fmt::Display for AuditOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditOutputFormat::Json => write!(f, "json"),
            AuditOutputFormat::Markdown => write!(f, "markdown"),
            AuditOutputFormat::AgentContext => write!(f, "agent_context"),
        }
    }
}

/// Audit sections that can be analyzed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AuditSection {
    /// File structure and inventory analysis
    Inventory,
    /// Dependency analysis
    Dependencies,
    /// Architecture pattern analysis
    Architecture,
    /// Testing coverage analysis
    Testing,
    /// Documentation analysis
    Documentation,
    /// API analysis
    Api,
    /// Technical debt detection
    TechDebt,
    /// Feature opportunities detection
    Opportunities,
}

impl std::fmt::Display for AuditSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSection::Inventory => write!(f, "inventory"),
            AuditSection::Dependencies => write!(f, "dependencies"),
            AuditSection::Architecture => write!(f, "architecture"),
            AuditSection::Testing => write!(f, "testing"),
            AuditSection::Documentation => write!(f, "documentation"),
            AuditSection::Api => write!(f, "api"),
            AuditSection::TechDebt => write!(f, "tech_debt"),
            AuditSection::Opportunities => write!(f, "opportunities"),
        }
    }
}

/// Request parameters for the start_audit tool.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct StartAuditRequest {
    /// Path to the directory to audit.
    /// If not provided, defaults to the current PRD directory or current working directory.
    #[schemars(
        description = "Path to the directory to audit (optional, defaults to PRD directory)"
    )]
    #[serde(default)]
    pub path: Option<String>,

    /// Sections to include in the audit.
    /// If not provided, all sections will be analyzed.
    #[schemars(
        description = "Sections to analyze: inventory, dependencies, architecture, testing, documentation, api, tech_debt, opportunities"
    )]
    #[serde(default)]
    pub sections: Option<Vec<AuditSection>>,

    /// Output format for the audit report.
    /// Defaults to "json".
    #[schemars(description = "Output format: json, markdown, or agent_context (default: json)")]
    #[serde(default)]
    pub format: Option<AuditOutputFormat>,
}

/// Response from the start_audit tool.
#[derive(Debug, Serialize)]
pub struct StartAuditResponse {
    /// Whether the audit was started successfully
    pub success: bool,

    /// Unique audit ID for status checking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_id: Option<String>,

    /// Path being audited
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Sections being analyzed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<String>>,

    /// Output format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Message describing the result
    pub message: String,
}

/// Audit state for tracking in-progress audits.
#[derive(Debug, Clone)]
pub struct AuditState {
    /// Unique audit ID
    pub audit_id: String,
    /// Path being audited
    pub path: PathBuf,
    /// Sections to analyze
    pub sections: Vec<AuditSection>,
    /// Output format
    pub format: AuditOutputFormat,
    /// When the audit started (Unix timestamp)
    pub started_at: u64,
    /// Whether the audit is complete
    pub completed: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Error types for start_audit operations.
#[derive(Debug)]
pub enum StartAuditError {
    /// Path does not exist
    PathNotFound(String),
    /// Path is not a directory
    NotADirectory(String),
    /// Invalid section specified
    InvalidSection(String),
    /// Audit initialization failed
    InitializationError(String),
}

impl std::fmt::Display for StartAuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartAuditError::PathNotFound(path) => {
                write!(f, "Path not found: {}", path)
            }
            StartAuditError::NotADirectory(path) => {
                write!(f, "Path is not a directory: {}", path)
            }
            StartAuditError::InvalidSection(section) => {
                write!(f, "Invalid audit section: {}", section)
            }
            StartAuditError::InitializationError(msg) => {
                write!(f, "Failed to initialize audit: {}", msg)
            }
        }
    }
}

/// Generate a unique audit ID.
pub fn generate_audit_id() -> String {
    let counter = AUDIT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = current_timestamp();
    format!("audit-{}-{}", timestamp, counter)
}

/// Get the current Unix timestamp.
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Get all available audit sections.
pub fn all_sections() -> Vec<AuditSection> {
    vec![
        AuditSection::Inventory,
        AuditSection::Dependencies,
        AuditSection::Architecture,
        AuditSection::Testing,
        AuditSection::Documentation,
        AuditSection::Api,
        AuditSection::TechDebt,
        AuditSection::Opportunities,
    ]
}

/// Validate the audit path.
pub fn validate_path(path: &str) -> Result<PathBuf, StartAuditError> {
    let path_buf = PathBuf::from(path);

    // Canonicalize the path to handle relative paths
    let canonical = if path_buf.is_absolute() {
        path_buf
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(&path_buf))
            .unwrap_or(path_buf)
    };

    if !canonical.exists() {
        return Err(StartAuditError::PathNotFound(path.to_string()));
    }

    if !canonical.is_dir() {
        return Err(StartAuditError::NotADirectory(path.to_string()));
    }

    Ok(canonical)
}

/// Resolve the audit path from request parameters and server state.
pub fn resolve_audit_path(
    requested_path: Option<&str>,
    prd_path: Option<&PathBuf>,
) -> Result<PathBuf, StartAuditError> {
    // Priority: requested path > PRD directory > current directory
    if let Some(path) = requested_path {
        return validate_path(path);
    }

    if let Some(prd) = prd_path {
        if let Some(parent) = prd.parent() {
            if parent.exists() && parent.is_dir() {
                return Ok(parent.to_path_buf());
            }
        }
    }

    // Fall back to current directory
    std::env::current_dir().map_err(|e| {
        StartAuditError::InitializationError(format!("Failed to get current directory: {}", e))
    })
}

/// Create a success response for start_audit.
pub fn create_success_response(state: &AuditState) -> StartAuditResponse {
    let section_names: Vec<String> = state.sections.iter().map(|s| s.to_string()).collect();

    StartAuditResponse {
        success: true,
        audit_id: Some(state.audit_id.clone()),
        path: Some(state.path.display().to_string()),
        sections: Some(section_names),
        format: Some(state.format.to_string()),
        message: format!(
            "Audit started successfully. Use audit_id '{}' to check status.",
            state.audit_id
        ),
    }
}

/// Create an error response for start_audit.
pub fn create_error_response(error: &StartAuditError) -> StartAuditResponse {
    StartAuditResponse {
        success: false,
        audit_id: None,
        path: None,
        sections: None,
        format: None,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_audit_id() {
        let id1 = generate_audit_id();
        let id2 = generate_audit_id();

        assert!(id1.starts_with("audit-"));
        assert!(id2.starts_with("audit-"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        // Should be after 2024-01-01 (Unix timestamp 1704067200)
        assert!(ts > 1704067200);
    }

    #[test]
    fn test_all_sections() {
        let sections = all_sections();
        assert_eq!(sections.len(), 8);
        assert!(sections.contains(&AuditSection::Inventory));
        assert!(sections.contains(&AuditSection::Dependencies));
        assert!(sections.contains(&AuditSection::Architecture));
        assert!(sections.contains(&AuditSection::Testing));
        assert!(sections.contains(&AuditSection::Documentation));
        assert!(sections.contains(&AuditSection::Api));
        assert!(sections.contains(&AuditSection::TechDebt));
        assert!(sections.contains(&AuditSection::Opportunities));
    }

    #[test]
    fn test_validate_path_success() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_path(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_not_found() {
        let result = validate_path("/nonexistent/path/to/directory");
        assert!(result.is_err());

        match result.unwrap_err() {
            StartAuditError::PathNotFound(_) => {}
            _ => panic!("Expected PathNotFound error"),
        }
    }

    #[test]
    fn test_validate_path_not_directory() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test").unwrap();

        let result = validate_path(file.path().to_str().unwrap());
        assert!(result.is_err());

        match result.unwrap_err() {
            StartAuditError::NotADirectory(_) => {}
            _ => panic!("Expected NotADirectory error"),
        }
    }

    #[test]
    fn test_resolve_audit_path_with_requested() {
        let temp_dir = TempDir::new().unwrap();
        let result = resolve_audit_path(Some(temp_dir.path().to_str().unwrap()), None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_resolve_audit_path_with_prd() {
        let temp_dir = TempDir::new().unwrap();
        let prd_path = temp_dir.path().join("prd.json");

        let result = resolve_audit_path(None, Some(&prd_path));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_resolve_audit_path_fallback_to_cwd() {
        let result = resolve_audit_path(None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), std::env::current_dir().unwrap());
    }

    #[test]
    fn test_audit_output_format_default() {
        let format: AuditOutputFormat = Default::default();
        assert_eq!(format, AuditOutputFormat::Json);
    }

    #[test]
    fn test_audit_output_format_display() {
        assert_eq!(AuditOutputFormat::Json.to_string(), "json");
        assert_eq!(AuditOutputFormat::Markdown.to_string(), "markdown");
        assert_eq!(AuditOutputFormat::AgentContext.to_string(), "agent_context");
    }

    #[test]
    fn test_audit_section_display() {
        assert_eq!(AuditSection::Inventory.to_string(), "inventory");
        assert_eq!(AuditSection::Dependencies.to_string(), "dependencies");
        assert_eq!(AuditSection::Architecture.to_string(), "architecture");
        assert_eq!(AuditSection::Testing.to_string(), "testing");
        assert_eq!(AuditSection::Documentation.to_string(), "documentation");
        assert_eq!(AuditSection::Api.to_string(), "api");
        assert_eq!(AuditSection::TechDebt.to_string(), "tech_debt");
        assert_eq!(AuditSection::Opportunities.to_string(), "opportunities");
    }

    #[test]
    fn test_create_success_response() {
        let state = AuditState {
            audit_id: "audit-123-1".to_string(),
            path: PathBuf::from("/test/project"),
            sections: vec![AuditSection::Inventory, AuditSection::Dependencies],
            format: AuditOutputFormat::Json,
            started_at: 1234567890,
            completed: false,
            error: None,
        };

        let response = create_success_response(&state);

        assert!(response.success);
        assert_eq!(response.audit_id, Some("audit-123-1".to_string()));
        assert_eq!(response.path, Some("/test/project".to_string()));
        assert_eq!(
            response.sections,
            Some(vec!["inventory".to_string(), "dependencies".to_string()])
        );
        assert_eq!(response.format, Some("json".to_string()));
        assert!(response.message.contains("audit-123-1"));
    }

    #[test]
    fn test_create_error_response() {
        let error = StartAuditError::PathNotFound("/bad/path".to_string());
        let response = create_error_response(&error);

        assert!(!response.success);
        assert!(response.audit_id.is_none());
        assert!(response.path.is_none());
        assert!(response.sections.is_none());
        assert!(response.format.is_none());
        assert!(response.message.contains("/bad/path"));
    }

    #[test]
    fn test_start_audit_error_display() {
        assert!(StartAuditError::PathNotFound("/test".to_string())
            .to_string()
            .contains("Path not found"));

        assert!(StartAuditError::NotADirectory("/test".to_string())
            .to_string()
            .contains("not a directory"));

        assert!(StartAuditError::InvalidSection("bad".to_string())
            .to_string()
            .contains("Invalid audit section"));

        assert!(StartAuditError::InitializationError("failed".to_string())
            .to_string()
            .contains("Failed to initialize"));
    }

    #[test]
    fn test_start_audit_request_deserialization() {
        let json = r#"{"path": "/test/project", "sections": ["inventory", "dependencies"], "format": "markdown"}"#;
        let req: StartAuditRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.path, Some("/test/project".to_string()));
        assert_eq!(
            req.sections,
            Some(vec![AuditSection::Inventory, AuditSection::Dependencies])
        );
        assert_eq!(req.format, Some(AuditOutputFormat::Markdown));
    }

    #[test]
    fn test_start_audit_request_defaults() {
        let json = r#"{}"#;
        let req: StartAuditRequest = serde_json::from_str(json).unwrap();

        assert!(req.path.is_none());
        assert!(req.sections.is_none());
        assert!(req.format.is_none());
    }

    #[test]
    fn test_start_audit_response_serialization() {
        let response = StartAuditResponse {
            success: true,
            audit_id: Some("audit-123".to_string()),
            path: Some("/test".to_string()),
            sections: Some(vec!["inventory".to_string()]),
            format: Some("json".to_string()),
            message: "Success".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"audit_id\":\"audit-123\""));
        assert!(json.contains("\"path\":\"/test\""));
    }

    #[test]
    fn test_start_audit_response_none_fields_not_serialized() {
        let response = StartAuditResponse {
            success: false,
            audit_id: None,
            path: None,
            sections: None,
            format: None,
            message: "Error".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("audit_id"));
        assert!(!json.contains("path"));
        assert!(!json.contains("sections"));
        assert!(!json.contains("format"));
    }

    #[test]
    fn test_audit_state_clone() {
        let state = AuditState {
            audit_id: "audit-123".to_string(),
            path: PathBuf::from("/test"),
            sections: vec![AuditSection::Inventory],
            format: AuditOutputFormat::Json,
            started_at: 1234567890,
            completed: false,
            error: None,
        };

        let cloned = state.clone();
        assert_eq!(cloned.audit_id, state.audit_id);
        assert_eq!(cloned.path, state.path);
        assert_eq!(cloned.sections, state.sections);
    }
}
