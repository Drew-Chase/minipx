use actix_multipart::Multipart;
use actix_web::{HttpResponse, Result as ActixResult, delete, get, post, put, web};
use chrono::Utc;
use futures_util::StreamExt;
use log::*;
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::http_error::Error;
use crate::models::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/servers")
            .service(list_servers)
            .service(create_server)
            .service(get_server)
            .service(update_server)
            .service(delete_server)
            .service(start_server)
            .service(stop_server)
            .service(restart_server)
            .service(upload_binary),
    );
}

#[get("")]
async fn list_servers(pool: web::Data<SqlitePool>) -> ActixResult<HttpResponse> {
    let servers = sqlx::query_as::<_, Server>("SELECT * FROM servers ORDER BY created_at DESC")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().json(servers))
}

#[get("/{id}")]
async fn get_server(pool: web::Data<SqlitePool>, id: web::Path<String>) -> ActixResult<HttpResponse> {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id.as_str())
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| Error::from(anyhow::anyhow!("Server not found")))?;

    Ok(HttpResponse::Ok().json(server))
}

#[post("")]
async fn create_server(pool: web::Data<SqlitePool>, req: web::Json<CreateServerRequest>) -> ActixResult<HttpResponse> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let host = req.host.clone().unwrap_or_else(|| "localhost".to_string());
    let path = req.path.clone().unwrap_or_default();
    let ssl_enabled = req.ssl_enabled.unwrap_or(false);
    let redirect_to_https = req.redirect_to_https.unwrap_or(false);

    // Create servers directory if it doesn't exist
    let servers_dir = PathBuf::from("servers").join(&id);
    fs::create_dir_all(&servers_dir).map_err(|e| Error::from(anyhow::anyhow!("Failed to create server directory: {}", e)))?;

    let binary_path = servers_dir.to_str().unwrap().to_string();

    sqlx::query(
        "INSERT INTO servers (id, name, domain, host, port, path, ssl_enabled, redirect_to_https, listen_port, status, binary_path, startup_command, runtime_id, main_executable, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.domain)
    .bind(&host)
    .bind(req.port as i64)
    .bind(&path)
    .bind(ssl_enabled)
    .bind(redirect_to_https)
    .bind(req.listen_port.map(|p| p as i64))
    .bind("stopped")
    .bind(&binary_path)
    .bind(&req.startup_command)
    .bind(&req.runtime_id)
    .bind(&req.main_executable)
    .bind(&now)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    // Add route to minipx config
    let mut config =
        minipx::config::Config::try_load("./minipx.json").await.map_err(|e| Error::from(anyhow::anyhow!("Failed to load config: {}", e)))?;

    let route = minipx::config::ProxyRoute::new(host.clone(), path.clone(), req.port, ssl_enabled, req.listen_port, redirect_to_https);

    config.add_route(req.domain.clone(), route).await.map_err(|e| Error::from(anyhow::anyhow!("Failed to add route: {}", e)))?;

    config.save().await.map_err(|e| Error::from(anyhow::anyhow!("Failed to save config: {}", e)))?;

    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(&id)
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    info!("Created server: {} ({})", server.name, server.id);
    Ok(HttpResponse::Created().json(server))
}

#[put("/{id}")]
async fn update_server(pool: web::Data<SqlitePool>, id: web::Path<String>, req: web::Json<UpdateServerRequest>) -> ActixResult<HttpResponse> {
    let now = Utc::now().to_rfc3339();

    // Get existing server
    let existing = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id.as_str())
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| Error::from(anyhow::anyhow!("Server not found")))?;

    let name = req.name.clone().unwrap_or(existing.name);
    let domain = req.domain.clone().unwrap_or(existing.domain.clone());
    let host = req.host.clone().unwrap_or(existing.host);
    let port = req.port.map(|p| p as i64).unwrap_or(existing.port);
    let path = req.path.clone().unwrap_or(existing.path);
    let ssl_enabled = req.ssl_enabled.unwrap_or(existing.ssl_enabled);
    let redirect_to_https = req.redirect_to_https.unwrap_or(existing.redirect_to_https);
    let listen_port = req.listen_port.map(|p| Some(p as i64)).unwrap_or(existing.listen_port);
    let status = req.status.clone().unwrap_or(existing.status);
    let startup_command = req.startup_command.clone().or(existing.startup_command);
    let runtime_id = req.runtime_id.clone().or(existing.runtime_id);
    let main_executable = req.main_executable.clone().or(existing.main_executable);

    sqlx::query(
        "UPDATE servers SET name = ?, domain = ?, host = ?, port = ?, path = ?,
         ssl_enabled = ?, redirect_to_https = ?, listen_port = ?, status = ?,
         startup_command = ?, runtime_id = ?, main_executable = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&name)
    .bind(&domain)
    .bind(&host)
    .bind(port)
    .bind(&path)
    .bind(ssl_enabled)
    .bind(redirect_to_https)
    .bind(listen_port)
    .bind(&status)
    .bind(&startup_command)
    .bind(&runtime_id)
    .bind(&main_executable)
    .bind(&now)
    .bind(id.as_str())
    .execute(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    // Update minipx config if domain changed
    if domain != existing.domain {
        let mut config =
            minipx::config::Config::try_load("./minipx.json").await.map_err(|e| Error::from(anyhow::anyhow!("Failed to load config: {}", e)))?;

        config.remove_route(&existing.domain).await.map_err(|e| Error::from(anyhow::anyhow!("Failed to remove old route: {}", e)))?;

        let route =
            minipx::config::ProxyRoute::new(host.clone(), path.clone(), port as u16, ssl_enabled, listen_port.map(|p| p as u16), redirect_to_https);

        config.add_route(domain.clone(), route).await.map_err(|e| Error::from(anyhow::anyhow!("Failed to add route: {}", e)))?;

        config.save().await.map_err(|e| Error::from(anyhow::anyhow!("Failed to save config: {}", e)))?;
    }

    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id.as_str())
        .fetch_one(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    info!("Updated server: {} ({})", server.name, server.id);
    Ok(HttpResponse::Ok().json(server))
}

