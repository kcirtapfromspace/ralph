// MCP Tool: run_all
// This module implements the run_all tool for executing all failing stories until complete

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request parameters for the run_all tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunAllRequest {
    /// Maximum total iterations across all stories (0 = unlimited)
    #[serde(default)]
    pub max_iterations: Option<u32>,
    /// Maximum iterations per individual story (default: 10)
    #[serde(default)]
    pub max_iterations_per_story: Option<u32>,
}

/// Response from the run_all tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAllResponse {
    /// Whether execution started successfully
    pub success: bool,
    /// Number of stories that need to be executed
    pub stories_to_execute: usize,
    /// Total number of stories in the PRD
    pub total_stories: usize,
    /// Status message
    pub message: String,
}

/// Progress update during run_all execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAllProgress {
    /// Current story being executed
    pub current_story_id: String,
    /// Current story title
    pub current_story_title: String,
    /// Number of stories completed so far
    pub stories_completed: usize,
    /// Total stories that need to pass
    pub total_stories: usize,
    /// Current iteration within the story
    pub current_iteration: u32,
    /// Max iterations for current story
    pub max_iterations: u32,
}

/// Final result from run_all execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAllResult {
    /// Whether all stories passed
    pub all_passed: bool,
    /// Number of stories that passed
    pub stories_passed: usize,
    /// Total number of stories
    pub total_stories: usize,
    /// Total iterations used
    pub total_iterations: u32,
    /// Error message if failed
    pub error: Option<String>,
    /// Completion message (includes <promise>COMPLETE</promise> if all passed)
    pub message: String,
}

/// Create a success response indicating execution has started
pub fn create_started_response(stories_to_execute: usize, total_stories: usize) -> RunAllResponse {
    RunAllResponse {
        success: true,
        stories_to_execute,
        total_stories,
        message: format!(
            "Started execution of {} failing stories out of {} total. Use get_status to monitor progress.",
            stories_to_execute, total_stories
        ),
    }
}

/// Create an error response
pub fn create_error_response(error: &str) -> RunAllResponse {
    RunAllResponse {
        success: false,
        stories_to_execute: 0,
        total_stories: 0,
        message: error.to_string(),
    }
}

/// Create the final result response
pub fn create_result(
    all_passed: bool,
    stories_passed: usize,
    total_stories: usize,
    total_iterations: u32,
    error: Option<String>,
) -> RunAllResult {
    let message = if all_passed {
        format!(
            "All {} stories passed!\n<promise>COMPLETE</promise>",
            total_stories
        )
    } else if let Some(ref err) = error {
        format!(
            "Execution stopped: {}. {}/{} stories passed.",
            err, stories_passed, total_stories
        )
    } else {
        format!(
            "Execution incomplete: {}/{} stories passed.",
            stories_passed, total_stories
        )
    };

    RunAllResult {
        all_passed,
        stories_passed,
        total_stories,
        total_iterations,
        error,
        message,
    }
}
