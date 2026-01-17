//! Linear integration provider
//!
//! This module implements the ProjectTracker trait for Linear,
//! allowing Ralph to sync story progress to Linear issue boards.

#![allow(dead_code)]

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::traits::{
    CreateItemRequest, FailureIssueRequest, ItemInfo, ItemStatus, ProjectTracker, TrackerError,
    TrackerResult, UpdateItemRequest,
};

/// Linear GraphQL API endpoint
const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

/// Configuration for Linear provider
#[derive(Debug, Clone)]
pub struct LinearConfig {
    /// Linear API key
    pub api_key: String,
    /// Team ID to create issues in
    pub team_id: String,
}

impl LinearConfig {
    /// Create a new LinearConfig from environment variables
    ///
    /// Expects:
    /// - LINEAR_API_KEY: Linear API key
    /// - LINEAR_TEAM_ID: Team ID to create issues in
    pub fn from_env() -> TrackerResult<Self> {
        let api_key = std::env::var("LINEAR_API_KEY").map_err(|_| {
            TrackerError::ConfigError("LINEAR_API_KEY environment variable not set".to_string())
        })?;

        let team_id = std::env::var("LINEAR_TEAM_ID").map_err(|_| {
            TrackerError::ConfigError("LINEAR_TEAM_ID environment variable not set".to_string())
        })?;

        Ok(Self { api_key, team_id })
    }

    /// Create a new LinearConfig with explicit values
    pub fn new(api_key: String, team_id: String) -> Self {
        Self { api_key, team_id }
    }
}

/// Linear provider
///
/// Implements the ProjectTracker trait using Linear's GraphQL API
/// to manage issues in Linear.
pub struct LinearProvider {
    /// HTTP client for API requests
    client: Client,
    /// Provider configuration
    config: LinearConfig,
}

impl LinearProvider {
    /// Create a new Linear provider
    pub fn new(config: LinearConfig) -> TrackerResult<Self> {
        let client = Client::builder().build().map_err(|e| {
            TrackerError::ConfigError(format!("Failed to create HTTP client: {}", e))
        })?;

        Ok(Self { client, config })
    }

    /// Create a new provider from environment variables
    pub fn from_env() -> TrackerResult<Self> {
        let config = LinearConfig::from_env()?;
        Self::new(config)
    }

    /// Execute a GraphQL query against the Linear API
    async fn execute_graphql<T: for<'de> Deserialize<'de>>(&self, query: &str) -> TrackerResult<T> {
        let request_body = serde_json::json!({
            "query": query
        });

        let response = self
            .client
            .post(LINEAR_API_URL)
            .header("Authorization", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| TrackerError::ApiError(format!("HTTP request failed: {}", e)))?;

        // Check for HTTP errors
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(TrackerError::AuthenticationError(
                "Invalid Linear API key".to_string(),
            ));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(TrackerError::RateLimitError(
                "Linear API rate limit exceeded".to_string(),
            ));
        }
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TrackerError::ApiError(format!(
                "HTTP {} error: {}",
                status, error_text
            )));
        }

        let result: T = response
            .json()
            .await
            .map_err(|e| TrackerError::ApiError(format!("Failed to parse response: {}", e)))?;

        Ok(result)
    }

    /// Create an issue using the issueCreate mutation
    async fn create_issue(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> TrackerResult<LinearIssue> {
        let description_field = description
            .map(|d| format!(r#", description: "{}""#, escape_graphql_string(d)))
            .unwrap_or_default();

        let mutation = format!(
            r#"mutation {{
                issueCreate(input: {{
                    teamId: "{team_id}",
                    title: "{title}"{description}
                }}) {{
                    success
                    issue {{
                        id
                        identifier
                        title
                        url
                    }}
                }}
            }}"#,
            team_id = self.config.team_id,
            title = escape_graphql_string(title),
            description = description_field
        );

        let response: GraphQLResponse<IssueCreateData> = self.execute_graphql(&mutation).await?;

        // Check for GraphQL errors
        if let Some(errors) = response.errors {
            let error_msg = errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown GraphQL error".to_string());
            return Err(TrackerError::ApiError(error_msg));
        }

        let data = response
            .data
            .ok_or_else(|| TrackerError::ApiError("No data in response".to_string()))?;

        if !data.issue_create.success {
            return Err(TrackerError::ApiError("Issue creation failed".to_string()));
        }

        data.issue_create
            .issue
            .ok_or_else(|| TrackerError::ApiError("No issue in response".to_string()))
    }
}

