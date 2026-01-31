use axum::{
    extract::{State, Path, Multipart},
    response::{Json, IntoResponse},
    http::StatusCode,
};
use std::path::PathBuf;
use uuid::Uuid;
use crate::state::{AppState, Job, JobStatus};
use zkipfs_proof_core::{ProofGenerator, ProofConfig, ContentSelection};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

#[derive(serde::Deserialize)]
pub struct GenerateRequest {
    // Fields are parsed manually from multipart
}

pub async fn generate_proof(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let job_id = Uuid::new_v4().to_string();
    let job_id_clone = job_id.clone();
    
    // Create initial job
    {
        let mut jobs = state.jobs.write().unwrap();
        jobs.insert(job_id.clone(), Job {
            id: job_id.clone(),
            status: JobStatus::Pending,
            created_at: chrono::Utc::now().timestamp() as u64,
        });
    }

    // Spawn processing task
    tokio::spawn(async move {
        match process_proof_request(state.clone(), job_id_clone.clone(), multipart).await {
            Ok(proof) => {
                let mut jobs = state.jobs.write().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = JobStatus::Completed(serde_json::to_value(proof).unwrap());
                }
            }
            Err(e) => {
                let mut jobs = state.jobs.write().unwrap();
                if let Some(job) = jobs.get_mut(&job_id_clone) {
                    job.status = JobStatus::Failed(e.to_string());
                }
            }
        }
    });

    Json(serde_json::json!({ "job_id": job_id }))
}

pub async fn get_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let jobs = state.jobs.read().unwrap();
    if let Some(job) = jobs.get(&id) {
        Json(job.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Job not found").into_response()
    }
}

async fn process_proof_request(
    state: AppState,
    job_id: String,
    mut multipart: Multipart,
) -> anyhow::Result<zkipfs_proof_core::types::Proof> {
    // Update status to processing
    {
        let mut jobs = state.jobs.write().unwrap();
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Processing;
        }
    }

    let mut file_path: Option<PathBuf> = None;
    let mut content_str: Option<String> = None;
    let mut security_level = 128;
    // directory to keep temp file
    let temp_dir = tempfile::tempdir()?;
    
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.tmp").to_string();
            let path = temp_dir.path().join(file_name);
            let mut file = tokio::fs::File::create(&path).await?;
            let data = field.bytes().await?;
            file.write_all(&data).await?;
            file_path = Some(path);
        } else if name == "content_selection" {
             content_str = Some(field.text().await?);
        } else if name == "security_level" {
             if let Ok(val) = field.text().await?.parse::<u32>() {
                 security_level = val;
             }
        }
    }

    let file_path = file_path.ok_or_else(|| anyhow::anyhow!("No file uploaded"))?;
    let content_str = content_str.unwrap_or_else(|| "pattern: ".to_string()); // Default or error?

    // Parse content selection (Basic parsing similar to CLI utils, implemented here for simplicity)
    // In a real implementation, we should expose `parse_content_selection` from CLI or move it to Core.
    // Assuming standard formats: "pattern:...", "regex:...", "xpath:...", "range:..."
    let selection = if let Some(stripped) = content_str.strip_prefix("pattern:") {
         ContentSelection::Pattern { content: stripped.as_bytes().to_vec() }
    } else if let Some(stripped) = content_str.strip_prefix("regex:") {
         ContentSelection::Regex { pattern: stripped.to_string() }
    } else if let Some(stripped) = content_str.strip_prefix("xpath:") {
         ContentSelection::XPath { selector: stripped.to_string() }
    } else if let Some(stripped) = content_str.strip_prefix("range:") {
         let parts: Vec<&str> = stripped.split(':').collect();
         if parts.len() == 2 {
             let start: usize = parts[0].parse()?;
             let end: usize = parts[1].parse()?;
             ContentSelection::ByteRange { start, end }
         } else {
             // Fallback to pattern
             ContentSelection::Pattern { content: content_str.as_bytes().to_vec() }
         }
    } else {
         // Default
          ContentSelection::Pattern { content: content_str.as_bytes().to_vec() }
    };

    let config = ProofConfig {
        security_level,
        use_hardware_acceleration: true,
        prover_type: zkipfs_proof_core::ProverType::Local,
        ..ProofConfig::default()
    };

    let mut generator = ProofGenerator::with_config(config).await?;
    let proof = generator.generate_proof(&file_path, selection).await?;

    Ok(proof)
}

// Enterprise Handlers

pub async fn create_api_key(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // In real world, authenticated user creates a key. Here we simulate "admin" creating it or open creation for demo.
    // For demo/POC: Allow open creation or require a "master" key?
    // Let's assume open for now or require a secret hardcoded in env?
    // For simplicity: Open endpoint for generating keys.
    match state.db.create_api_key("demo_user").await {
        Ok((key, raw)) => Json(serde_json::json!({
            "key": raw,
            "id": key.id,
            "created_at": key.created_at
        })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

pub async fn list_api_keys(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.db.list_keys().await {
        Ok(keys) => Json(keys).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

pub async fn revoke_api_key(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.db.revoke_key(&id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}
