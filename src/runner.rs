// Terminal runner for Ralph
// This module implements the default "run all stories until complete" behavior

use std::io::{self, Write};
use std::path::PathBuf;
use tokio::sync::watch;

use chrono::Utc;

use crate::checkpoint::{Checkpoint, CheckpointManager, PauseReason, StoryCheckpoint};
use crate::mcp::tools::executor::{detect_agent, ExecutorConfig, ExecutorError, StoryExecutor};
use crate::mcp::tools::load_prd::{PrdFile, PrdUserStory};
use crate::parallel::scheduler::ParallelRunnerConfig;
use crate::ui::{DisplayOptions, TuiRunnerDisplay};

/// User's choice when prompted about an existing checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeChoice {
    /// Resume execution from the checkpoint.
    Resume,
    /// Discard the checkpoint and start fresh.
    Discard,
    /// Show detailed checkpoint information.
    ViewDetails,
}

/// Configuration for the runner
#[derive(Debug, Clone)]
#[allow(dead_code)] // parallel fields will be used in future stories
pub struct RunnerConfig {
    /// Path to the PRD file (defaults to "prd.json" in current dir)
    pub prd_path: PathBuf,
    /// Working directory (defaults to current dir)
    pub working_dir: PathBuf,
    /// Maximum iterations per story
    pub max_iterations_per_story: u32,
    /// Maximum total iterations across all stories (0 = unlimited)
    pub max_total_iterations: u32,
    /// Agent command to use (auto-detect if None)
    pub agent_command: Option<String>,
    /// Display options for UI rendering (includes quiet mode, verbosity, etc.)
    pub display_options: DisplayOptions,
    /// Enable parallel execution mode
    pub parallel: bool,
    /// Configuration for parallel execution (used when parallel is true)
    pub parallel_config: Option<ParallelRunnerConfig>,
    /// Resume from checkpoint if available
    pub resume: bool,
    /// Skip checkpoint prompt (do not resume)
    pub no_resume: bool,
    /// Agent timeout override in seconds (None = use default)
    pub timeout_seconds: Option<u64>,
    /// Disable checkpointing
    pub no_checkpoint: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            prd_path: PathBuf::from("prd.json"),
            working_dir: PathBuf::from("."),
            max_iterations_per_story: 10,
            max_total_iterations: 0, // unlimited
            agent_command: None,
            display_options: DisplayOptions::default(),
            parallel: false,
            parallel_config: None,
            resume: false,
            no_resume: false,
            timeout_seconds: None,
            no_checkpoint: false,
        }
    }
}

/// Result of running all stories
#[derive(Debug)]
#[allow(dead_code)] // Fields may be used by callers
pub struct RunResult {
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
}

/// The main runner that iterates through stories
pub struct Runner {
    config: RunnerConfig,
    /// Optional checkpoint manager (None if checkpointing is disabled)
    checkpoint_manager: Option<CheckpointManager>,
}

impl Runner {
    /// Create a new runner with the given configuration
    pub fn new(config: RunnerConfig) -> Self {
        // Initialize checkpoint manager if checkpointing is enabled
        let checkpoint_manager = if config.no_checkpoint {
            None
        } else {
            match CheckpointManager::new(&config.working_dir) {
                Ok(manager) => Some(manager),
                Err(e) => {
                    // Log warning but continue without checkpointing
                    eprintln!("Warning: Failed to initialize checkpoint manager: {}", e);
                    None
                }
            }
        };

        Self {
            config,
            checkpoint_manager,
        }
    }

    /// Run all stories until all pass or an error occurs.
    ///
    /// Routes to parallel or sequential execution based on config.parallel.
    pub async fn run(&self) -> RunResult {
        if self.config.parallel {
            // Use parallel execution
            let parallel_config = self.config.parallel_config.clone().unwrap_or_default();
            let parallel_runner = crate::parallel::scheduler::ParallelRunner::new(
                parallel_config,
                self.config.clone(),
            );
            parallel_runner.run().await
        } else {
            // Use sequential execution
            self.run_sequential().await
        }
    }

