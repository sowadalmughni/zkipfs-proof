//! Performance profiling and optimization tools for zkIPFS-Proof
//! 
//! This module provides comprehensive performance profiling, benchmarking,
//! and optimization tools for analyzing and improving system performance.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Performance profile data for a specific operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Operation name
    pub operation: String,
    /// Total execution time
    pub total_duration: Duration,
    /// Time spent in different phases
    pub phase_durations: HashMap<String, Duration>,
    /// Memory allocations during operation
    pub memory_allocations: Vec<MemoryAllocation>,
    /// CPU usage samples
    pub cpu_samples: Vec<CpuSample>,
    /// I/O operations performed
    pub io_operations: Vec<IoOperation>,
    /// Custom metrics
    pub custom_metrics: HashMap<String, f64>,
    /// Timestamp when profiling started
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Profiling metadata
    pub metadata: HashMap<String, String>,
}

/// Memory allocation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    /// Size of allocation in bytes
    pub size: usize,
    /// Timestamp of allocation
    pub timestamp: Duration,
    /// Allocation type (heap, stack, etc.)
    pub allocation_type: String,
    /// Source location if available
    pub source_location: Option<String>,
}

/// CPU usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSample {
    /// CPU usage percentage at sample time
    pub usage_percent: f64,
    /// Timestamp of sample
    pub timestamp: Duration,
    /// Thread ID if applicable
    pub thread_id: Option<u32>,
}

/// I/O operation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoOperation {
    /// Type of I/O operation (read, write, network, etc.)
    pub operation_type: String,
    /// Size of data transferred
    pub size: usize,
    /// Duration of operation
    pub duration: Duration,
    /// Timestamp when operation started
    pub timestamp: Duration,
    /// Target (file path, network endpoint, etc.)
    pub target: String,
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of iterations to run
    pub iterations: usize,
    /// Warmup iterations before measurement
    pub warmup_iterations: usize,
    /// Maximum duration for benchmark
    pub max_duration: Duration,
    /// Sample size for statistical analysis
    pub sample_size: usize,
    /// Enable memory profiling
    pub profile_memory: bool,
    /// Enable CPU profiling
    pub profile_cpu: bool,
    /// Enable I/O profiling
    pub profile_io: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            max_duration: Duration::from_secs(300), // 5 minutes
            sample_size: 1000,
            profile_memory: true,
            profile_cpu: true,
            profile_io: true,
        }
    }
}

/// Benchmark results with statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    /// Operation being benchmarked
    pub operation: String,
    /// Number of successful iterations
    pub successful_iterations: usize,
    /// Number of failed iterations
    pub failed_iterations: usize,
    /// Mean execution time
    pub mean_duration: Duration,
    /// Median execution time
    pub median_duration: Duration,
    /// Standard deviation
    pub std_deviation: Duration,
    /// Minimum execution time
    pub min_duration: Duration,
    /// Maximum execution time
    pub max_duration: Duration,
    /// 95th percentile
    pub p95_duration: Duration,
    /// 99th percentile
    pub p99_duration: Duration,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// CPU usage statistics
    pub cpu_stats: CpuStats,
    /// I/O statistics
    pub io_stats: IoStats,
    /// Performance regression analysis
    pub regression_analysis: Option<RegressionAnalysis>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Peak memory usage
    pub peak_usage: usize,
    /// Average memory usage
    pub average_usage: usize,
    /// Total allocations
    pub total_allocations: usize,
    /// Total deallocations
    pub total_deallocations: usize,
    /// Memory leaks detected
    pub leaks_detected: usize,
}

/// CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    /// Average CPU usage
    pub average_usage: f64,
    /// Peak CPU usage
    pub peak_usage: f64,
    /// CPU time spent in user mode
    pub user_time: Duration,
    /// CPU time spent in system mode
    pub system_time: Duration,
}

/// I/O statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoStats {
    /// Total bytes read
    pub bytes_read: usize,
    /// Total bytes written
    pub bytes_written: usize,
    /// Number of read operations
    pub read_operations: usize,
    /// Number of write operations
    pub write_operations: usize,
    /// Average I/O latency
    pub average_latency: Duration,
}

