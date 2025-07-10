//! Extended proof types for various verification scenarios

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Different types of proofs supported by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofType {
    /// Basic content existence proof
    ContentExistence,
    /// Proof of content pattern with position
    ContentPattern,
    /// Proof of file integrity without revealing content
    FileIntegrity,
    /// Proof of data range (e.g., numerical values within bounds)
    DataRange,
    /// Proof of timestamp validity
    TimestampProof,
    /// Proof of file format compliance
    FormatCompliance,
    /// Proof of content similarity (fuzzy matching)
    ContentSimilarity,
    /// Proof of data aggregation (sum, count, etc.)
    DataAggregation,
    /// Proof of access permissions
    AccessControl,
    /// Composite proof combining multiple types
    Composite(Vec<ProofType>),
}

/// Configuration for different proof types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofTypeConfig {
    pub proof_type: ProofType,
    pub parameters: HashMap<String, ProofParameter>,
    pub security_level: u16,
    pub optimization_level: OptimizationLevel,
}

/// Parameters for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofParameter {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Bytes(Vec<u8>),
    Range { min: f64, max: f64 },
    Pattern { regex: String, case_sensitive: bool },
    Timestamp { before: Option<i64>, after: Option<i64> },
}

/// Optimization levels for proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// Fastest generation, larger proof size
    Speed,
    /// Balanced generation time and proof size
    Balanced,
    /// Smallest proof size, slower generation
    Size,
    /// Maximum security, slowest generation
    Security,
}

/// Content existence proof - proves a specific pattern exists in the file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentExistenceProof {
    pub pattern: String,
    pub found: bool,
    pub position: Option<usize>,
    pub context_hash: [u8; 32],
    pub proof_data: Vec<u8>,
}

/// Data range proof - proves numerical data falls within specified bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRangeProof {
    pub field_name: String,
    pub min_value: f64,
    pub max_value: f64,
    pub actual_count: u32,
    pub proof_data: Vec<u8>,
}

/// Timestamp proof - proves file was created/modified within time bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampProof {
    pub timestamp_type: TimestampType,
    pub claimed_time: i64,
    pub time_bounds: (Option<i64>, Option<i64>),
    pub valid: bool,
    pub proof_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimestampType {
    Creation,
    Modification,
    Access,
    Custom(String),
}

/// Format compliance proof - proves file adheres to specific format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatComplianceProof {
    pub format_type: String,
    pub version: Option<String>,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub proof_data: Vec<u8>,
}

/// Content similarity proof - proves content similarity without revealing exact content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSimilarityProof {
    pub reference_hash: [u8; 32],
    pub similarity_threshold: f64,
    pub actual_similarity: f64,
    pub meets_threshold: bool,
    pub proof_data: Vec<u8>,
}

/// Data aggregation proof - proves statistical properties of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAggregationProof {
    pub aggregation_type: AggregationType,
    pub field_name: String,
    pub result: f64,
    pub count: u32,
    pub proof_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Count,
    Min,
    Max,
    Median,
    StandardDeviation,
}

/// Access control proof - proves user has specific permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlProof {
    pub user_id: String,
    pub resource_id: String,
    pub permission: String,
    pub granted: bool,
    pub expiry: Option<i64>,
    pub proof_data: Vec<u8>,
}

/// Composite proof combining multiple proof types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeProof {
    pub sub_proofs: Vec<ProofResult>,
    pub combination_logic: CombinationLogic,
    pub overall_result: bool,
    pub proof_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombinationLogic {
    And,  // All sub-proofs must be valid
    Or,   // At least one sub-proof must be valid
    Threshold(u32), // At least N sub-proofs must be valid
    Custom(String), // Custom logic expression
}

/// Unified proof result that can contain any proof type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofResult {
    ContentExistence(ContentExistenceProof),
    DataRange(DataRangeProof),
    Timestamp(TimestampProof),
    FormatCompliance(FormatComplianceProof),
    ContentSimilarity(ContentSimilarityProof),
    DataAggregation(DataAggregationProof),
    AccessControl(AccessControlProof),
    Composite(CompositeProof),
}

impl ProofResult {
    /// Check if the proof is valid
    pub fn is_valid(&self) -> bool {
        match self {
            ProofResult::ContentExistence(p) => p.found,
            ProofResult::DataRange(p) => p.actual_count > 0,
            ProofResult::Timestamp(p) => p.valid,
            ProofResult::FormatCompliance(p) => p.compliant,
            ProofResult::ContentSimilarity(p) => p.meets_threshold,
            ProofResult::DataAggregation(_) => true, // Always valid if generated
            ProofResult::AccessControl(p) => p.granted,
            ProofResult::Composite(p) => p.overall_result,
        }
    }

