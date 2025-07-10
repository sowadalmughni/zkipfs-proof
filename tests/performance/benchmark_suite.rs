//! Performance benchmarks for zkIPFS-Proof

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use zkipfs_proof_core::{
    proof::{ProofGenerator, ProofVerifier},
    types::{FileInfo, ProofConfig},
};
use std::time::Duration;
use tempfile::NamedTempFile;
use std::io::Write;

fn create_test_file(size_kb: usize, pattern: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    
    let chunk_size = 1024;
    let base_chunk = "A".repeat(chunk_size - pattern.len() - 1);
    let total_chunks = size_kb;
    
    for i in 0..total_chunks {
        if i == total_chunks / 2 {
            // Insert pattern in the middle
            file.write_all(pattern.as_bytes()).expect("Failed to write pattern");
            file.write_all(base_chunk.as_bytes()).expect("Failed to write chunk");
        } else {
            file.write_all(base_chunk.as_bytes()).expect("Failed to write chunk");
        }
    }
    
    file
}

fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_generation");
    group.measurement_time(Duration::from_secs(30));
    
    let config = ProofConfig {
        security_level: 128,
        chunk_size: 4096,
        max_file_size: 100 * 1024 * 1024, // 100MB
        enable_compression: false,
    };
    
    let generator = ProofGenerator::new(config).expect("Failed to create generator");
    let pattern = "SECRET_PATTERN_12345";
    
    // Test different file sizes
    for size_kb in [1, 10, 100, 1000, 10000].iter() {
        let file = create_test_file(*size_kb, pattern);
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        
        group.bench_with_input(
            BenchmarkId::new("file_size_kb", size_kb),
            size_kb,
            |b, _| {
                b.iter(|| {
                    let result = generator.generate_proof(
                        black_box(&file_info),
                        black_box(pattern)
                    );
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_proof_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_verification");
    
    let config = ProofConfig::default();
    let generator = ProofGenerator::new(config.clone()).expect("Failed to create generator");
    let verifier = ProofVerifier::new(config).expect("Failed to create verifier");
    let pattern = "VERIFICATION_PATTERN_67890";
    
    // Pre-generate proofs for different file sizes
    let mut proofs = Vec::new();
    for size_kb in [1, 10, 100, 1000].iter() {
        let file = create_test_file(*size_kb, pattern);
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        let proof_result = generator.generate_proof(&file_info, pattern)
            .expect("Failed to generate proof");
        proofs.push((*size_kb, proof_result.proof_data));
    }
    
    for (size_kb, proof_data) in proofs.iter() {
        group.bench_with_input(
            BenchmarkId::new("file_size_kb", size_kb),
            size_kb,
            |b, _| {
                b.iter(|| {
                    let result = verifier.verify_proof(
                        black_box(proof_data),
                        black_box(pattern)
                    );
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_security_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_levels");
    
    let pattern = "SECURITY_TEST_PATTERN";
    let file = create_test_file(100, pattern); // 100KB file
    let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
    
    for security_level in [128, 192, 256].iter() {
        let config = ProofConfig {
            security_level: *security_level,
            chunk_size: 4096,
            max_file_size: 10 * 1024 * 1024,
            enable_compression: false,
        };
        
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        
        group.bench_with_input(
            BenchmarkId::new("security_bits", security_level),
            security_level,
            |b, _| {
                b.iter(|| {
                    let result = generator.generate_proof(
                        black_box(&file_info),
                        black_box(pattern)
                    );
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_chunk_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_sizes");
    
    let pattern = "CHUNK_SIZE_TEST_PATTERN";
    let file = create_test_file(1000, pattern); // 1MB file
    let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
    
    for chunk_size in [1024, 2048, 4096, 8192, 16384].iter() {
        let config = ProofConfig {
            security_level: 128,
            chunk_size: *chunk_size,
            max_file_size: 10 * 1024 * 1024,
            enable_compression: false,
        };
        
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        
        group.bench_with_input(
            BenchmarkId::new("chunk_bytes", chunk_size),
            chunk_size,
            |b, _| {
                b.iter(|| {
                    let result = generator.generate_proof(
                        black_box(&file_info),
                        black_box(pattern)
                    );
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_compression_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression_impact");
    
    let pattern = "COMPRESSION_TEST_PATTERN";
    let file = create_test_file(1000, pattern); // 1MB file
    let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
    
    for enable_compression in [false, true].iter() {
        let config = ProofConfig {
            security_level: 128,
            chunk_size: 4096,
            max_file_size: 10 * 1024 * 1024,
            enable_compression: *enable_compression,
        };
        
        let generator = ProofGenerator::new(config).expect("Failed to create generator");
        
        group.bench_with_input(
            BenchmarkId::new("compression_enabled", enable_compression),
            enable_compression,
            |b, _| {
                b.iter(|| {
                    let result = generator.generate_proof(
                        black_box(&file_info),
                        black_box(pattern)
                    );
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    
    let config = ProofConfig::default();
    let pattern = "CONCURRENT_TEST_PATTERN";
    let file = create_test_file(100, pattern); // 100KB file
    let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
    
    for thread_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("thread_count", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count).map(|_| {
                        let config = config.clone();
                        let file_info = file_info.clone();
                        let pattern = pattern.to_string();
                        
                        std::thread::spawn(move || {
                            let generator = ProofGenerator::new(config).expect("Failed to create generator");
                            let result = generator.generate_proof(&file_info, &pattern);
                            black_box(result)
                        })
                    }).collect();
                    
                    for handle in handles {
                        handle.join().expect("Thread panicked");
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(60));
    
    let config = ProofConfig::default();
    let generator = ProofGenerator::new(config).expect("Failed to create generator");
    let pattern = "MEMORY_TEST_PATTERN";
    
    // Test memory usage with very large files
    for size_mb in [1, 5, 10, 25, 50].iter() {
        let size_kb = size_mb * 1024;
        let file = create_test_file(size_kb, pattern);
        let file_info = FileInfo::from_path(file.path()).expect("Failed to create FileInfo");
        
        group.bench_with_input(
            BenchmarkId::new("file_size_mb", size_mb),
            size_mb,
            |b, _| {
                b.iter(|| {
                    let result = generator.generate_proof(
                        black_box(&file_info),
                        black_box(pattern)
                    );
                    black_box(result)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_proof_generation,
    bench_proof_verification,
    bench_security_levels,
    bench_chunk_sizes,
    bench_compression_impact,
    bench_concurrent_operations,
    bench_memory_usage
);

criterion_main!(benches);