/// Performance regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    /// Baseline performance data
    pub baseline: BenchmarkSummary,
    /// Current performance data
    pub current: BenchmarkSummary,
    /// Performance change percentage
    pub performance_change: f64,
    /// Regression detected
    pub regression_detected: bool,
    /// Improvement detected
    pub improvement_detected: bool,
    /// Confidence level of analysis
    pub confidence_level: f64,
}

/// Summary of benchmark data for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub mean_duration: Duration,
    pub throughput: f64,
    pub memory_usage: usize,
    pub cpu_usage: f64,
}

/// Performance profiler for tracking operation performance
pub struct PerformanceProfiler {
    operation: String,
    start_time: Instant,
    start_timestamp: chrono::DateTime<chrono::Utc>,
    phase_timers: HashMap<String, Instant>,
    memory_allocations: Vec<MemoryAllocation>,
    cpu_samples: Vec<CpuSample>,
    io_operations: Vec<IoOperation>,
    custom_metrics: HashMap<String, f64>,
    metadata: HashMap<String, String>,
}

impl PerformanceProfiler {
    /// Start profiling an operation
    pub fn start(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
            start_timestamp: chrono::Utc::now(),
            phase_timers: HashMap::new(),
            memory_allocations: Vec::new(),
            cpu_samples: Vec::new(),
            io_operations: Vec::new(),
            custom_metrics: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Start timing a specific phase
    pub fn start_phase(&mut self, phase: &str) {
        self.phase_timers.insert(phase.to_string(), Instant::now());
    }

    /// End timing a specific phase
    pub fn end_phase(&mut self, phase: &str) -> Option<Duration> {
        if let Some(start_time) = self.phase_timers.remove(phase) {
            Some(start_time.elapsed())
        } else {
            None
        }
    }

    /// Record a memory allocation
    pub fn record_allocation(&mut self, size: usize, allocation_type: &str) {
        let allocation = MemoryAllocation {
            size,
            timestamp: self.start_time.elapsed(),
            allocation_type: allocation_type.to_string(),
            source_location: None,
        };
        self.memory_allocations.push(allocation);
    }

    /// Record CPU usage sample
    pub fn record_cpu_sample(&mut self, usage_percent: f64) {
        let sample = CpuSample {
            usage_percent,
            timestamp: self.start_time.elapsed(),
            thread_id: None,
        };
        self.cpu_samples.push(sample);
    }

    /// Record I/O operation
    pub fn record_io_operation(&mut self, operation_type: &str, size: usize, duration: Duration, target: &str) {
        let operation = IoOperation {
            operation_type: operation_type.to_string(),
            size,
            duration,
            timestamp: self.start_time.elapsed(),
            target: target.to_string(),
        };
        self.io_operations.push(operation);
    }

    /// Add custom metric
    pub fn add_metric(&mut self, name: &str, value: f64) {
        self.custom_metrics.insert(name.to_string(), value);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }

    /// Finish profiling and return results
    pub fn finish(self) -> PerformanceProfile {
        let total_duration = self.start_time.elapsed();
        
        // Calculate phase durations for any remaining active phases
        let mut phase_durations = HashMap::new();
        for (phase, start_time) in self.phase_timers {
            phase_durations.insert(phase, start_time.elapsed());
        }

        PerformanceProfile {
            operation: self.operation,
            total_duration,
            phase_durations,
            memory_allocations: self.memory_allocations,
            cpu_samples: self.cpu_samples,
            io_operations: self.io_operations,
            custom_metrics: self.custom_metrics,
            start_time: self.start_timestamp,
            metadata: self.metadata,
        }
    }
}

/// Benchmark runner for performance testing
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    baseline_results: Option<BenchmarkResults>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            baseline_results: None,
        }
    }

    /// Set baseline results for regression analysis
    pub fn set_baseline(&mut self, baseline: BenchmarkResults) {
        self.baseline_results = Some(baseline);
    }

    /// Run benchmark for a given operation
    pub fn run_benchmark<F, T>(&self, operation_name: &str, mut operation: F) -> BenchmarkResults
    where
        F: FnMut() -> Result<T, Box<dyn std::error::Error>>,
    {
        let mut durations = Vec::new();
        let mut successful_iterations = 0;
        let mut failed_iterations = 0;
        let mut memory_stats = MemoryStats {
            peak_usage: 0,
            average_usage: 0,
            total_allocations: 0,
            total_deallocations: 0,
            leaks_detected: 0,
        };
        let mut cpu_stats = CpuStats {
            average_usage: 0.0,
            peak_usage: 0.0,
            user_time: Duration::default(),
            system_time: Duration::default(),
        };
        let mut io_stats = IoStats {
            bytes_read: 0,
            bytes_written: 0,
            read_operations: 0,
            write_operations: 0,
            average_latency: Duration::default(),
        };

        // Warmup iterations
        for _ in 0..self.config.warmup_iterations {
            let _ = operation();
        }

        let benchmark_start = Instant::now();

        // Main benchmark iterations
        for i in 0..self.config.iterations {
            if benchmark_start.elapsed() > self.config.max_duration {
                break;
            }

            let start = Instant::now();
            match operation() {
                Ok(_) => {
                    let duration = start.elapsed();
                    durations.push(duration);
                    successful_iterations += 1;
                }
                Err(_) => {
                    failed_iterations += 1;
                }
            }

            // Sample system metrics periodically
            if i % 10 == 0 {
                // Update memory stats (placeholder implementation)
                memory_stats.peak_usage = memory_stats.peak_usage.max(get_current_memory_usage());
                
                // Update CPU stats (placeholder implementation)
                let current_cpu = get_current_cpu_usage();
                cpu_stats.peak_usage = cpu_stats.peak_usage.max(current_cpu);
            }
        }

        // Calculate statistics
        durations.sort();
        let mean_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
        let median_duration = durations[durations.len() / 2];
        let min_duration = durations.first().copied().unwrap_or_default();
        let max_duration = durations.last().copied().unwrap_or_default();
        let p95_duration = durations[(durations.len() as f64 * 0.95) as usize];
        let p99_duration = durations[(durations.len() as f64 * 0.99) as usize];

        // Calculate standard deviation
        let variance: f64 = durations
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean_duration.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>() / durations.len() as f64;
        let std_deviation = Duration::from_nanos(variance.sqrt() as u64);

        // Calculate throughput
        let total_time = benchmark_start.elapsed();
        let throughput = successful_iterations as f64 / total_time.as_secs_f64();

        // Finalize stats
        memory_stats.average_usage = memory_stats.peak_usage / 2; // Simplified
        cpu_stats.average_usage = cpu_stats.peak_usage / 2; // Simplified
        io_stats.average_latency = mean_duration / 10; // Simplified

        let mut results = BenchmarkResults {
            operation: operation_name.to_string(),
            successful_iterations,
            failed_iterations,
            mean_duration,
            median_duration,
            std_deviation,
            min_duration,
            max_duration,
            p95_duration,
            p99_duration,
            throughput,
            memory_stats,
            cpu_stats,
            io_stats,
            regression_analysis: None,
        };

        // Perform regression analysis if baseline is available
        if let Some(baseline) = &self.baseline_results {
            results.regression_analysis = Some(self.analyze_regression(baseline, &results));
        }

        results
    }

    fn analyze_regression(&self, baseline: &BenchmarkResults, current: &BenchmarkResults) -> RegressionAnalysis {
        let baseline_summary = BenchmarkSummary {
            mean_duration: baseline.mean_duration,
            throughput: baseline.throughput,
            memory_usage: baseline.memory_stats.average_usage,
            cpu_usage: baseline.cpu_stats.average_usage,
        };

        let current_summary = BenchmarkSummary {
            mean_duration: current.mean_duration,
            throughput: current.throughput,
            memory_usage: current.memory_stats.average_usage,
            cpu_usage: current.cpu_stats.average_usage,
        };

        // Calculate performance change (positive = improvement, negative = regression)
        let performance_change = (baseline.throughput - current.throughput) / baseline.throughput * 100.0;
        
        let regression_detected = performance_change < -5.0; // 5% threshold
        let improvement_detected = performance_change > 5.0;
        let confidence_level = 0.95; // Simplified confidence calculation

        RegressionAnalysis {
            baseline: baseline_summary,
            current: current_summary,
            performance_change,
            regression_detected,
            improvement_detected,
            confidence_level,
        }
    }
}

