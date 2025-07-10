//! Comprehensive error handling patterns for zkIPFS-Proof
//! 
//! This module provides standardized error handling patterns, error recovery
//! strategies, and error reporting mechanisms used throughout the application.

use std::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error severity levels for categorizing and handling errors appropriately
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - warnings that don't affect functionality
    Low,
    /// Medium severity - errors that affect some functionality but allow continuation
    Medium,
    /// High severity - critical errors that prevent operation
    High,
    /// Fatal severity - unrecoverable errors that require immediate attention
    Fatal,
}

/// Error categories for better error classification and handling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Input validation errors
    Validation,
    /// Network and connectivity errors
    Network,
    /// File system and I/O errors
    FileSystem,
    /// Cryptographic operation errors
    Cryptography,
    /// IPFS-related errors
    Ipfs,
    /// Smart contract interaction errors
    Blockchain,
    /// Configuration and setup errors
    Configuration,
    /// Internal system errors
    Internal,
    /// External service errors
    External,
}

/// Comprehensive error context with metadata for debugging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Error category
    pub category: ErrorCategory,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context and metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Timestamp when the error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// Suggested recovery actions
    pub recovery_suggestions: Vec<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(
        category: ErrorCategory,
        severity: ErrorSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            severity,
            code: code.into(),
            message: message.into(),
            metadata: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
            stack_trace: None,
            recovery_suggestions: Vec::new(),
        }
    }

    /// Add metadata to the error context
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add recovery suggestion
    pub fn with_recovery(mut self, suggestion: impl Into<String>) -> Self {
        self.recovery_suggestions.push(suggestion.into());
        self
    }

    /// Add stack trace
    pub fn with_stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }

    /// Check if error is recoverable based on severity
    pub fn is_recoverable(&self) -> bool {
        matches!(self.severity, ErrorSeverity::Low | ErrorSeverity::Medium)
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self.category {
            ErrorCategory::Validation => {
                format!("Invalid input: {}", self.message)
            }
            ErrorCategory::Network => {
                format!("Network error: {}. Please check your connection.", self.message)
            }
            ErrorCategory::FileSystem => {
                format!("File error: {}. Please check file permissions.", self.message)
            }
            ErrorCategory::Cryptography => {
                format!("Security error: {}. Please try again.", self.message)
            }
            ErrorCategory::Ipfs => {
                format!("IPFS error: {}. Please check IPFS configuration.", self.message)
            }
            ErrorCategory::Blockchain => {
                format!("Blockchain error: {}. Please check network connection.", self.message)
            }
            ErrorCategory::Configuration => {
                format!("Configuration error: {}. Please check settings.", self.message)
            }
            ErrorCategory::Internal => {
                format!("Internal error: {}. Please report this issue.", self.message)
            }
            ErrorCategory::External => {
                format!("External service error: {}. Please try again later.", self.message)
            }
        }
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} (Code: {})",
            self.severity_string(),
            self.category_string(),
            self.message,
            self.code
        )
    }
}

impl ErrorContext {
    fn severity_string(&self) -> &'static str {
        match self.severity {
            ErrorSeverity::Low => "LOW",
            ErrorSeverity::Medium => "MEDIUM",
            ErrorSeverity::High => "HIGH",
            ErrorSeverity::Fatal => "FATAL",
        }
    }

    fn category_string(&self) -> &'static str {
        match self.category {
            ErrorCategory::Validation => "VALIDATION",
            ErrorCategory::Network => "NETWORK",
            ErrorCategory::FileSystem => "FILESYSTEM",
            ErrorCategory::Cryptography => "CRYPTO",
            ErrorCategory::Ipfs => "IPFS",
            ErrorCategory::Blockchain => "BLOCKCHAIN",
            ErrorCategory::Configuration => "CONFIG",
            ErrorCategory::Internal => "INTERNAL",
            ErrorCategory::External => "EXTERNAL",
        }
    }
}

/// Error recovery strategies
pub trait ErrorRecovery {
    /// Attempt to recover from the error
    fn recover(&self) -> Result<(), ErrorContext>;
    
    /// Check if recovery is possible
    fn can_recover(&self) -> bool;
    
    /// Get recovery suggestions
    fn recovery_suggestions(&self) -> Vec<String>;
}

/// Error reporting trait for different output formats
pub trait ErrorReporter {
    /// Report error in JSON format
    fn report_json(&self, error: &ErrorContext) -> String;
    
    /// Report error in human-readable format
    fn report_human(&self, error: &ErrorContext) -> String;
    
    /// Report error for logging
    fn report_log(&self, error: &ErrorContext) -> String;
}