/// Generic GraphQL response wrapper
#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error
#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

/// Response data from issueCreate mutation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IssueCreateData {
    issue_create: IssueCreateResult,
}

/// Result of issueCreate mutation
#[derive(Debug, Deserialize)]
struct IssueCreateResult {
    success: bool,
    issue: Option<LinearIssue>,
}

/// Linear issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearIssue {
    /// Internal Linear ID (UUID)
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123")
    pub identifier: String,
    /// Issue title
    pub title: String,
    /// URL to view the issue
    pub url: String,
}

/// Escape special characters for GraphQL string values
fn escape_graphql_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[async_trait]
impl ProjectTracker for LinearProvider {
    fn name(&self) -> &str {
        "linear"
    }

    async fn create_item(&self, request: CreateItemRequest) -> TrackerResult<ItemInfo> {
        let issue = self
            .create_issue(&request.title, request.description.as_deref())
            .await?;

        Ok(ItemInfo {
            id: issue.id,
            title: issue.title,
            url: Some(issue.url),
        })
    }

    async fn update_item(
        &self,
        _item_id: &str,
        _request: UpdateItemRequest,
    ) -> TrackerResult<ItemInfo> {
        // Will be implemented in US-033
        Err(TrackerError::ApiError(
            "update_item not yet implemented".to_string(),
        ))
    }

    async fn create_failure_issue(&self, _request: FailureIssueRequest) -> TrackerResult<ItemInfo> {
        // Will be implemented in US-033
        Err(TrackerError::ApiError(
            "create_failure_issue not yet implemented".to_string(),
        ))
    }

    async fn add_comment(&self, _item_id: &str, _comment: &str) -> TrackerResult<()> {
        // Will be implemented in US-033
        Err(TrackerError::ApiError(
            "add_comment not yet implemented".to_string(),
        ))
    }

