//! Performance optimization utilities for zkIPFS-Proof
//!
//! This module provides performance monitoring, optimization hints,
//! and resource management functionality.

use crate::{error::Result, types::*};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Performance monitor for tracking system resources and optimization opportunities
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    /// Historical performance data
    history: Vec<PerformanceSnapshot>,
    /// Current optimization settings
    optimizations: OptimizationSettings,
    /// Resource usage thresholds
    thresholds: ResourceThresholds,
}

/// Snapshot of performance metrics at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub disk_io_bytes_per_sec: u64,
    pub network_io_bytes_per_sec: u64,
    pub active_proofs: u32,
    pub queue_length: u32,
}

/// Optimization settings for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    /// Enable parallel processing
    pub parallel_processing: bool,
    /// Maximum number of concurrent proofs
    pub max_concurrent_proofs: u32,
    /// Enable memory pooling
    pub memory_pooling: bool,
    /// Enable disk caching
    pub disk_caching: bool,
    /// Cache size in bytes
    pub cache_size_bytes: u64,
    /// Enable compression
    pub compression_enabled: bool,
    /// Compression level (1-9)
    pub compression_level: u8,
}

/// Resource usage thresholds for optimization decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceThresholds {
    /// Maximum memory usage before optimization (bytes)
    pub max_memory_bytes: u64,
    /// Maximum CPU usage before throttling (percentage)
    pub max_cpu_percent: f64,
    /// Maximum disk I/O before caching (bytes/sec)
    pub max_disk_io_bytes_per_sec: u64,
    /// Maximum queue length before scaling
    pub max_queue_length: u32,
}

/// Performance optimization recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendations {
    /// Recommended changes to optimization settings
    pub settings_changes: HashMap<String, String>,
    /// Estimated performance improvement
    pub estimated_improvement_percent: f64,
    /// Resource allocation recommendations
    pub resource_recommendations: Vec<String>,
    /// Priority level (1-5, 5 being highest)
    pub priority: u8,
}

