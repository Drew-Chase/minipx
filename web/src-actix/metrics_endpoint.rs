use actix_web::{web, HttpResponse, Result as ActixResult};
use sqlx::SqlitePool;
use sysinfo::{System, Disks, Networks};
use uuid::Uuid;
use chrono::Utc;

use crate::models::*;
use crate::http_error::Error;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/metrics")
            .route("/system", web::get().to(get_system_stats))
            .route("/server/{id}", web::get().to(get_server_metrics))
            .route("/server/{id}/history", web::get().to(get_server_metrics_history))
    );
}

async fn get_system_stats() -> ActixResult<HttpResponse> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    let cpu_usage = sys.global_cpu_usage() as f64;

    let (disk_total, disk_used) = disks.iter().fold((0u64, 0u64), |(total, used), disk| {
        (total + disk.total_space(), used + (disk.total_space() - disk.available_space()))
    });

    let disk_usage = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    let (network_in, network_out) = networks.iter().fold((0u64, 0u64), |(rx, tx), (_, network)| {
        (rx + network.received(), tx + network.transmitted())
    });

    let stats = SystemStats {
        cpu_usage,
        memory_usage,
        memory_total: total_memory,
        memory_used: used_memory,
        disk_usage,
        disk_total,
        disk_used,
        network_in: network_in as f64,
        network_out: network_out as f64,
    };

    Ok(HttpResponse::Ok().json(stats))
}

async fn get_server_metrics(
    pool: web::Data<SqlitePool>,
    id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    // Check if server exists
    let _server = sqlx::query_as::<_, crate::models::Server>(
        "SELECT * FROM servers WHERE id = ?"
    )
    .bind(id.as_str())
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?
    .ok_or_else(|| Error::from(anyhow::anyhow!("Server not found")))?;

    // Get system stats (in a real implementation, you'd track per-process metrics)
    let mut sys = System::new_all();
    sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();

    // Simulate server-specific metrics (in reality, you'd track the actual process)
    let cpu_usage = (sys.global_cpu_usage() as f64 * 0.1).min(100.0); // Fake: 10% of system
    let memory_usage = (sys.used_memory() as f64 / sys.total_memory() as f64) * 10.0; // Fake: 10% relative

    let (disk_total, disk_used) = disks.iter().fold((0u64, 0u64), |(total, used), disk| {
        (total + disk.total_space(), used + (disk.total_space() - disk.available_space()))
    });
    let disk_usage = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    let (network_in, network_out) = networks.iter().fold((0u64, 0u64), |(rx, tx), (_, network)| {
        (rx + network.received(), tx + network.transmitted())
    });

    // Store metric in database
    let metric_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO resource_metrics (id, server_id, cpu_usage, memory_usage, disk_usage, network_in, network_out, timestamp)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&metric_id)
    .bind(id.as_str())
    .bind(cpu_usage)
    .bind(memory_usage)
    .bind(disk_usage)
    .bind(network_in as f64)
    .bind(network_out as f64)
    .bind(&now)
    .execute(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    let metric = ResourceMetric {
        id: metric_id,
        server_id: id.to_string(),
        cpu_usage,
        memory_usage,
        disk_usage,
        network_in: network_in as f64,
        network_out: network_out as f64,
        timestamp: now,
    };

    Ok(HttpResponse::Ok().json(metric))
}

async fn get_server_metrics_history(
    pool: web::Data<SqlitePool>,
    id: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let metrics = sqlx::query_as::<_, ResourceMetric>(
        "SELECT * FROM resource_metrics WHERE server_id = ? ORDER BY timestamp DESC LIMIT 100"
    )
    .bind(id.as_str())
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| Error::from(anyhow::anyhow!("Database error: {}", e)))?;

    Ok(HttpResponse::Ok().json(metrics))
}
