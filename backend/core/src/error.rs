//! Error types and handling for zkIPFS-Proof operations
//!
//! This module provides comprehensive error handling for all zkIPFS-Proof operations,
//! including proof generation, verification, IPFS operations, and cryptographic operations.

use std::fmt;
use thiserror::Error;

/// Result type alias for zkIPFS-Proof operations
pub type Result<T> = std::result::Result<T, ProofError>;

/// Comprehensive error type for zkIPFS-Proof operations
#[derive(Error, Debug)]
pub enum ProofError {
    /// Errors related to file I/O operations
    #[error("File operation failed: {message}")]
    FileError {
        message: String,
        #[source]
        source: Option<std::io::Error>,
    },

    /// Errors related to IPFS operations
    #[error("IPFS operation failed: {operation} - {message}")]
    IpfsError {
        operation: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Errors related to zero-knowledge proof operations
    #[error("ZK proof operation failed: {operation} - {message}")]
    ZkProofError {
        operation: String,
        message: String,
        #[source]
        source: Option<risc0_zkvm::ExecutorError>,
    },

    /// Errors related to cryptographic operations
    #[error("Cryptographic operation failed: {operation} - {message}")]
    CryptographicError {
        operation: String,
        message: String,
    },

    /// Errors related to content selection and validation
    #[error("Content selection error: {message}")]
    ContentSelectionError {
        message: String,
    },

    /// Errors related to proof verification
    #[error("Proof verification failed: {reason}")]
    VerificationError {
        reason: String,
    },

    /// Errors related to serialization and deserialization
    #[error("Serialization error: {message}")]
    SerializationError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Errors related to configuration and setup
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
    },

    /// Errors related to network operations
    #[error("Network operation failed: {operation} - {message}")]
    NetworkError {
        operation: String,
        message: String,
        #[source]
        source: Option<reqwest::Error>,
    },

    /// Errors related to resource limits and constraints
    #[error("Resource limit exceeded: {resource} - {message}")]
    ResourceLimitError {
        resource: String,
        message: String,
    },

    /// Errors related to invalid input data
    #[error("Invalid input: {field} - {message}")]
    InvalidInputError {
        field: String,
        message: String,
    },

    /// Errors related to timeout operations
    #[error("Operation timed out: {operation} after {duration_ms}ms")]
    TimeoutError {
        operation: String,
        duration_ms: u64,
    },

    /// Generic internal errors
    #[error("Internal error: {message}")]
    InternalError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl ProofError {
    /// Creates a new file error
    pub fn file_error(message: impl Into<String>, source: Option<std::io::Error>) -> Self {
        Self::FileError {
            message: message.into(),
            source,
        }
    }

    /// Creates a new IPFS error
    pub fn ipfs_error(
        operation: impl Into<String>,
        message: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::IpfsError {
            operation: operation.into(),
            message: message.into(),
            source,
        }
    }

    /// Creates a new ZK proof error
    pub fn zk_proof_error(
        operation: impl Into<String>,
        message: impl Into<String>,
        source: Option<risc0_zkvm::ExecutorError>,
    ) -> Self {
        Self::ZkProofError {
            operation: operation.into(),
            message: message.into(),
            source,
        }
    }

    /// Creates a new cryptographic error
    pub fn cryptographic_error(
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::CryptographicError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Creates a new content selection error
    pub fn content_selection_error(message: impl Into<String>) -> Self {
        Self::ContentSelectionError {
            message: message.into(),
        }
    }

    /// Creates a new verification error
    pub fn verification_error(reason: impl Into<String>) -> Self {
        Self::VerificationError {
            reason: reason.into(),
        }
    }

    /// Creates a new serialization error
    pub fn serialization_error(
        message: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::SerializationError {
            message: message.into(),
            source,
        }
    }

    /// Creates a new configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// Creates a new network error
    pub fn network_error(
        operation: impl Into<String>,
        message: impl Into<String>,
        source: Option<reqwest::Error>,
    ) -> Self {
        Self::NetworkError {
            operation: operation.into(),
            message: message.into(),
            source,
        }
    }

    /// Creates a new resource limit error
    pub fn resource_limit_error(
        resource: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ResourceLimitError {
            resource: resource.into(),
            message: message.into(),
        }
    }

    /// Creates a new invalid input error
    pub fn invalid_input_error(
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidInputError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Creates a new timeout error
    pub fn timeout_error(operation: impl Into<String>, duration_ms: u64) -> Self {
        Self::TimeoutError {
            operation: operation.into(),
            duration_ms,
        }
    }

    /// Creates a new internal error
    pub fn internal_error(
        message: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::InternalError {
            message: message.into(),
            source,
        }
    }

    /// Returns true if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ProofError::NetworkError { .. } | 
            ProofError::TimeoutError { .. } |
            ProofError::IpfsError { .. }
        )
    }

    /// Returns true if this error is a user input error
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            ProofError::InvalidInputError { .. } |
            ProofError::ContentSelectionError { .. } |
            ProofError::ConfigurationError { .. }
        )
    }

    /// Returns true if this error is a system/internal error
    pub fn is_system_error(&self) -> bool {
        matches!(
            self,
            ProofError::InternalError { .. } |
            ProofError::CryptographicError { .. } |
            ProofError::ResourceLimitError { .. }
        )
    }
}

