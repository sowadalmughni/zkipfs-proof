use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::db::Db;

#[derive(Clone)]
pub struct AppState {
    pub jobs: Arc<RwLock<HashMap<String, Job>>>,
    pub db: Db,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "status", content = "result")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed(serde_json::Value),
    Failed(String),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Job {
    pub id: String,
    pub status: JobStatus,
    pub created_at: u64,
}
