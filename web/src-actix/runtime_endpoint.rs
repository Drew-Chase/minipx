use actix_web::{HttpResponse, Result as ActixResult, get, post, web};
use log::*;
use sqlx::SqlitePool;

use crate::http_error::Error;
use crate::models::Runtime;
use crate::runtime_detector;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/runtimes").service(list_runtimes).service(detect_and_store_runtimes).service(scan_archive));
}

#[get("")]
async fn list_runtimes(pool: web::Data<SqlitePool>) -> ActixResult<HttpResponse> {
    let runtimes = sqlx::query_as::<_, Runtime>("SELECT * FROM runtimes WHERE is_available = 1 ORDER BY runtime_type, name")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().json(runtimes))
}

#[post("/detect")]
async fn detect_and_store_runtimes(pool: web::Data<SqlitePool>) -> ActixResult<HttpResponse> {
    // Clear existing runtimes
    sqlx::query("DELETE FROM runtimes").execute(pool.get_ref()).await.map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    // Detect runtimes
    let runtimes = runtime_detector::detect_runtimes().map_err(|e| Error::from(anyhow::anyhow!("Runtime detection error: {}", e)))?;

    // Store detected runtimes in database
    for runtime in &runtimes {
        sqlx::query(
            "INSERT INTO runtimes (id, name, display_name, version, executable_path, runtime_type, detected_at, is_available)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&runtime.id)
        .bind(&runtime.name)
        .bind(&runtime.display_name)
        .bind(&runtime.version)
        .bind(&runtime.executable_path)
        .bind(&runtime.runtime_type)
        .bind(&runtime.detected_at)
        .bind(runtime.is_available)
        .execute(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;
    }

    info!("Detected and stored {} runtimes", runtimes.len());
    Ok(HttpResponse::Ok().json(runtimes))
}

#[post("/scan-archive")]
async fn scan_archive(body: web::Json<ScanArchiveRequest>) -> ActixResult<HttpResponse> {
    // This will be implemented client-side with WASM
    // For now, we'll return a placeholder response
    info!("Received archive scan request for {} files", body.files.len());

    let executables: Vec<String> = body.files.iter().filter(|f| is_executable_file(f)).cloned().collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "executables": executables
    })))
}

fn is_executable_file(filename: &str) -> bool {
    let executable_extensions = vec![".exe", ".jar", ".dll", ".so", ".dylib", ".sh", ".bat", ".cmd", ".ps1"];
    executable_extensions.iter().any(|ext| filename.to_lowercase().ends_with(ext))
}

#[derive(serde::Deserialize)]
struct ScanArchiveRequest {
    files: Vec<String>,
}