    async fn update_status(&self, _item_id: &str, _status: ItemStatus) -> TrackerResult<ItemInfo> {
        // Will be implemented in US-033
        Err(TrackerError::ApiError(
            "update_status not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_config_new() {
        let config = LinearConfig::new("lin_api_test_key".to_string(), "team_test_id".to_string());

        assert_eq!(config.api_key, "lin_api_test_key");
        assert_eq!(config.team_id, "team_test_id");
    }

    #[test]
    fn test_escape_graphql_string() {
        assert_eq!(escape_graphql_string("hello"), "hello");
        assert_eq!(escape_graphql_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_graphql_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_graphql_string("tab\there"), "tab\\there");
        assert_eq!(escape_graphql_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_escape_graphql_string_complex() {
        let input = "Line 1\nLine 2\r\nWith \"quotes\" and \\backslash";
        let escaped = escape_graphql_string(input);
        assert_eq!(
            escaped,
            "Line 1\\nLine 2\\r\\nWith \\\"quotes\\\" and \\\\backslash"
        );
    }

    #[test]
    fn test_linear_issue_serialize() {
        let issue = LinearIssue {
            id: "abc123".to_string(),
            identifier: "ENG-42".to_string(),
            title: "Test Issue".to_string(),
            url: "https://linear.app/team/issue/ENG-42".to_string(),
        };

        let json = serde_json::to_string(&issue).unwrap();
        assert!(json.contains("\"id\":\"abc123\""));
        assert!(json.contains("\"identifier\":\"ENG-42\""));
        assert!(json.contains("\"title\":\"Test Issue\""));
        assert!(json.contains("\"url\":\"https://linear.app/team/issue/ENG-42\""));
    }

    #[test]
    fn test_linear_issue_deserialize() {
        let json = r#"{
            "id": "uuid-123",
            "identifier": "ENG-100",
            "title": "Deserialized Issue",
            "url": "https://linear.app/test"
        }"#;

        let issue: LinearIssue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.id, "uuid-123");
        assert_eq!(issue.identifier, "ENG-100");
        assert_eq!(issue.title, "Deserialized Issue");
        assert_eq!(issue.url, "https://linear.app/test");
    }

    #[test]
    fn test_linear_config_from_env_missing_vars() {
        // Save original values
        let orig_api_key = std::env::var("LINEAR_API_KEY").ok();
        let orig_team_id = std::env::var("LINEAR_TEAM_ID").ok();

        // Remove env vars
        std::env::remove_var("LINEAR_API_KEY");
        std::env::remove_var("LINEAR_TEAM_ID");

        let result = LinearConfig::from_env();

        // Restore original values
        if let Some(v) = orig_api_key {
            std::env::set_var("LINEAR_API_KEY", v);
        }
        if let Some(v) = orig_team_id {
            std::env::set_var("LINEAR_TEAM_ID", v);
        }

        // Should get a ConfigError
        match result {
            Err(TrackerError::ConfigError(msg)) => {
                assert!(msg.contains("LINEAR_API_KEY"));
            }
            _ => panic!("Expected ConfigError, got: {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_provider_name() {
        let config = LinearConfig::new("test_key".to_string(), "team_id".to_string());
        let provider = LinearProvider::new(config).unwrap();
        assert_eq!(provider.name(), "linear");
    }

    #[tokio::test]
    async fn test_update_item_not_implemented() {
        let config = LinearConfig::new("test_key".to_string(), "team_id".to_string());
        let provider = LinearProvider::new(config).unwrap();
        let request = UpdateItemRequest {
            title: None,
            description: None,
            status: Some(ItemStatus::Done),
            add_labels: vec![],
            remove_labels: vec![],
        };

        let result = provider.update_item("item-123", request).await;
        assert!(result.is_err());
        if let Err(TrackerError::ApiError(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        }
    }

    #[tokio::test]
    async fn test_add_comment_not_implemented() {
        let config = LinearConfig::new("test_key".to_string(), "team_id".to_string());
        let provider = LinearProvider::new(config).unwrap();

        let result = provider.add_comment("item-123", "Test comment").await;
        assert!(result.is_err());
        if let Err(TrackerError::ApiError(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        }
    }

    #[tokio::test]
    async fn test_update_status_not_implemented() {
        let config = LinearConfig::new("test_key".to_string(), "team_id".to_string());
        let provider = LinearProvider::new(config).unwrap();

        let result = provider.update_status("item-123", ItemStatus::Done).await;
        assert!(result.is_err());
        if let Err(TrackerError::ApiError(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        }
    }

    #[tokio::test]
    async fn test_create_failure_issue_not_implemented() {
        let config = LinearConfig::new("test_key".to_string(), "team_id".to_string());
        let provider = LinearProvider::new(config).unwrap();
        let request = FailureIssueRequest {
            story_id: "US-001".to_string(),
            story_title: "Test story".to_string(),
            error: "Test error".to_string(),
            context: None,
        };

        let result = provider.create_failure_issue(request).await;
        assert!(result.is_err());
        if let Err(TrackerError::ApiError(msg)) = result {
            assert!(msg.contains("not yet implemented"));
        }
    }

    #[test]
    fn test_graphql_response_deserialize() {
        let json = r#"{
            "data": {
                "issueCreate": {
                    "success": true,
                    "issue": {
                        "id": "test-id",
                        "identifier": "ENG-1",
                        "title": "Test",
                        "url": "https://linear.app/test"
                    }
                }
            }
        }"#;

        let response: GraphQLResponse<IssueCreateData> = serde_json::from_str(json).unwrap();
        assert!(response.errors.is_none());
        let data = response.data.unwrap();
        assert!(data.issue_create.success);
        let issue = data.issue_create.issue.unwrap();
        assert_eq!(issue.id, "test-id");
        assert_eq!(issue.identifier, "ENG-1");
    }

    #[test]
    fn test_graphql_error_deserialize() {
        let json = r#"{
            "errors": [
                {"message": "Invalid API key"}
            ]
        }"#;

        let response: GraphQLResponse<IssueCreateData> = serde_json::from_str(json).unwrap();
        assert!(response.data.is_none());
        let errors = response.errors.unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Invalid API key");
    }

    #[test]
    fn test_create_item_request_construction() {
        let request = CreateItemRequest {
            title: "New Linear Issue".to_string(),
            description: Some("Issue description".to_string()),
            status: Some(ItemStatus::Todo),
            labels: vec!["bug".to_string()],
        };

        assert_eq!(request.title, "New Linear Issue");
        assert_eq!(request.description, Some("Issue description".to_string()));
        assert_eq!(request.status, Some(ItemStatus::Todo));
        assert_eq!(request.labels, vec!["bug".to_string()]);
    }
}