    /// Run all stories sequentially until all pass or an error occurs
    async fn run_sequential(&self) -> RunResult {
        let mut total_iterations: u32 = 0;

        // Create TUI display with display options
        let mut display =
            TuiRunnerDisplay::with_display_options(self.config.display_options.clone());

        // Load and validate PRD
        let prd = match self.load_prd() {
            Ok(prd) => prd,
            Err(e) => {
                return RunResult {
                    all_passed: false,
                    stories_passed: 0,
                    total_stories: 0,
                    total_iterations: 0,
                    error: Some(format!("Failed to load PRD: {}", e)),
                };
            }
        };

        let total_stories = prd.user_stories.len();

        // Initialize display with story list
        let story_status: Vec<(String, bool)> = prd
            .user_stories
            .iter()
            .map(|s| (s.id.clone(), s.passes))
            .collect();
        display.init_stories(story_status);

        // Check if all stories already pass
        let passing_count = prd.user_stories.iter().filter(|s| s.passes).count();
        if passing_count == total_stories {
            display.display_all_complete(total_stories);
            return RunResult {
                all_passed: true,
                stories_passed: total_stories,
                total_stories,
                total_iterations: 0,
                error: None,
            };
        }

        // Detect agent (only needed if there are failing stories)
        let agent = match self.config.agent_command.clone().or_else(detect_agent) {
            Some(a) => a,
            None => {
                return RunResult {
                    all_passed: false,
                    stories_passed: passing_count,
                    total_stories,
                    total_iterations: 0,
                    error: Some("No agent found. Install Claude Code CLI or Amp CLI.".to_string()),
                };
            }
        };

        // Display startup banner
        display.display_startup(
            &self.config.prd_path.display().to_string(),
            &agent,
            passing_count,
            total_stories,
        );

        // Main loop - continue until all stories pass
        loop {
            // Reload PRD each iteration to get updated passes status
            let prd = match self.load_prd() {
                Ok(prd) => prd,
                Err(e) => {
                    return RunResult {
                        all_passed: false,
                        stories_passed: self.count_passing_stories().unwrap_or(0),
                        total_stories,
                        total_iterations,
                        error: Some(format!("Failed to reload PRD: {}", e)),
                    };
                }
            };

            // Update display with current story states
            let story_status: Vec<(String, bool)> = prd
                .user_stories
                .iter()
                .map(|s| (s.id.clone(), s.passes))
                .collect();
            display.init_stories(story_status);

            // Find next story to work on (highest priority where passes: false)
            let next_story = self.find_next_story(&prd);

            match next_story {
                None => {
                    // All stories pass! Clear checkpoint on full completion.
                    self.clear_checkpoint();
                    display.display_all_complete(total_stories);
                    return RunResult {
                        all_passed: true,
                        stories_passed: total_stories,
                        total_stories,
                        total_iterations,
                        error: None,
                    };
                }
                Some(story) => {
                    // Check total iteration limit
                    if self.config.max_total_iterations > 0
                        && total_iterations >= self.config.max_total_iterations
                    {
                        // Save checkpoint on reaching iteration limit
                        self.save_checkpoint(
                            &story.id,
                            1,
                            self.config.max_iterations_per_story,
                            PauseReason::Error(format!(
                                "Max total iterations ({}) reached",
                                self.config.max_total_iterations
                            )),
                        );
                        return RunResult {
                            all_passed: false,
                            stories_passed: self.count_passing_stories().unwrap_or(0),
                            total_stories,
                            total_iterations,
                            error: Some(format!(
                                "Max total iterations ({}) reached",
                                self.config.max_total_iterations
                            )),
                        };
                    }

                    // Display story start
                    display.start_story(&story.id, &story.title, story.priority);

                    // Execute the story
                    let executor_config = ExecutorConfig {
                        prd_path: self.config.prd_path.clone(),
                        project_root: self.config.working_dir.clone(),
                        progress_path: self.config.working_dir.join("progress.txt"),
                        quality_profile: None,
                        agent_command: agent.clone(),
                        max_iterations: self.config.max_iterations_per_story,
                        git_mutex: None, // Sequential execution doesn't need mutex
                        timeout_config: crate::timeout::TimeoutConfig::default(),
                    };

                    let executor = StoryExecutor::new(executor_config);
                    let (_cancel_tx, cancel_rx) = watch::channel(false);

                    let story_id = story.id.clone();
                    let max_iterations = self.config.max_iterations_per_story;

                    // Save checkpoint before starting story execution
                    self.save_checkpoint(&story_id, 1, max_iterations, PauseReason::UserRequested);

                    let result = executor
                        .execute_story(&story_id, cancel_rx, |iter, max| {
                            display.update_iteration(iter, max);
                        })
                        .await;

                    total_iterations += result.as_ref().map(|r| r.iterations_used).unwrap_or(1);

                    match result {
                        Ok(exec_result) => {
                            if exec_result.success {
                                // Clear checkpoint on successful story completion
                                self.clear_checkpoint();
                                display
                                    .complete_story(&story_id, exec_result.commit_hash.as_deref());
                            } else {
                                // Save checkpoint on story failure (quality gates didn't pass)
                                self.save_checkpoint(
                                    &story_id,
                                    exec_result.iterations_used,
                                    max_iterations,
                                    PauseReason::Error(
                                        exec_result
                                            .error
                                            .clone()
                                            .unwrap_or_else(|| "Quality gates failed".to_string()),
                                    ),
                                );
                                display.fail_story(
                                    &story_id,
                                    exec_result.error.as_deref().unwrap_or("unknown"),
                                );
                            }
                        }
                        Err(e) => {
                            // Save checkpoint on executor error
                            self.save_checkpoint(
                                &story_id,
                                1,
                                max_iterations,
                                Self::error_to_pause_reason(&e),
                            );
                            display.fail_story(&story_id, &e.to_string());
                            // Don't fail the whole run, just continue
                        }
                    }
                }
            }
        }
    }

