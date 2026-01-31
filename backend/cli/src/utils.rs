//! Utility functions for the zkIPFS-Proof CLI
//!
//! This module provides common utility functions used throughout the CLI,
//! including input validation, formatting, and file operations.

use std::path::Path;
use zkipfs_proof_core::{ContentSelection, error::{ProofError, Result}};

/// Parse content selection from string format
pub fn parse_content_selection(input: &str) -> Result<ContentSelection> {
    if input.contains(',') {
        // Multiple selections
        let selections: Result<Vec<ContentSelection>> = input
            .split(',')
            .map(|s| parse_single_content_selection(s.trim()))
            .collect();
        
        Ok(ContentSelection::Multiple(selections?))
    } else {
        parse_single_content_selection(input)
    }
}

/// Parse a single content selection
fn parse_single_content_selection(input: &str) -> Result<ContentSelection> {
    if let Some(pattern_content) = input.strip_prefix("pattern:") {
        Ok(ContentSelection::Pattern {
            content: pattern_content.as_bytes().to_vec(),
        })
    } else if let Some(regex_pattern) = input.strip_prefix("regex:") {
        Ok(ContentSelection::Regex {
            pattern: regex_pattern.to_string(),
        })
    } else if let Some(xpath_selector) = input.strip_prefix("xpath:") {
        Ok(ContentSelection::XPath {
            selector: xpath_selector.to_string(),
        })
    } else if let Some(range_spec) = input.strip_prefix("range:") {
        let parts: Vec<&str> = range_spec.split(':').collect();
        if parts.len() != 2 {
            return Err(ProofError::invalid_input_error(
                "content_selection",
                "Range format should be 'range:start:end'"
            ));
        }
        
        let start: usize = parts[0].parse()
            .map_err(|_| ProofError::invalid_input_error(
                "content_selection",
                "Invalid start position in range"
            ))?;
        
        let end: usize = parts[1].parse()
            .map_err(|_| ProofError::invalid_input_error(
                "content_selection",
                "Invalid end position in range"
            ))?;
        
        if start >= end {
            return Err(ProofError::invalid_input_error(
                "content_selection",
                "Start position must be less than end position"
            ));
        }
        
        Ok(ContentSelection::ByteRange { start, end })
    } else {
        // Default to pattern if no prefix
        Ok(ContentSelection::Pattern {
            content: input.as_bytes().to_vec(),
        })
    }
}

/// Validate that a file path exists and is readable
pub fn validate_file_path(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(ProofError::invalid_input_error(
            "file_path",
            format!("File does not exist: {}", path.display())
        ));
    }
    
    if !path.is_file() {
        return Err(ProofError::invalid_input_error(
            "file_path",
            format!("Path is not a file: {}", path.display())
        ));
    }
    
    // Check if file is readable
    match std::fs::File::open(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProofError::file_error(
            format!("Cannot read file: {}", path.display()),
            Some(e)
        )),
    }
}

/// Format duration in milliseconds to human-readable string
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        let minutes = ms / 60_000;
        let seconds = (ms % 60_000) / 1000;
        format!("{}m {}s", minutes, seconds)
    } else {
        let hours = ms / 3_600_000;
        let minutes = (ms % 3_600_000) / 60_000;
        format!("{}h {}m", hours, minutes)
    }
}

/// Format bytes to human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format hash as hex string with optional truncation
pub fn format_hash(hash: &[u8], truncate: Option<usize>) -> String {
    let hex_string = hex::encode(hash);
    
    if let Some(len) = truncate {
        if hex_string.len() > len {
            format!("{}...", &hex_string[..len])
        } else {
            hex_string
        }
    } else {
        hex_string
    }
}

/// Validate hex string and convert to bytes
pub fn parse_hex_string(hex_str: &str) -> Result<Vec<u8>> {
    // Remove 0x prefix if present
    let clean_hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    
    hex::decode(clean_hex)
        .map_err(|e| ProofError::invalid_input_error(
            "hex_string",
            format!("Invalid hex string: {}", e)
        ))
}

/// Check if a string is a valid JSON
pub fn is_valid_json(s: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(s).is_ok()
}

/// Pretty print JSON string
pub fn pretty_print_json(json_str: &str) -> Result<String> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| ProofError::serialization_error(
            "Invalid JSON string",
            Some(Box::new(e))
        ))?;
    
    serde_json::to_string_pretty(&value)
        .map_err(|e| ProofError::serialization_error(
            "Failed to format JSON",
            Some(Box::new(e))
        ))
}

