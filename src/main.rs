mod command_line_arguments;
mod config;
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

    info!("Starting minipx");
    trace!("Arguments: {:#?}", args);

    Config::try_load(&args.config_path).await?;
    if args.watch_config {
        Config::watch_config_file(args.config_path);
    }

    // Run HTTP and HTTPS servers concurrently
    tokio::try_join!(
        reverse_proxy::start_rp_server(),
        ssl_server::start_ssl_server()
    )?;

    Ok(())
}