#[delete("/{id}")]
async fn delete_server(pool: web::Data<SqlitePool>, id: web::Path<String>) -> ActixResult<HttpResponse> {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id.as_str())
        .fetch_optional(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| Error::from(anyhow::anyhow!("Server not found")))?;

    // Remove from database
    sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(id.as_str())
        .execute(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    // Remove from minipx config
    let mut config =
        minipx::config::Config::try_load("./minipx.json").await.map_err(|e| Error::from(anyhow::anyhow!("Failed to load config: {}", e)))?;

    config.remove_route(&server.domain).await.map_err(|e| Error::from(anyhow::anyhow!("Failed to remove route: {}", e)))?;

    config.save().await.map_err(|e| Error::from(anyhow::anyhow!("Failed to save config: {}", e)))?;

    // Delete server directory
    let _ = fs::remove_dir_all(&server.binary_path);

    info!("Deleted server: {} ({})", server.name, server.id);
    Ok(HttpResponse::NoContent().finish())
}

#[post("/{id}/start")]
async fn start_server(pool: web::Data<SqlitePool>, id: web::Path<String>) -> ActixResult<HttpResponse> {
    sqlx::query("UPDATE servers SET status = 'running' WHERE id = ?")
        .bind(id.as_str())
        .execute(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Server started"})))
}

#[post("/{id}/stop")]
async fn stop_server(pool: web::Data<SqlitePool>, id: web::Path<String>) -> ActixResult<HttpResponse> {
    sqlx::query("UPDATE servers SET status = 'stopped' WHERE id = ?")
        .bind(id.as_str())
        .execute(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Server stopped"})))
}

#[post("/{id}/restart")]
async fn restart_server(pool: web::Data<SqlitePool>, id: web::Path<String>) -> ActixResult<HttpResponse> {
    sqlx::query("UPDATE servers SET status = 'restarting' WHERE id = ?")
        .bind(id.as_str())
        .execute(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    // In a real implementation, you would actually restart the server process here

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    sqlx::query("UPDATE servers SET status = 'running' WHERE id = ?")
        .bind(id.as_str())
        .execute(pool.get_ref())
        .await
        .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "Server restarted"})))
}

#[post("/upload")]
async fn upload_binary(_pool: web::Data<SqlitePool>, mut payload: Multipart) -> ActixResult<HttpResponse> {
    let mut server_id: Option<String> = None;
    let mut file_saved = false;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| Error::from(anyhow::anyhow!("Multipart error: {}", e)))?;
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.and_then(|cd| cd.get_name()).unwrap_or("");

        if field_name == "serverId" {
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let data_chunk = chunk.map_err(|e| Error::from(anyhow::anyhow!("Chunk read error: {}", e)))?;
                data.extend_from_slice(&data_chunk);
            }
            server_id = Some(String::from_utf8_lossy(&data).to_string());
        } else if field_name == "file" {
            let filename = content_disposition.and_then(|cd| cd.get_filename()).unwrap_or("binary");

            if server_id.is_none() {
                return Err(Error::from(anyhow::anyhow!("serverId must be provided before file")).into());
            }

            let sid = server_id.as_ref().unwrap();
            let server_dir = PathBuf::from("servers").join(sid);
            fs::create_dir_all(&server_dir).map_err(|e| Error::from(anyhow::anyhow!("Failed to create directory: {}", e)))?;

            let filepath = server_dir.join(filename);
            let mut file = fs::File::create(&filepath).map_err(|e| Error::from(anyhow::anyhow!("Failed to create file: {}", e)))?;

            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| Error::from(anyhow::anyhow!("Chunk read error: {}", e)))?;
                use std::io::Write;
                file.write_all(&data).map_err(|e| Error::from(anyhow::anyhow!("Failed to write file: {}", e)))?;
            }

            // Check if it's an archive and extract if needed
            let extension = filepath.extension().and_then(|s| s.to_str()).unwrap_or("");
            match extension {
                "7z" | "zip" => {
                    // Extract 7z/zip archive to server directory
                    sevenz_rust::decompress_file(&filepath, &server_dir)
                        .map_err(|e| Error::from(anyhow::anyhow!("Failed to extract archive: {}", e)))?;

                    // Remove the archive file after extraction
                    let _ = fs::remove_file(&filepath);

                    info!("Extracted archive to {}", server_dir.display());
                }
                "tar" | "gz" | "tgz" => {
                    // Handle tar archives
                    info!("Tar archive support would be implemented here");
                }
                _ => {
                    // Just a binary file, leave it as is
                }
            }

            file_saved = true;
        }
    }

    if !file_saved {
        return Err(Error::from(anyhow::anyhow!("No file was uploaded")).into());
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"message": "File uploaded successfully"})))
}
