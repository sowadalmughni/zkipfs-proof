//! Progress tracking utilities for CLI operations
//!
//! This module provides progress tracking and user feedback for long-running
//! operations like proof generation and verification.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Progress tracker for CLI operations
pub struct ProgressTracker {
    bar: ProgressBar,
    operation_name: String,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(operation_name: &str) -> Self {
        let bar = ProgressBar::new(100);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}% {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  ")
        );
        
        Self {
            bar,
            operation_name: operation_name.to_string(),
        }
    }

    /// Set the current progress (0-100)
    pub fn set_progress(&self, progress: u64) {
        self.bar.set_position(progress);
    }

    /// Increment progress by a certain amount
    pub fn inc_progress(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// Set the current message
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish the progress bar with a success message
    pub fn finish(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    /// Finish the progress bar with an error message
    pub fn finish_with_error(&self, message: &str) {
        self.bar.abandon_with_message(format!("❌ {}", message));
    }

    /// Create a spinner for indeterminate progress
    pub fn spinner(operation_name: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        bar.enable_steady_tick(Duration::from_millis(100));
        
        Self {
            bar,
            operation_name: operation_name.to_string(),
        }
    }

    /// Create a progress bar for file operations
    pub fn file_progress(file_size: u64, operation_name: &str) -> Self {
        let bar = ProgressBar::new(file_size);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏  ")
        );
        
        Self {
            bar,
            operation_name: operation_name.to_string(),
        }
    }

    /// Update file progress with bytes processed
    pub fn set_file_progress(&self, bytes_processed: u64) {
        self.bar.set_position(bytes_processed);
    }
}

impl Drop for ProgressTracker {
    fn drop(&mut self) {
        if !self.bar.is_finished() {
            self.bar.abandon();
        }
    }
}

/// Multi-step progress tracker for complex operations
pub struct MultiStepProgress {
    steps: Vec<String>,
    current_step: usize,
    step_progress: ProgressTracker,
}

impl MultiStepProgress {
    /// Create a new multi-step progress tracker
    pub fn new(steps: Vec<String>) -> Self {
        let first_step = steps.first().cloned().unwrap_or_else(|| "Starting...".to_string());
        let step_progress = ProgressTracker::new(&first_step);
        
        Self {
            steps,
            current_step: 0,
            step_progress,
        }
    }

    /// Move to the next step
    pub fn next_step(&mut self) {
        if self.current_step < self.steps.len() - 1 {
            self.current_step += 1;
            let step_name = &self.steps[self.current_step];
            self.step_progress = ProgressTracker::new(step_name);
            
            // Calculate overall progress
            let overall_progress = (self.current_step as f64 / self.steps.len() as f64 * 100.0) as u64;
            self.step_progress.set_progress(0);
            println!("Step {}/{}: {}", self.current_step + 1, self.steps.len(), step_name);
        }
    }

    /// Set progress for the current step
    pub fn set_step_progress(&self, progress: u64) {
        self.step_progress.set_progress(progress);
    }

    /// Set message for the current step
    pub fn set_step_message(&self, message: &str) {
        self.step_progress.set_message(message);
    }

    /// Finish the current step
    pub fn finish_step(&self, message: &str) {
        self.step_progress.finish(message);
    }

    /// Finish all steps with success
    pub fn finish_all(&self, message: &str) {
        self.step_progress.finish(message);
        println!("✅ All steps completed successfully!");
    }

    /// Finish with error
    pub fn finish_with_error(&self, message: &str) {
        self.step_progress.finish_with_error(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new("Test Operation");
        // Just test that it can be created without panicking
        tracker.set_progress(50);
        tracker.set_message("Testing...");
        tracker.finish("Test completed");
    }

    #[test]
    fn test_multi_step_progress() {
        let steps = vec![
            "Step 1".to_string(),
            "Step 2".to_string(),
            "Step 3".to_string(),
        ];
        
        let mut multi_progress = MultiStepProgress::new(steps);
        assert_eq!(multi_progress.current_step, 0);
        
        multi_progress.next_step();
        assert_eq!(multi_progress.current_step, 1);
        
        multi_progress.set_step_progress(75);
        multi_progress.finish_step("Step completed");
    }

    #[test]
    fn test_file_progress() {
        let tracker = ProgressTracker::file_progress(1000, "File Processing");
        tracker.set_file_progress(500);
        tracker.set_message("Processing file...");
        tracker.finish("File processed successfully");
    }

    #[test]
    fn test_spinner() {
        let tracker = ProgressTracker::spinner("Loading");
        tracker.set_message("Loading data...");
        // Let it spin briefly
        thread::sleep(Duration::from_millis(100));
        tracker.finish("Loading completed");
    }
}

