//! Main display controller for Ralph's terminal UI.
//!
//! Coordinates all UI components and manages terminal output.

#![allow(dead_code)]

use crate::ui::colors::Theme;
use crate::ui::spinner::{IterationProgress, ProgressManager, RalphSpinner};

/// Main display controller for Ralph's terminal output.
///
/// Coordinates rendering of story panels, progress indicators,
/// quality gates, and other UI components.
#[derive(Debug)]
pub struct RalphDisplay {
    /// Color theme for terminal output
    theme: Theme,
    /// Whether colors are enabled (respects NO_COLOR env var)
    colors_enabled: bool,
    /// Whether the terminal supports advanced features
    advanced_features: bool,
    /// Progress manager for handling multiple progress indicators
    progress_manager: ProgressManager,
    /// Current active spinner (if any)
    active_spinner: Option<RalphSpinner>,
    /// Current iteration progress bar (if any)
    iteration_progress: Option<IterationProgress>,
}

impl Default for RalphDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl RalphDisplay {
    /// Create a new RalphDisplay with default settings.
    pub fn new() -> Self {
        let theme = Theme::default();
        Self {
            theme,
            colors_enabled: Self::detect_color_support(),
            advanced_features: Self::detect_advanced_features(),
            progress_manager: ProgressManager::with_theme(theme),
            active_spinner: None,
            iteration_progress: None,
        }
    }

    /// Create a RalphDisplay with a custom theme.
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme,
            colors_enabled: Self::detect_color_support(),
            advanced_features: Self::detect_advanced_features(),
            progress_manager: ProgressManager::with_theme(theme),
            active_spinner: None,
            iteration_progress: None,
        }
    }

    /// Get the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Check if colors are enabled.
    pub fn colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    /// Enable or disable colors.
    pub fn set_colors_enabled(&mut self, enabled: bool) {
        self.colors_enabled = enabled;
    }

    /// Check if advanced terminal features are available.
    pub fn advanced_features(&self) -> bool {
        self.advanced_features
    }

    /// Detect if color output should be enabled.
    ///
    /// Respects the NO_COLOR environment variable.
    fn detect_color_support() -> bool {
        std::env::var("NO_COLOR").is_err()
    }

    /// Detect if advanced terminal features are available.
    ///
    /// Checks for Ghostty or other modern terminal emulators.
    fn detect_advanced_features() -> bool {
        // Check for Ghostty
        if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
            return true;
        }

        // Check for other modern terminals that support 24-bit color
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") || term.contains("truecolor") {
                return true;
            }
        }

        // Check COLORTERM for truecolor support
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return true;
            }
        }

        false
    }

    // =========================================================================
    // Spinner Management
    // =========================================================================

    /// Start a spinner with the given action message.
    ///
    /// If a spinner is already active, it will be stopped first.
    /// The spinner displays elapsed time and the current action.
    pub fn start_spinner(&mut self, message: impl Into<String>) {
        // Stop any existing spinner first
        self.stop_spinner();

        let spinner = self.progress_manager.add_spinner(message);
        self.active_spinner = Some(spinner);
    }

    /// Stop the current spinner with a success message.
    ///
    /// If no spinner is active, this is a no-op.
    pub fn stop_spinner_with_success(&mut self, message: impl Into<String>) {
        if let Some(spinner) = self.active_spinner.take() {
            spinner.finish_with_success(message);
        }
    }

    /// Stop the current spinner with an error message.
    ///
    /// If no spinner is active, this is a no-op.
    pub fn stop_spinner_with_error(&mut self, message: impl Into<String>) {
        if let Some(spinner) = self.active_spinner.take() {
            spinner.finish_with_error(message);
        }
    }

    /// Stop the current spinner and clear it from the display.
    ///
    /// If no spinner is active, this is a no-op.
    pub fn stop_spinner(&mut self) {
        if let Some(spinner) = self.active_spinner.take() {
            spinner.finish_and_clear();
        }
    }

    /// Update the message on the current spinner.
    ///
    /// If no spinner is active, this is a no-op.
    pub fn update_spinner_message(&self, message: impl Into<String>) {
        if let Some(ref spinner) = self.active_spinner {
            spinner.set_message(message);
        }
    }

    /// Check if a spinner is currently active.
    pub fn has_active_spinner(&self) -> bool {
        self.active_spinner.is_some()
    }

    // =========================================================================
    // Iteration Progress Management
    // =========================================================================

    /// Start an iteration progress bar with the given total iterations.
    ///
    /// If a progress bar is already active, it will be stopped first.
    pub fn start_iteration_progress(&mut self, total: u64) {
        // Stop any existing progress bar first
        self.stop_iteration_progress();

        let progress = self.progress_manager.add_iteration_progress(total);
        self.iteration_progress = Some(progress);
    }

    /// Increment the iteration progress by one.
    ///
    /// If no progress bar is active, this is a no-op.
    pub fn inc_iteration(&mut self) {
        if let Some(ref mut progress) = self.iteration_progress {
            progress.inc();
        }
    }

    /// Set the current iteration position.
    ///
    /// If no progress bar is active, this is a no-op.
    pub fn set_iteration(&mut self, pos: u64) {
        if let Some(ref mut progress) = self.iteration_progress {
            progress.set_position(pos);
        }
    }

    /// Get the current iteration count.
    ///
    /// Returns 0 if no progress bar is active.
    pub fn current_iteration(&self) -> u64 {
        self.iteration_progress
            .as_ref()
            .map(|p| p.current())
            .unwrap_or(0)
    }

    /// Get the total iteration count.
    ///
    /// Returns 0 if no progress bar is active.
    pub fn total_iterations(&self) -> u64 {
        self.iteration_progress
            .as_ref()
            .map(|p| p.total())
            .unwrap_or(0)
    }

    /// Stop and clear the iteration progress bar.
    ///
    /// If no progress bar is active, this is a no-op.
    pub fn stop_iteration_progress(&mut self) {
        if let Some(progress) = self.iteration_progress.take() {
            progress.finish_and_clear();
        }
    }

    /// Finish the iteration progress bar (keeps it visible).
    ///
    /// If no progress bar is active, this is a no-op.
    pub fn finish_iteration_progress(&mut self) {
        if let Some(progress) = self.iteration_progress.take() {
            progress.finish();
        }
    }

    /// Get the progress manager for advanced multi-progress use.
    pub fn progress_manager(&self) -> &ProgressManager {
        &self.progress_manager
    }
}