/// Optimization recommendations based on profiling data
pub struct OptimizationRecommendations {
    pub recommendations: Vec<OptimizationRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub priority: OptimizationPriority,
    pub description: String,
    pub potential_improvement: String,
    pub implementation_effort: ImplementationEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

impl OptimizationRecommendations {
    /// Generate recommendations based on performance profile
    pub fn generate(profile: &PerformanceProfile) -> Self {
        let mut recommendations = Vec::new();

        // Analyze memory usage
        if profile.memory_allocations.len() > 1000 {
            recommendations.push(OptimizationRecommendation {
                category: "Memory".to_string(),
                priority: OptimizationPriority::High,
                description: "High number of memory allocations detected".to_string(),
                potential_improvement: "Reduce allocations by using object pooling or pre-allocation".to_string(),
                implementation_effort: ImplementationEffort::Medium,
            });
        }

        // Analyze CPU usage
        let avg_cpu = profile.cpu_samples.iter()
            .map(|s| s.usage_percent)
            .sum::<f64>() / profile.cpu_samples.len() as f64;
        
        if avg_cpu > 80.0 {
            recommendations.push(OptimizationRecommendation {
                category: "CPU".to_string(),
                priority: OptimizationPriority::High,
                description: "High CPU usage detected".to_string(),
                potential_improvement: "Consider algorithm optimization or parallel processing".to_string(),
                implementation_effort: ImplementationEffort::High,
            });
        }

        // Analyze I/O operations
        let total_io_time: Duration = profile.io_operations.iter()
            .map(|op| op.duration)
            .sum();
        
        if total_io_time > profile.total_duration / 2 {
            recommendations.push(OptimizationRecommendation {
                category: "I/O".to_string(),
                priority: OptimizationPriority::Medium,
                description: "I/O operations consume significant time".to_string(),
                potential_improvement: "Consider caching, batching, or asynchronous I/O".to_string(),
                implementation_effort: ImplementationEffort::Medium,
            });
        }

        Self { recommendations }
    }
}

// Helper functions for system metrics (placeholder implementations)
fn get_current_memory_usage() -> usize {
    1024 * 1024 * 50 // 50MB placeholder
}

fn get_current_cpu_usage() -> f64 {
    25.0 // 25% placeholder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_profiler() {
        let mut profiler = PerformanceProfiler::start("test_operation");
        
        profiler.start_phase("phase1");
        std::thread::sleep(Duration::from_millis(10));
        profiler.end_phase("phase1");
        
        profiler.record_allocation(1024, "heap");
        profiler.record_cpu_sample(50.0);
        profiler.add_metric("custom_metric", 42.0);
        
        let profile = profiler.finish();
        
        assert_eq!(profile.operation, "test_operation");
        assert!(profile.total_duration >= Duration::from_millis(10));
        assert_eq!(profile.memory_allocations.len(), 1);
        assert_eq!(profile.cpu_samples.len(), 1);
        assert_eq!(profile.custom_metrics.get("custom_metric"), Some(&42.0));
    }

