use actix_web::{web, HttpResponse, Result as ActixResult};
use actix_multipart::Multipart;
use futures_util::StreamExt;
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;
use log::*;

use crate::models::*;
use crate::http_error::Error;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/certificates")
            .route("", web::get().to(list_certificates))
            .route("", web::post().to(create_certificate))
            .route("/{id}", web::get().to(get_certificate))
            .route("/{id}", web::delete().to(delete_certificate))
            .route("/upload", web::post().to(upload_certificate))
    );
}

async fn list_certificates(pool: web::Data<SqlitePool>) -> ActixResult<HttpResponse> {
    let certificates = sqlx::query_as::<_, Certificate>(
        "SELECT * FROM certificates ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().json(certificates))
}

async fn get_certificate(
    pool: web::Data<SqlitePool>,
    id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let certificate = sqlx::query_as::<_, Certificate>(
        "SELECT * FROM certificates WHERE id = ?"
    )
    .bind(id.as_str())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?
    .ok_or_else(|| Error::from(anyhow::anyhow!("Certificate not found")))?;

    Ok(HttpResponse::Ok().json(certificate))
}

async fn create_certificate(
    pool: web::Data<SqlitePool>,
    req: web::Json<CreateCertificateRequest>,
) -> ActixResult<HttpResponse> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let is_letsencrypt = req.is_letsencrypt.unwrap_or(true);

    // For Let's Encrypt certificates, we don't need to store paths
    let cert_path = if is_letsencrypt {
        "letsencrypt".to_string()
    } else {
        format!("certificates/{}", id)
    };

    sqlx::query(
        "INSERT INTO certificates (id, name, domain, cert_path, is_letsencrypt, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.domain)
    .bind(&cert_path)
    .bind(is_letsencrypt)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    let certificate = sqlx::query_as::<_, Certificate>(
        "SELECT * FROM certificates WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    info!("Created certificate: {} ({})", certificate.name, certificate.id);
    Ok(HttpResponse::Created().json(certificate))
}

async fn delete_certificate(
    pool: web::Data<SqlitePool>,
    id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let certificate = sqlx::query_as::<_, Certificate>(
        "SELECT * FROM certificates WHERE id = ?"
    )
    .bind(id.as_str())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?
    .ok_or_else(|| Error::from(anyhow::anyhow!("Certificate not found")))?;

    sqlx::query("DELETE FROM certificates WHERE id = ?")
        .bind(id.as_str())
        .execute(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    // Delete certificate files if not Let's Encrypt
    if !certificate.is_letsencrypt {
        let cert_dir = PathBuf::from(&certificate.cert_path);
        let _ = fs::remove_dir_all(cert_dir);
    }

    info!("Deleted certificate: {} ({})", certificate.name, certificate.id);
    Ok(HttpResponse::NoContent().finish())
}

async fn upload_certificate(
    pool: web::Data<SqlitePool>,
    mut payload: Multipart,
) -> ActixResult<HttpResponse> {
    let mut cert_id: Option<String> = None;
    let mut cert_saved = false;
    let mut key_saved = false;

    // Create certificates directory
    fs::create_dir_all("certificates")
        .map_err(|e| Error::from(anyhow::anyhow!("Failed to create certificates dir: {}", e)))?;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| Error::from(anyhow::anyhow!("Multipart error: {}", e)))?;
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.and_then(|cd| cd.get_name()).map(|s| s.to_string()).unwrap_or_default();

        if field_name == "certificateId" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let data_chunk = chunk.map_err(|e| Error::from(anyhow::anyhow!("Chunk read error: {}", e)))?;
                data.extend_from_slice(&data_chunk);
            }
            cert_id = Some(String::from_utf8_lossy(&data).to_string());
        } else if field_name == "cert" || field_name == "key" {
            if cert_id.is_none() {
                return Err(Error::from(anyhow::anyhow!("certificateId must be provided first")).into());
            }

            let cid = cert_id.as_ref().unwrap();
            let cert_dir = PathBuf::from("certificates").join(cid);
            fs::create_dir_all(&cert_dir)
                .map_err(|e| Error::from(anyhow::anyhow!("Failed to create directory: {}", e)))?;

            let filename = if field_name == "cert" { "cert.pem" } else { "key.pem" };
            let filepath = cert_dir.join(filename);

            let mut file = fs::File::create(&filepath)
                .map_err(|e| Error::from(anyhow::anyhow!("Failed to create file: {}", e)))?;

            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| Error::from(anyhow::anyhow!("Chunk read error: {}", e)))?;
                use std::io::Write;
                file.write_all(&data)
                    .map_err(|e| Error::from(anyhow::anyhow!("Failed to write file: {}", e)))?;
            }

            if field_name == "cert" {
                cert_saved = true;
            } else {
                key_saved = true;
            }
        }
    }

    if !cert_saved {
        return Err(Error::from(anyhow::anyhow!("Certificate file is required")).into());
    }

    // Update certificate paths in database
    if let Some(cid) = cert_id {
        let cert_path = format!("certificates/{}/cert.pem", cid);
        let key_path = if key_saved {
            Some(format!("certificates/{}/key.pem", cid))
        } else {
            None
        };

        sqlx::query(
            "UPDATE certificates SET cert_path = ?, key_path = ?, is_letsencrypt = 0 WHERE id = ?"
        )
        .bind(&cert_path)
        .bind(&key_path)
        .bind(&cid)
        .execute(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Certificate uploaded successfully"})))
}
