//! Structured logging and monitoring for zkIPFS-Proof
//! 
//! This module provides comprehensive logging, metrics collection, and monitoring
//! capabilities for performance tracking and debugging.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug, trace};

/// Log levels for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Structured log entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Component that generated the log
    pub component: String,
    /// Operation being performed
    pub operation: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Correlation ID for tracing requests
    pub correlation_id: Option<String>,
    /// User ID if applicable
    pub user_id: Option<String>,
    /// Session ID if applicable
    pub session_id: Option<String>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, component: &str, message: &str) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level,
            message: message.to_string(),
            component: component.to_string(),
            operation: None,
            metadata: HashMap::new(),
            correlation_id: None,
            user_id: None,
            session_id: None,
        }
    }

    /// Add metadata to the log entry
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Set operation
    pub fn with_operation(mut self, operation: &str) -> Self {
        self.operation = Some(operation.to_string());
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: &str) -> Self {
        self.correlation_id = Some(id.to_string());
        self
    }
}

/// Performance metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Operation name
    pub operation: String,
    /// Duration of the operation
    pub duration: Duration,
    /// Memory usage during operation
    pub memory_usage: Option<u64>,
    /// CPU usage percentage
    pub cpu_usage: Option<f64>,
    /// Success/failure status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Additional metrics
    pub custom_metrics: HashMap<String, f64>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// System health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Disk usage percentage
    pub disk_usage: f64,
    /// Network connectivity status
    pub network_status: bool,
    /// IPFS node status
    pub ipfs_status: bool,
    /// Active connections count
    pub active_connections: u32,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable performance monitoring
    pub enable_performance: bool,
    /// Enable health monitoring
    pub enable_health: bool,
    /// Log level threshold
    pub log_level: LogLevel,
    /// Metrics collection interval
    pub metrics_interval: Duration,
    /// Log retention period
    pub log_retention: Duration,
    /// Export metrics to external systems
    pub export_metrics: bool,
    /// External metrics endpoint
    pub metrics_endpoint: Option<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_performance: true,
            enable_health: true,
            log_level: LogLevel::Info,
            metrics_interval: Duration::from_secs(60),
            log_retention: Duration::from_secs(7 * 24 * 3600), // 7 days
            export_metrics: false,
            metrics_endpoint: None,
        }
    }
}

/// Performance timer for measuring operation duration
pub struct PerformanceTimer {
    operation: String,
    start_time: Instant,
    metadata: HashMap<String, String>,
}