impl PerformanceMonitor {
    /// Creates a new performance monitor with default settings
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            optimizations: OptimizationSettings::default(),
            thresholds: ResourceThresholds::default(),
        }
    }

    /// Creates a performance monitor with custom settings
    pub fn with_settings(
        optimizations: OptimizationSettings,
        thresholds: ResourceThresholds,
    ) -> Self {
        Self {
            history: Vec::new(),
            optimizations,
            thresholds,
        }
    }

    /// Records a performance snapshot
    pub fn record_snapshot(&mut self) -> Result<()> {
        let snapshot = self.capture_current_snapshot()?;
        self.history.push(snapshot);

        // Keep only last 1000 snapshots to prevent memory bloat
        if self.history.len() > 1000 {
            self.history.remove(0);
        }

        // Check for optimization opportunities
        self.check_optimization_opportunities();

        Ok(())
    }

    /// Captures current system performance metrics
    fn capture_current_snapshot(&self) -> Result<PerformanceSnapshot> {
        Ok(PerformanceSnapshot {
            timestamp: chrono::Utc::now(),
            memory_usage_bytes: self.get_memory_usage(),
            cpu_usage_percent: self.get_cpu_usage(),
            disk_io_bytes_per_sec: self.get_disk_io_rate(),
            network_io_bytes_per_sec: self.get_network_io_rate(),
            active_proofs: self.get_active_proof_count(),
            queue_length: self.get_queue_length(),
        })
    }

    /// Gets current memory usage in bytes
    fn get_memory_usage(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
                for line in contents.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024;
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ps")
                .args(&["-o", "rss=", "-p"])
                .arg(std::process::id().to_string())
                .output()
            {
                if let Ok(rss_str) = String::from_utf8(output.stdout) {
                    if let Ok(rss_kb) = rss_str.trim().parse::<u64>() {
                        return rss_kb * 1024;
                    }
                }
            }
        }

        // Fallback estimate
        256 * 1024 * 1024 // 256MB
    }

    /// Gets current CPU usage percentage
    fn get_cpu_usage(&self) -> f64 {
        // Simplified CPU usage estimation
        // In a real implementation, this would use system APIs
        50.0 // Placeholder
    }

    /// Gets current disk I/O rate in bytes per second
    fn get_disk_io_rate(&self) -> u64 {
        // Simplified disk I/O rate estimation
        1024 * 1024 // 1MB/s placeholder
    }

    /// Gets current network I/O rate in bytes per second
    fn get_network_io_rate(&self) -> u64 {
        // Simplified network I/O rate estimation
        512 * 1024 // 512KB/s placeholder
    }

    /// Gets number of active proof generation processes
    fn get_active_proof_count(&self) -> u32 {
        // This would be tracked by the proof generator
        1 // Placeholder
    }

    /// Gets current queue length
    fn get_queue_length(&self) -> u32 {
        // This would be tracked by the job queue
        0 // Placeholder
    }

    /// Checks for optimization opportunities based on current metrics
    fn check_optimization_opportunities(&self) {
        if let Some(latest) = self.history.last() {
            // Check memory usage
            if latest.memory_usage_bytes > self.thresholds.max_memory_bytes {
                warn!(
                    "Memory usage ({} MB) exceeds threshold ({} MB)",
                    latest.memory_usage_bytes / (1024 * 1024),
                    self.thresholds.max_memory_bytes / (1024 * 1024)
                );
            }

            // Check CPU usage
            if latest.cpu_usage_percent > self.thresholds.max_cpu_percent {
                warn!(
                    "CPU usage ({:.1}%) exceeds threshold ({:.1}%)",
                    latest.cpu_usage_percent,
                    self.thresholds.max_cpu_percent
                );
            }

            // Check queue length
            if latest.queue_length > self.thresholds.max_queue_length {
                warn!(
                    "Queue length ({}) exceeds threshold ({})",
                    latest.queue_length,
                    self.thresholds.max_queue_length
                );
            }
        }
    }

    /// Generates optimization recommendations based on performance history
    pub fn get_optimization_recommendations(&self) -> OptimizationRecommendations {
        let mut recommendations = HashMap::new();
        let mut estimated_improvement = 0.0;
        let mut resource_recommendations = Vec::new();
        let mut priority = 1;

        if self.history.len() < 5 {
            return OptimizationRecommendations {
                settings_changes: recommendations,
                estimated_improvement_percent: 0.0,
                resource_recommendations: vec![
                    "Insufficient performance data for recommendations".to_string()
                ],
                priority: 1,
            };
        }

        // Analyze recent performance trends
        let recent_snapshots = &self.history[self.history.len().saturating_sub(10)..];
        let avg_memory = recent_snapshots.iter()
            .map(|s| s.memory_usage_bytes)
            .sum::<u64>() / recent_snapshots.len() as u64;
        let avg_cpu = recent_snapshots.iter()
            .map(|s| s.cpu_usage_percent)
            .sum::<f64>() / recent_snapshots.len() as f64;

        // Memory optimization recommendations
        if avg_memory > self.thresholds.max_memory_bytes * 80 / 100 {
            recommendations.insert(
                "memory_pooling".to_string(),
                "Enable memory pooling to reduce allocation overhead".to_string(),
            );
            estimated_improvement += 15.0;
            priority = priority.max(3);
        }

        // CPU optimization recommendations
        if avg_cpu > self.thresholds.max_cpu_percent * 80 / 100 {
            if !self.optimizations.parallel_processing {
                recommendations.insert(
                    "parallel_processing".to_string(),
                    "Enable parallel processing to utilize multiple CPU cores".to_string(),
                );
                estimated_improvement += 25.0;
                priority = priority.max(4);
            }
        }

        // Disk I/O optimization recommendations
        let avg_disk_io = recent_snapshots.iter()
            .map(|s| s.disk_io_bytes_per_sec)
            .sum::<u64>() / recent_snapshots.len() as u64;
        
        if avg_disk_io > self.thresholds.max_disk_io_bytes_per_sec * 70 / 100 {
            if !self.optimizations.disk_caching {
                recommendations.insert(
                    "disk_caching".to_string(),
                    "Enable disk caching to reduce I/O operations".to_string(),
                );
                estimated_improvement += 20.0;
                priority = priority.max(3);
            }
        }

        // Resource allocation recommendations
        if avg_memory > 1024 * 1024 * 1024 { // > 1GB
            resource_recommendations.push(
                "Consider increasing available memory for better performance".to_string()
            );
        }

        if self.optimizations.max_concurrent_proofs < 4 && avg_cpu < 60.0 {
            resource_recommendations.push(
                "CPU utilization is low - consider increasing concurrent proof limit".to_string()
            );
        }

        OptimizationRecommendations {
            settings_changes: recommendations,
            estimated_improvement_percent: estimated_improvement.min(100.0),
            resource_recommendations,
            priority,
        }
    }

    /// Applies optimization recommendations
    pub fn apply_optimizations(&mut self, recommendations: &OptimizationRecommendations) {
        for (setting, _description) in &recommendations.settings_changes {
            match setting.as_str() {
                "memory_pooling" => {
                    self.optimizations.memory_pooling = true;
                    info!("Enabled memory pooling optimization");
                }
                "parallel_processing" => {
                    self.optimizations.parallel_processing = true;
                    info!("Enabled parallel processing optimization");
                }
                "disk_caching" => {
                    self.optimizations.disk_caching = true;
                    info!("Enabled disk caching optimization");
                }
                _ => {
                    debug!("Unknown optimization setting: {}", setting);
                }
            }
        }
    }

    /// Gets current optimization settings
    pub fn get_optimizations(&self) -> &OptimizationSettings {
        &self.optimizations
    }

    /// Updates optimization settings
    pub fn update_optimizations(&mut self, optimizations: OptimizationSettings) {
        self.optimizations = optimizations;
    }

    /// Gets performance history
    pub fn get_history(&self) -> &[PerformanceSnapshot] {
        &self.history
    }

    /// Calculates performance statistics over a time period
    pub fn get_performance_stats(&self, duration: Duration) -> Option<PerformanceStats> {
        if self.history.is_empty() {
            return None;
        }

        let cutoff_time = chrono::Utc::now() - chrono::Duration::from_std(duration).ok()?;
        let relevant_snapshots: Vec<_> = self.history.iter()
            .filter(|s| s.timestamp > cutoff_time)
            .collect();

        if relevant_snapshots.is_empty() {
            return None;
        }

        let count = relevant_snapshots.len() as f64;
        
        Some(PerformanceStats {
            avg_memory_usage_bytes: relevant_snapshots.iter()
                .map(|s| s.memory_usage_bytes)
                .sum::<u64>() / relevant_snapshots.len() as u64,
            avg_cpu_usage_percent: relevant_snapshots.iter()
                .map(|s| s.cpu_usage_percent)
                .sum::<f64>() / count,
            max_memory_usage_bytes: relevant_snapshots.iter()
                .map(|s| s.memory_usage_bytes)
                .max()
                .unwrap_or(0),
            max_cpu_usage_percent: relevant_snapshots.iter()
                .map(|s| s.cpu_usage_percent)
                .fold(0.0, |a, s| a.max(s.cpu_usage_percent)),
            total_snapshots: relevant_snapshots.len() as u32,
        })
    }
}

/// Performance statistics over a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub avg_memory_usage_bytes: u64,
    pub avg_cpu_usage_percent: f64,
    pub max_memory_usage_bytes: u64,
    pub max_cpu_usage_percent: f64,
    pub total_snapshots: u32,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            parallel_processing: true,
            max_concurrent_proofs: 2,
            memory_pooling: false,
            disk_caching: true,
            cache_size_bytes: 1024 * 1024 * 1024, // 1GB
            compression_enabled: true,
            compression_level: 6,
        }
    }
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            max_memory_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            max_cpu_percent: 80.0,
            max_disk_io_bytes_per_sec: 100 * 1024 * 1024, // 100MB/s
            max_queue_length: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert_eq!(monitor.history.len(), 0);
        assert!(monitor.optimizations.parallel_processing);
    }

    #[test]
    fn test_optimization_recommendations() {
        let mut monitor = PerformanceMonitor::new();
        
        // Add some mock performance data
        for _ in 0..10 {
            let _ = monitor.record_snapshot();
        }
        
        let recommendations = monitor.get_optimization_recommendations();
        assert!(recommendations.estimated_improvement_percent >= 0.0);
        assert!(recommendations.priority >= 1 && recommendations.priority <= 5);
    }
}