/// Default error reporter implementation
pub struct DefaultErrorReporter;

impl ErrorReporter for DefaultErrorReporter {
    fn report_json(&self, error: &ErrorContext) -> String {
        serde_json::to_string_pretty(error).unwrap_or_else(|_| {
            format!(r#"{{"error": "Failed to serialize error context", "original": "{}"}}"#, error.message)
        })
    }

    fn report_human(&self, error: &ErrorContext) -> String {
        let mut report = format!("Error: {}\n", error.user_message());
        
        if !error.recovery_suggestions.is_empty() {
            report.push_str("\nSuggested actions:\n");
            for (i, suggestion) in error.recovery_suggestions.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, suggestion));
            }
        }
        
        if !error.metadata.is_empty() {
            report.push_str("\nAdditional information:\n");
            for (key, value) in &error.metadata {
                report.push_str(&format!("  {}: {}\n", key, value));
            }
        }
        
        report
    }

    fn report_log(&self, error: &ErrorContext) -> String {
        format!(
            "{} [{}:{}] {} | Code: {} | Metadata: {:?}",
            error.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            error.category_string(),
            error.severity_string(),
            error.message,
            error.code,
            error.metadata
        )
    }
}

/// Macro for creating error contexts with automatic code location
#[macro_export]
macro_rules! error_context {
    ($category:expr, $severity:expr, $code:expr, $message:expr) => {
        $crate::error_patterns::ErrorContext::new($category, $severity, $code, $message)
            .with_metadata("file", file!())
            .with_metadata("line", line!().to_string())
            .with_metadata("column", column!().to_string())
    };
    
    ($category:expr, $severity:expr, $code:expr, $message:expr, $($key:expr => $value:expr),+) => {
        {
            let mut ctx = $crate::error_patterns::ErrorContext::new($category, $severity, $code, $message)
                .with_metadata("file", file!())
                .with_metadata("line", line!().to_string())
                .with_metadata("column", column!().to_string());
            $(
                ctx = ctx.with_metadata($key, $value);
            )+
            ctx
        }
    };
}

/// Result type with error context
pub type ContextResult<T> = Result<T, ErrorContext>;

/// Trait for converting errors to error contexts
pub trait IntoErrorContext<T> {
    fn into_error_context(self, category: ErrorCategory, code: &str) -> ContextResult<T>;
}

impl<T, E: std::error::Error> IntoErrorContext<T> for Result<T, E> {
    fn into_error_context(self, category: ErrorCategory, code: &str) -> ContextResult<T> {
        self.map_err(|e| {
            let severity = match category {
                ErrorCategory::Internal | ErrorCategory::Cryptography => ErrorSeverity::High,
                ErrorCategory::Network | ErrorCategory::External => ErrorSeverity::Medium,
                _ => ErrorSeverity::Low,
            };
            
            ErrorContext::new(category, severity, code, e.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let error = ErrorContext::new(
            ErrorCategory::Validation,
            ErrorSeverity::Medium,
            "VAL001",
            "Invalid file format"
        );
        
        assert_eq!(error.category, ErrorCategory::Validation);
        assert_eq!(error.severity, ErrorSeverity::Medium);
        assert_eq!(error.code, "VAL001");
        assert_eq!(error.message, "Invalid file format");
    }

    #[test]
    fn test_error_context_with_metadata() {
        let error = ErrorContext::new(
            ErrorCategory::FileSystem,
            ErrorSeverity::High,
            "FS001",
            "File not found"
        )
        .with_metadata("path", "/tmp/test.txt")
        .with_recovery("Check if the file exists and you have read permissions");
        
        assert_eq!(error.metadata.get("path"), Some(&"/tmp/test.txt".to_string()));
        assert_eq!(error.recovery_suggestions.len(), 1);
    }

    #[test]
    fn test_error_reporter() {
        let error = ErrorContext::new(
            ErrorCategory::Network,
            ErrorSeverity::Medium,
            "NET001",
            "Connection timeout"
        );
        
        let reporter = DefaultErrorReporter;
        let json_report = reporter.report_json(&error);
        let human_report = reporter.report_human(&error);
        
        assert!(json_report.contains("Connection timeout"));
        assert!(human_report.contains("Network error"));
    }

    #[test]
    fn test_error_macro() {
        let error = error_context!(
            ErrorCategory::Validation,
            ErrorSeverity::Low,
            "VAL002",
            "Invalid input format",
            "input" => "test.txt",
            "expected" => "JSON"
        );
        
        assert_eq!(error.code, "VAL002");
        assert!(error.metadata.contains_key("input"));
        assert!(error.metadata.contains_key("file"));
    }
}

