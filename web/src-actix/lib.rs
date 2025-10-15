use crate::asset_endpoint::AssetsAppConfig;
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use anyhow::Result;
use log::*;
use serde_json::json;
use std::env::set_current_dir;
use vite_actix::proxy_vite_options::ProxyViteOptions;
use vite_actix::start_vite_server;

mod asset_endpoint;
mod certificate_endpoint;
mod db;
mod http_error;
mod metrics_endpoint;
mod models;
mod runtime_detector;
mod runtime_endpoint;
mod server_endpoint;
mod test_endpoint;

pub static DEBUG: bool = cfg!(debug_assertions);
const PORT: u16 = 6671;

pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder().filter_level(if DEBUG { LevelFilter::Debug } else { LevelFilter::Info }).format_timestamp(None).init();

    // Start the Vite server in development mode
    if DEBUG {
        // setup serde hashids
        serde_hash::hashids::SerdeHashOptions::new().with_min_length(16).with_salt("minipx-web-panel").build();

        let dev_env_path = "target/dev-env";
        std::fs::create_dir_all(dev_env_path)?;
        set_current_dir(dev_env_path)?;

        ProxyViteOptions::new().disable_logging().working_directory("../../web").build()?;
        std::thread::spawn(|| {
            loop {
                info!("Starting Vite server in development mode...");
                let status = start_vite_server().expect("Failed to start vite server").wait().expect("Vite server crashed!");
                if !status.success() {
                    error!("The vite server has crashed!");
                } else {
                    break;
                }
            }
        });
    }

    // Initialize database
    let pool = db::init_database().await?;
    info!("Database initialized successfully");
    let pool_data = web::Data::new(pool);

    // Start background system stats refresher
    let stats_tx = metrics_endpoint::spawn_system_stats_refresher();
    info!("System stats refresher started");
    let stats_data = web::Data::new(stats_tx);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .app_data(stats_data.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", "*"))
                    .add(("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS"))
                    .add(("Access-Control-Allow-Headers", "Content-Type, Authorization")),
            )
            .app_data(web::JsonConfig::default().limit(8192).error_handler(|err, _req| {
                let error = json!({ "error": format!("{}", err) });
                actix_web::error::InternalError::from_response(err, HttpResponse::BadRequest().json(error)).into()
            }))
            .app_data(
                actix_multipart::form::MultipartFormConfig::default().total_limit(512 * 1024 * 1024), // 512 MB limit for file uploads
            )
            .service(
                web::scope("/api")
                    .configure(test_endpoint::configure)
                    .configure(server_endpoint::configure)
                    .configure(certificate_endpoint::configure)
                    .configure(metrics_endpoint::configure)
                    .configure(runtime_endpoint::configure),
            )
            .configure_frontend_routes()
    })
    .workers(4)
    .bind(format!("0.0.0.0:{port}", port = PORT))?
    .run();

    info!("Starting {} server at http://127.0.0.1:{}...", if DEBUG { "development" } else { "production" }, PORT);

    let stop_result = server.await;
    debug!("Server stopped");

    Ok(stop_result?)
}
