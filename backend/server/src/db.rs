use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::Arc;
use anyhow::Result;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct Db {
    pub pool: Pool<Sqlite>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ApiKey {
    pub id: String,
    pub key_hash: String,
    pub owner: String,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create the database file if it doesn't exist (handled by sqlx usually if configured or manually)
        // SqliteConnectOptions::new().create_if_missing(true)
        // But SqlitePoolOptions doesn't expose that directly easily without options.
        // Let's use connect_with
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(database_url)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        let db = Db { pool };
        db.migrate().await?;
        Ok(db)
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                key_hash TEXT NOT NULL,
                owner TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                revoked BOOLEAN NOT NULL DEFAULT FALSE,
                last_used_at DATETIME
            )",
        )
        .execute(&self.pool)
        .await?;

         sqlx::query(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                result JSON,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_api_key(&self, owner: &str) -> Result<(ApiKey, String)> {
        let salt = SaltString::generate(&mut OsRng);
        let api_key_raw = Uuid::new_v4().to_string().replace("-", "") + &Uuid::new_v4().to_string().replace("-", ""); 
        
        let mut hasher = Sha256::new();
        hasher.update(api_key_raw.as_bytes());
        let key_hash = hex::encode(hasher.finalize());
        
        // We use fast hasing (SHA256) for lookup/verification of machine keys. 
        // Argon2 is for passwords. 
        
        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            key_hash: key_hash.clone(),
            owner: owner.to_string(),
            created_at: Utc::now(),
            revoked: false,
            last_used_at: None,
        };

        sqlx::query(
            "INSERT INTO api_keys (id, key_hash, owner, created_at, revoked) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&api_key.id)
        .bind(&api_key.key_hash)
        .bind(&api_key.owner)
        .bind(&api_key.created_at)
        .bind(&api_key.revoked)
        .execute(&self.pool)
        .await?;

        Ok((api_key, api_key_raw))
    }

    pub async fn validate_api_key(&self, raw_key: &str) -> Result<Option<ApiKey>> {
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let hash = hex::encode(hasher.finalize());
        
        let key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = ? AND revoked = FALSE"
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(ref k) = key {
             let _ = sqlx::query("UPDATE api_keys SET last_used_at = ? WHERE id = ?")
                .bind(Utc::now())
                .bind(&k.id)
                .execute(&self.pool)
                .await;
        }

        Ok(key)
    }

    pub async fn list_keys(&self) -> Result<Vec<ApiKey>> {
        let keys = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(keys)
    }

    pub async fn revoke_key(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE api_keys SET revoked = TRUE WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
