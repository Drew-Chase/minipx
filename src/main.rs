mod command_line_arguments;
mod config;
mod ipc;
mod reverse_proxy;
mod ssl_server;

use crate::command_line_arguments::MinipxArguments;
use crate::config::Config;
use anyhow::Result;
use clap::Parser;
use log::{LevelFilter, info, trace};

#[tokio::main]
async fn main() -> Result<()> {
    let args = MinipxArguments::parse();
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(if args.verbose { LevelFilter::Trace } else { LevelFilter::Info })
        .init();

    // Handle command line arguments
    args.handle_arguments().await?;

    info!("Starting minipx");
    trace!("Arguments: {:#?}", args);

    let effective_config_path = Config::resolve_config_path(args.config_path.clone()).await;
    let config = Config::try_load(&effective_config_path).await?;
    if args.watch_config {
        config.watch_config_file();
    }

    ipc::start_ipc_server(std::path::PathBuf::from(&effective_config_path));

    // Run HTTP and HTTPS servers concurrently
    tokio::try_join!(reverse_proxy::start_rp_server(), ssl_server::start_ssl_server())?;

    Ok(())
}