    /// Get the proof data for verification
    pub fn proof_data(&self) -> &[u8] {
        match self {
            ProofResult::ContentExistence(p) => &p.proof_data,
            ProofResult::DataRange(p) => &p.proof_data,
            ProofResult::Timestamp(p) => &p.proof_data,
            ProofResult::FormatCompliance(p) => &p.proof_data,
            ProofResult::ContentSimilarity(p) => &p.proof_data,
            ProofResult::DataAggregation(p) => &p.proof_data,
            ProofResult::AccessControl(p) => &p.proof_data,
            ProofResult::Composite(p) => &p.proof_data,
        }
    }

    /// Get a human-readable description of the proof
    pub fn description(&self) -> String {
        match self {
            ProofResult::ContentExistence(p) => {
                format!("Content '{}' {} in file", p.pattern, 
                    if p.found { "found" } else { "not found" })
            },
            ProofResult::DataRange(p) => {
                format!("Field '{}' has {} values in range [{}, {}]", 
                    p.field_name, p.actual_count, p.min_value, p.max_value)
            },
            ProofResult::Timestamp(p) => {
                format!("Timestamp proof for {:?}: {}", 
                    p.timestamp_type, if p.valid { "valid" } else { "invalid" })
            },
            ProofResult::FormatCompliance(p) => {
                format!("Format compliance for {}: {}", 
                    p.format_type, if p.compliant { "compliant" } else { "non-compliant" })
            },
            ProofResult::ContentSimilarity(p) => {
                format!("Content similarity: {:.2}% (threshold: {:.2}%)", 
                    p.actual_similarity * 100.0, p.similarity_threshold * 100.0)
            },
            ProofResult::DataAggregation(p) => {
                format!("{:?} of '{}': {} (count: {})", 
                    p.aggregation_type, p.field_name, p.result, p.count)
            },
            ProofResult::AccessControl(p) => {
                format!("User '{}' {} access to '{}' with permission '{}'", 
                    p.user_id, if p.granted { "has" } else { "lacks" }, 
                    p.resource_id, p.permission)
            },
            ProofResult::Composite(p) => {
                format!("Composite proof with {} sub-proofs: {}", 
                    p.sub_proofs.len(), if p.overall_result { "valid" } else { "invalid" })
            },
        }
    }
}

/// Builder pattern for creating proof configurations
pub struct ProofConfigBuilder {
    proof_type: ProofType,
    parameters: HashMap<String, ProofParameter>,
    security_level: u16,
    optimization_level: OptimizationLevel,
}

impl ProofConfigBuilder {
    pub fn new(proof_type: ProofType) -> Self {
        Self {
            proof_type,
            parameters: HashMap::new(),
            security_level: 128,
            optimization_level: OptimizationLevel::Balanced,
        }
    }

    pub fn with_parameter(mut self, key: &str, value: ProofParameter) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }

    pub fn with_security_level(mut self, level: u16) -> Self {
        self.security_level = level;
        self
    }

    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    pub fn build(self) -> ProofTypeConfig {
        ProofTypeConfig {
            proof_type: self.proof_type,
            parameters: self.parameters,
            security_level: self.security_level,
            optimization_level: self.optimization_level,
        }
    }
}

/// Convenience functions for common proof configurations
impl ProofTypeConfig {
    /// Create a content existence proof configuration
    pub fn content_existence(pattern: &str) -> Self {
        ProofConfigBuilder::new(ProofType::ContentExistence)
            .with_parameter("pattern", ProofParameter::String(pattern.to_string()))
            .build()
    }

    /// Create a data range proof configuration
    pub fn data_range(field: &str, min: f64, max: f64) -> Self {
        ProofConfigBuilder::new(ProofType::DataRange)
            .with_parameter("field", ProofParameter::String(field.to_string()))
            .with_parameter("range", ProofParameter::Range { min, max })
            .build()
    }

    /// Create a timestamp proof configuration
    pub fn timestamp_proof(timestamp_type: TimestampType, before: Option<i64>, after: Option<i64>) -> Self {
        ProofConfigBuilder::new(ProofType::TimestampProof)
            .with_parameter("type", ProofParameter::String(format!("{:?}", timestamp_type)))
            .with_parameter("bounds", ProofParameter::Timestamp { before, after })
            .build()
    }

    /// Create a format compliance proof configuration
    pub fn format_compliance(format_type: &str) -> Self {
        ProofConfigBuilder::new(ProofType::FormatCompliance)
            .with_parameter("format", ProofParameter::String(format_type.to_string()))
            .build()
    }
}