    #[test]
    fn test_benchmark_runner() {
        let config = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 2,
            ..Default::default()
        };
        
        let runner = BenchmarkRunner::new(config);
        
        let results = runner.run_benchmark("test_op", || {
            std::thread::sleep(Duration::from_millis(1));
            Ok(())
        });
        
        assert_eq!(results.operation, "test_op");
        assert_eq!(results.successful_iterations, 10);
        assert_eq!(results.failed_iterations, 0);
        assert!(results.mean_duration >= Duration::from_millis(1));
    }

    #[test]
    fn test_optimization_recommendations() {
        let profile = PerformanceProfile {
            operation: "test".to_string(),
            total_duration: Duration::from_secs(1),
            phase_durations: HashMap::new(),
            memory_allocations: vec![MemoryAllocation {
                size: 1024,
                timestamp: Duration::from_millis(100),
                allocation_type: "heap".to_string(),
                source_location: None,
            }; 2000], // High number of allocations
            cpu_samples: vec![CpuSample {
                usage_percent: 90.0, // High CPU usage
                timestamp: Duration::from_millis(100),
                thread_id: None,
            }],
            io_operations: Vec::new(),
            custom_metrics: HashMap::new(),
            start_time: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        let recommendations = OptimizationRecommendations::generate(&profile);
        assert!(!recommendations.recommendations.is_empty());
        
        // Should have memory and CPU recommendations
        let categories: Vec<_> = recommendations.recommendations
            .iter()
            .map(|r| &r.category)
            .collect();
        assert!(categories.contains(&&"Memory".to_string()));
        assert!(categories.contains(&&"CPU".to_string()));
    }
}