impl PerformanceTimer {
    /// Start a new performance timer
    pub fn start(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the timer
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Stop the timer and return metrics
    pub fn stop(self) -> PerformanceMetrics {
        let duration = self.start_time.elapsed();
        
        PerformanceMetrics {
            operation: self.operation,
            duration,
            memory_usage: get_memory_usage(),
            cpu_usage: get_cpu_usage(),
            success: true,
            error_message: None,
            custom_metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Stop the timer with error
    pub fn stop_with_error(self, error: &str) -> PerformanceMetrics {
        let duration = self.start_time.elapsed();
        
        PerformanceMetrics {
            operation: self.operation,
            duration,
            memory_usage: get_memory_usage(),
            cpu_usage: get_cpu_usage(),
            success: false,
            error_message: Some(error.to_string()),
            custom_metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Monitoring system for collecting and managing metrics
pub struct MonitoringSystem {
    config: MonitoringConfig,
    performance_metrics: Arc<Mutex<Vec<PerformanceMetrics>>>,
    health_metrics: Arc<Mutex<Vec<HealthMetrics>>>,
    log_entries: Arc<Mutex<Vec<LogEntry>>>,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            performance_metrics: Arc::new(Mutex::new(Vec::new())),
            health_metrics: Arc::new(Mutex::new(Vec::new())),
            log_entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Log a structured entry
    pub fn log(&self, entry: LogEntry) {
        // Check if log level meets threshold
        if !self.should_log(&entry.level) {
            return;
        }

        // Add to internal storage
        if let Ok(mut logs) = self.log_entries.lock() {
            logs.push(entry.clone());
        }

        // Output to tracing system
        match entry.level {
            LogLevel::Trace => trace!("{}", self.format_log_entry(&entry)),
            LogLevel::Debug => debug!("{}", self.format_log_entry(&entry)),
            LogLevel::Info => info!("{}", self.format_log_entry(&entry)),
            LogLevel::Warn => warn!("{}", self.format_log_entry(&entry)),
            LogLevel::Error => error!("{}", self.format_log_entry(&entry)),
        }
    }

    /// Record performance metrics
    pub fn record_performance(&self, metrics: PerformanceMetrics) {
        if !self.config.enable_performance {
            return;
        }

        if let Ok(mut perf_metrics) = self.performance_metrics.lock() {
            perf_metrics.push(metrics);
        }
    }

    /// Record health metrics
    pub fn record_health(&self, metrics: HealthMetrics) {
        if !self.config.enable_health {
            return;
        }

        if let Ok(mut health_metrics) = self.health_metrics.lock() {
            health_metrics.push(metrics);
        }
    }

    /// Get current system health
    pub fn get_current_health(&self) -> HealthMetrics {
        HealthMetrics {
            cpu_usage: get_cpu_usage().unwrap_or(0.0),
            memory_usage: get_memory_usage().unwrap_or(0),
            available_memory: get_available_memory().unwrap_or(0),
            disk_usage: get_disk_usage().unwrap_or(0.0),
            network_status: check_network_status(),
            ipfs_status: check_ipfs_status(),
            active_connections: get_active_connections(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get performance summary
    pub fn get_performance_summary(&self, operation: &str) -> Option<PerformanceSummary> {
        let metrics = self.performance_metrics.lock().ok()?;
        let operation_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.operation == operation)
            .collect();

        if operation_metrics.is_empty() {
            return None;
        }

        let total_count = operation_metrics.len();
        let success_count = operation_metrics.iter().filter(|m| m.success).count();
        let durations: Vec<_> = operation_metrics.iter().map(|m| m.duration).collect();
        
        let avg_duration = durations.iter().sum::<Duration>() / total_count as u32;
        let min_duration = durations.iter().min().copied().unwrap_or_default();
        let max_duration = durations.iter().max().copied().unwrap_or_default();

        Some(PerformanceSummary {
            operation: operation.to_string(),
            total_count,
            success_count,
            success_rate: success_count as f64 / total_count as f64,
            avg_duration,
            min_duration,
            max_duration,
        })
    }

    /// Clean up old metrics and logs
    pub fn cleanup_old_data(&self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(self.config.log_retention).unwrap_or_default();

        // Clean up logs
        if let Ok(mut logs) = self.log_entries.lock() {
            logs.retain(|entry| entry.timestamp > cutoff);
        }

        // Clean up performance metrics
        if let Ok(mut metrics) = self.performance_metrics.lock() {
            metrics.retain(|metric| metric.timestamp > cutoff);
        }

        // Clean up health metrics
        if let Ok(mut metrics) = self.health_metrics.lock() {
            metrics.retain(|metric| metric.timestamp > cutoff);
        }
    }

    fn should_log(&self, level: &LogLevel) -> bool {
        match (&self.config.log_level, level) {
            (LogLevel::Trace, _) => true,
            (LogLevel::Debug, LogLevel::Trace) => false,
            (LogLevel::Debug, _) => true,
            (LogLevel::Info, LogLevel::Trace | LogLevel::Debug) => false,
            (LogLevel::Info, _) => true,
            (LogLevel::Warn, LogLevel::Trace | LogLevel::Debug | LogLevel::Info) => false,
            (LogLevel::Warn, _) => true,
            (LogLevel::Error, LogLevel::Error) => true,
            (LogLevel::Error, _) => false,
        }
    }

    fn format_log_entry(&self, entry: &LogEntry) -> String {
        let mut formatted = format!("[{}] {}", entry.component, entry.message);
        
        if let Some(operation) = &entry.operation {
            formatted.push_str(&format!(" | Operation: {}", operation));
        }
        
        if let Some(correlation_id) = &entry.correlation_id {
            formatted.push_str(&format!(" | CorrelationID: {}", correlation_id));
        }
        
        if !entry.metadata.is_empty() {
            formatted.push_str(&format!(" | Metadata: {:?}", entry.metadata));
        }
        
        formatted
    }
}

/// Performance summary for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub operation: String,
    pub total_count: usize,
    pub success_count: usize,
    pub success_rate: f64,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
}

// System metrics collection functions
fn get_memory_usage() -> Option<u64> {
    // Implementation would use system APIs to get actual memory usage
    // For now, return a placeholder
    Some(1024 * 1024 * 100) // 100MB placeholder
}

fn get_cpu_usage() -> Option<f64> {
    // Implementation would use system APIs to get actual CPU usage
    // For now, return a placeholder
    Some(15.5) // 15.5% placeholder
}

fn get_available_memory() -> Option<u64> {
    // Implementation would use system APIs to get available memory
    Some(1024 * 1024 * 1024 * 4) // 4GB placeholder
}

fn get_disk_usage() -> Option<f64> {
    // Implementation would use system APIs to get disk usage
    Some(45.2) // 45.2% placeholder
}

fn check_network_status() -> bool {
    // Implementation would check actual network connectivity
    true // Placeholder
}

fn check_ipfs_status() -> bool {
    // Implementation would check IPFS node status
    true // Placeholder
}

fn get_active_connections() -> u32 {
    // Implementation would get actual connection count
    5 // Placeholder
}

/// Macro for easy performance timing
#[macro_export]
macro_rules! time_operation {
    ($operation:expr, $code:block) => {
        {
            let timer = $crate::monitoring::PerformanceTimer::start($operation);
            let result = $code;
            let metrics = timer.stop();
            // Record metrics if monitoring system is available
            result
        }
    };
}

/// Macro for structured logging
#[macro_export]
macro_rules! log_structured {
    ($level:expr, $component:expr, $message:expr) => {
        {
            let entry = $crate::monitoring::LogEntry::new($level, $component, $message);
            // Log entry would be sent to monitoring system
            entry
        }
    };
    
    ($level:expr, $component:expr, $message:expr, $($key:expr => $value:expr),+) => {
        {
            let mut entry = $crate::monitoring::LogEntry::new($level, $component, $message);
            $(
                entry = entry.with_metadata($key, $value);
            )+
            entry
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "test", "Test message")
            .with_metadata("key", "value")
            .with_operation("test_op");
        
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.component, "test");
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.operation, Some("test_op".to_string()));
        assert_eq!(entry.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::start("test_operation")
            .with_metadata("test", "value");
        
        std::thread::sleep(Duration::from_millis(10));
        let metrics = timer.stop();
        
        assert_eq!(metrics.operation, "test_operation");
        assert!(metrics.duration >= Duration::from_millis(10));
        assert!(metrics.success);
    }

    #[test]
    fn test_monitoring_system() {
        let config = MonitoringConfig::default();
        let monitoring = MonitoringSystem::new(config);
        
        let entry = LogEntry::new(LogLevel::Info, "test", "Test log");
        monitoring.log(entry);
        
        let metrics = PerformanceMetrics {
            operation: "test".to_string(),
            duration: Duration::from_millis(100),
            memory_usage: Some(1024),
            cpu_usage: Some(10.0),
            success: true,
            error_message: None,
            custom_metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };
        monitoring.record_performance(metrics);
        
        let summary = monitoring.get_performance_summary("test");
        assert!(summary.is_some());
    }
}