// Implement conversions from common error types
impl From<std::io::Error> for ProofError {
    fn from(err: std::io::Error) -> Self {
        ProofError::file_error("I/O operation failed", Some(err))
    }
}

impl From<risc0_zkvm::ExecutorError> for ProofError {
    fn from(err: risc0_zkvm::ExecutorError) -> Self {
        ProofError::zk_proof_error("ZK execution failed", err.to_string(), Some(err))
    }
}

impl From<serde_json::Error> for ProofError {
    fn from(err: serde_json::Error) -> Self {
        ProofError::serialization_error(
            "JSON serialization failed",
            Some(Box::new(err)),
        )
    }
}

impl From<bincode::Error> for ProofError {
    fn from(err: bincode::Error) -> Self {
        ProofError::serialization_error(
            "Binary serialization failed",
            Some(Box::new(err)),
        )
    }
}

impl From<reqwest::Error> for ProofError {
    fn from(err: reqwest::Error) -> Self {
        ProofError::network_error(
            "HTTP request failed",
            err.to_string(),
            Some(err),
        )
    }
}

impl From<hex::FromHexError> for ProofError {
    fn from(err: hex::FromHexError) -> Self {
        ProofError::invalid_input_error(
            "hex_string",
            format!("Invalid hex string: {}", err),
        )
    }
}

impl From<tokio::time::error::Elapsed> for ProofError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        ProofError::timeout_error("async operation", 0) // Duration not available from Elapsed
    }
}

/// Helper trait for converting Results with context
pub trait ResultExt<T> {
    /// Adds context to an error
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Adds context to an error with a static string
    fn context(self, msg: &'static str) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|err| {
            ProofError::internal_error(f(), Some(Box::new(err)))
        })
    }

    fn context(self, msg: &'static str) -> Result<T> {
        self.with_context(|| msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ProofError::file_error("Test error", None);
        assert!(matches!(err, ProofError::FileError { .. }));
        assert_eq!(err.to_string(), "File operation failed: Test error");
    }

    #[test]
    fn test_error_classification() {
        let user_err = ProofError::invalid_input_error("field", "message");
        assert!(user_err.is_user_error());
        assert!(!user_err.is_system_error());
        assert!(!user_err.is_retryable());

        let network_err = ProofError::network_error("GET", "timeout", None);
        assert!(network_err.is_retryable());
        assert!(!network_err.is_user_error());
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let proof_err: ProofError = io_err.into();
        assert!(matches!(proof_err, ProofError::FileError { .. }));
    }

    #[test]
    fn test_result_ext() {
        let result: std::result::Result<(), std::io::Error> = 
            Err(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        
        let proof_result = result.context("Operation failed");
        assert!(proof_result.is_err());
        assert!(matches!(proof_result.unwrap_err(), ProofError::InternalError { .. }));
    }
}