/// Get file size in bytes
pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| ProofError::file_error(
            format!("Failed to get file metadata: {}", path.display()),
            Some(e)
        ))?;
    
    Ok(metadata.len())
}

/// Check if output file can be written
pub fn validate_output_path(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(ProofError::invalid_input_error(
            "output_path",
            format!("Output file already exists: {}. Use --force to overwrite", path.display())
        ));
    }
    
    // Check if parent directory exists and is writable
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Err(ProofError::invalid_input_error(
                "output_path",
                format!("Output directory does not exist: {}", parent.display())
            ));
        }
        
        // Try to create a temporary file to test write permissions
        let temp_file = parent.join(".zkipfs_write_test");
        match std::fs::write(&temp_file, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&temp_file);
                Ok(())
            }
            Err(e) => Err(ProofError::file_error(
                format!("Cannot write to output directory: {}", parent.display()),
                Some(e)
            )),
        }
    } else {
        Ok(())
    }
}

/// Truncate string to specified length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Convert relative path to absolute path
pub fn to_absolute_path(path: &Path) -> Result<std::path::PathBuf> {
    path.canonicalize()
        .map_err(|e| ProofError::file_error(
            format!("Failed to resolve absolute path: {}", path.display()),
            Some(e)
        ))
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get system information
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        family: std::env::consts::FAMILY.to_string(),
        cpu_count: num_cpus::get(),
        available_memory: get_available_memory(),
    }
}

/// System information structure
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub cpu_count: usize,
    pub available_memory: Option<u64>,
}

/// Get available system memory in bytes (best effort)
fn get_available_memory() -> Option<u64> {
    // This is a simplified implementation
    // In a real implementation, you might use system-specific APIs
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if line.starts_with("MemAvailable:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return Some(kb * 1024); // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_content_selection() {
        // Test pattern
        let selection = parse_content_selection("pattern:hello world").unwrap();
        match selection {
            ContentSelection::Pattern { content } => {
                assert_eq!(content, b"hello world");
            }
            _ => panic!("Expected pattern selection"),
        }

        // Test regex
        let selection = parse_content_selection("regex:^hello").unwrap();
        match selection {
            ContentSelection::Regex { pattern } => {
                assert_eq!(pattern, "^hello");
            }
            _ => panic!("Expected regex selection"),
        }

        // Test range
        let selection = parse_content_selection("range:10:20").unwrap();
        match selection {
            ContentSelection::ByteRange { start, end } => {
                assert_eq!(start, 10);
                assert_eq!(end, 20);
            }
            _ => panic!("Expected range selection"),
        }

        // Test multiple
        let selection = parse_content_selection("pattern:hello,range:10:20").unwrap();
        match selection {
            ContentSelection::Multiple(selections) => {
                assert_eq!(selections.len(), 2);
            }
            _ => panic!("Expected multiple selections"),
        }

        // Test default (pattern without prefix)
        let selection = parse_content_selection("hello").unwrap();
        match selection {
            ContentSelection::Pattern { content } => {
                assert_eq!(content, b"hello");
            }
            _ => panic!("Expected pattern selection"),
        }
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(65000), "1m 5s");
        assert_eq!(format_duration(3665000), "1h 1m");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_format_hash() {
        let hash = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        assert_eq!(format_hash(&hash, None), "123456789abcdef0");
        assert_eq!(format_hash(&hash, Some(8)), "12345678...");
        assert_eq!(format_hash(&hash, Some(20)), "123456789abcdef0");
    }

    #[test]
    fn test_parse_hex_string() {
        let result = parse_hex_string("0x123456").unwrap();
        assert_eq!(result, vec![0x12, 0x34, 0x56]);

        let result = parse_hex_string("abcdef").unwrap();
        assert_eq!(result, vec![0xab, 0xcd, 0xef]);

        assert!(parse_hex_string("invalid").is_err());
    }

    #[test]
    fn test_validate_file_path() {
        // Test with a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        
        let result = validate_file_path(temp_file.path());
        assert!(result.is_ok());

        // Test with non-existent file
        let result = validate_file_path(std::path::Path::new("/non/existent/file"));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_json() {
        assert!(is_valid_json(r#"{"key": "value"}"#));
        assert!(is_valid_json("[]"));
        assert!(is_valid_json("null"));
        assert!(!is_valid_json("invalid json"));
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 2), "hi");
        assert_eq!(truncate_string("hello", 3), "...");
    }

    #[test]
    fn test_get_system_info() {
        let info = get_system_info();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(info.cpu_count > 0);
    }
}

