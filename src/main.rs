use clap::{Parser, ValueEnum};
use rmcp::{transport::stdio, ServiceExt};
use std::path::PathBuf;

mod integrations;
mod mcp;
mod quality;
mod ui;

use mcp::RalphMcpServer;
use ui::{DisplayOptions, UiMode};

/// UI mode for terminal display
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum CliUiMode {
    /// Auto-detect based on terminal capabilities
    #[default]
    Auto,
    /// Force enable rich terminal UI
    Enabled,
    /// Force disable rich terminal UI (plain text only)
    Disabled,
}

impl From<CliUiMode> for UiMode {
    fn from(mode: CliUiMode) -> Self {
        match mode {
            CliUiMode::Auto => UiMode::Auto,
            CliUiMode::Enabled => UiMode::Enabled,
            CliUiMode::Disabled => UiMode::Disabled,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "ralph")]
#[command(version)]
#[command(about = "Enterprise-ready autonomous AI agent framework")]
struct Cli {
    /// UI mode: auto (default), enabled, or disabled
    #[arg(long, default_value = "auto", value_enum)]
    ui: CliUiMode,

    /// Disable colors (also respects NO_COLOR environment variable)
    #[arg(long)]
    no_color: bool,

    /// Suppress all output except errors
    #[arg(long, short)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Run quality checks
    Quality,
    /// Start MCP server mode for integration with AI assistants
    McpServer {
        /// Path to PRD file to preload (optional)
        #[arg(long)]
        prd: Option<PathBuf>,
    },
}

/// Build display options from CLI arguments
fn build_display_options(cli: &Cli) -> DisplayOptions {
    DisplayOptions::new()
        .with_ui_mode(cli.ui.into())
        .with_color(!cli.no_color)
        .with_quiet(cli.quiet)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Build display options from CLI flags
    let display_options = build_display_options(&cli);

    match cli.command {
        Some(Commands::Quality) => {
            // Initialize logging to stdout for quality checks (unless quiet)
            if !cli.quiet {
                tracing_subscriber::fmt::init();
                println!("Running quality checks...");
            }
        }
        Some(Commands::McpServer { prd }) => {
            // Configure logging to stderr only for MCP server mode
            // (stdout is reserved for MCP protocol communication)
            if !cli.quiet {
                tracing_subscriber::fmt()
                    .with_writer(std::io::stderr)
                    .init();
            }

            // Create the server, optionally with a preloaded PRD
            let server = match prd {
                Some(path) => {
                    if !cli.quiet {
                        tracing::info!("Starting MCP server with preloaded PRD: {:?}", path);
                    }
                    RalphMcpServer::with_prd_and_display(path, display_options)
                }
                None => {
                    if !cli.quiet {
                        tracing::info!("Starting MCP server");
                    }
                    RalphMcpServer::with_display(display_options)
                }
            };

            // Start the MCP server using stdio transport
            let service = server.serve(stdio()).await.map_err(|e| {
                tracing::error!("Error starting MCP server: {}", e);
                e
            })?;

            // Wait for the service to complete
            service.waiting().await?;
        }
        None => {
            // Initialize logging to stdout for default mode (unless quiet)
            if !cli.quiet {
                tracing_subscriber::fmt::init();
                println!("Ralph - Enterprise-ready autonomous AI agent framework");
                println!("Use --help for available commands");
            }
        }
    }

    Ok(())
}
