// MCP Server implementation for Ralph
// This module provides the core MCP server struct

#![allow(dead_code)]

/// RalphMcpServer - The main MCP server struct for Ralph
///
/// This server exposes Ralph's functionality via the Model Context Protocol,
/// allowing AI assistants to interact with Ralph's PRD management, story execution,
/// and quality checking capabilities.
pub struct RalphMcpServer {
    // Fields will be added in US-015
}

impl RalphMcpServer {
    /// Create a new RalphMcpServer instance
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RalphMcpServer {
    fn default() -> Self {
        Self::new()
    }
}
