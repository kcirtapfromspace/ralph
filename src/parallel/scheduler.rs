//! Parallel execution scheduler

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{RwLock, Semaphore};

use crate::runner::RunnerConfig;

/// Configuration options for parallel story execution.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ParallelRunnerConfig {
    /// Maximum number of stories to execute concurrently.
    pub max_concurrency: u32,
    /// Whether to automatically infer dependencies from file patterns.
    pub infer_dependencies: bool,
    /// Whether to fall back to sequential execution on errors.
    pub fallback_to_sequential: bool,
}

impl Default for ParallelRunnerConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 3,
            infer_dependencies: true,
            fallback_to_sequential: true,
        }
    }
}

/// Tracks execution state across parallel story executions.
///
/// This struct maintains the runtime state for the parallel scheduler,
/// including which stories are currently executing, which have completed,
/// which have failed, and which files are locked.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct ParallelExecutionState {
    /// Stories currently being executed, mapped by story ID.
    pub in_flight: HashSet<String>,
    /// Stories that have completed successfully, mapped by story ID.
    pub completed: HashSet<String>,
    /// Stories that have failed, mapped by story ID to error message.
    pub failed: HashMap<String, String>,
    /// Files currently locked by stories, mapped from file path to story ID.
    pub locked_files: HashMap<PathBuf, String>,
}

/// The main parallel runner that executes multiple stories concurrently.
///
/// This struct manages parallel story execution with concurrency limiting
/// via a semaphore and shared execution state protected by a read-write lock.
#[allow(dead_code)]
pub struct ParallelRunner {
    /// Configuration for parallel execution settings.
    config: ParallelRunnerConfig,
    /// Base runner configuration (paths, limits, etc.).
    base_config: RunnerConfig,
    /// Semaphore for limiting concurrent story executions.
    semaphore: Arc<Semaphore>,
    /// Shared execution state tracking in-flight, completed, and failed stories.
    execution_state: Arc<RwLock<ParallelExecutionState>>,
}

#[allow(dead_code)]
impl ParallelRunner {
    /// Create a new parallel runner with the given configurations.
    ///
    /// # Arguments
    /// * `config` - Parallel execution settings (concurrency, inference, fallback)
    /// * `base_config` - Base runner configuration (paths, iteration limits)
    pub fn new(config: ParallelRunnerConfig, base_config: RunnerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency as usize));
        let execution_state = Arc::new(RwLock::new(ParallelExecutionState::default()));

        Self {
            config,
            base_config,
            semaphore,
            execution_state,
        }
    }
}
