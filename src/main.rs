mod command_line_arguments;
mod config;
mod reverse_proxy;

use crate::command_line_arguments::MinipxArguments;
use crate::config::Config;
use anyhow::Result;
use clap::Parser;
use log::{info, trace, LevelFilter};

#[tokio::main]
async fn main() -> Result<()> {
    let args = MinipxArguments::parse();
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(if args.verbose {
            LevelFilter::Trace
        } else {
            LevelFilter::Info
        })
        .init();

    info!("Starting minipx");
    trace!("Arguments: {:#?}", args);

    Config::try_load(&args.config_path).await?;
    if args.watch_config {
        Config::watch_config_file(args.config_path);
    }

    reverse_proxy::start_rp_server().await?;
    Ok(())
}