    /// Load the PRD file
    fn load_prd(&self) -> Result<PrdFile, String> {
        let content = std::fs::read_to_string(&self.config.prd_path)
            .map_err(|e| format!("Failed to read {}: {}", self.config.prd_path.display(), e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse PRD: {}", e))
    }

    /// Find the next story to work on (highest priority where passes: false)
    fn find_next_story<'a>(&self, prd: &'a PrdFile) -> Option<&'a PrdUserStory> {
        prd.user_stories
            .iter()
            .filter(|s| !s.passes)
            .min_by_key(|s| s.priority) // Lower priority number = higher priority
    }

    /// Count stories that are currently passing
    fn count_passing_stories(&self) -> Result<usize, String> {
        let prd = self.load_prd()?;
        Ok(prd.user_stories.iter().filter(|s| s.passes).count())
    }

    /// Save a checkpoint with the current execution state.
    ///
    /// Does nothing if checkpointing is disabled.
    fn save_checkpoint(
        &self,
        story_id: &str,
        iteration: u32,
        max_iterations: u32,
        pause_reason: PauseReason,
    ) {
        if let Some(ref manager) = self.checkpoint_manager {
            let uncommitted_files = self.get_uncommitted_files().unwrap_or_default();
            let checkpoint = Checkpoint::new(
                Some(StoryCheckpoint::new(story_id, iteration, max_iterations)),
                pause_reason,
                uncommitted_files,
            );

            if let Err(e) = manager.save(&checkpoint) {
                eprintln!("Warning: Failed to save checkpoint: {}", e);
            }
        }
    }

    /// Clear the checkpoint (called on successful completion).
    ///
    /// Does nothing if checkpointing is disabled.
    fn clear_checkpoint(&self) {
        if let Some(ref manager) = self.checkpoint_manager {
            if let Err(e) = manager.clear() {
                eprintln!("Warning: Failed to clear checkpoint: {}", e);
            }
        }
    }

    /// Get list of uncommitted files from git.
    fn get_uncommitted_files(&self) -> Result<Vec<String>, String> {
        use std::process::Command;

        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.config.working_dir)
            .output()
            .map_err(|e| format!("Failed to run git status: {}", e))?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.len() > 3 {
                    Some(line[3..].to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(files)
    }

    /// Convert an ExecutorError to a PauseReason for checkpointing.
    fn error_to_pause_reason(error: &ExecutorError) -> PauseReason {
        match error {
            ExecutorError::Timeout(_) => PauseReason::Timeout,
            ExecutorError::Cancelled => PauseReason::UserRequested,
            _ => PauseReason::Error(error.to_string()),
        }
    }
}
