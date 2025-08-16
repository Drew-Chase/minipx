mod command_line_arguments;
mod config;
mod ipc;
mod reverse_proxy;
mod ssl_server;

use crate::command_line_arguments::{ConfigCommands, MinipxArguments, MinipxCommands, RouteCommands};
use crate::config::Config;
use anyhow::Result;
use clap::Parser;
use log::{LevelFilter, error, info, trace};

async fn resolve_config_path(arg: Option<String>) -> String {
    // Priority: explicit CLI arg > IPC from running process > default
    if let Some(s) = arg
        && !s.trim().is_empty()
    {
        return s;
    }
    if let Some(path) = ipc::get_running_config_path().await {
        return path;
    }
    "./minipx.json".to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = MinipxArguments::parse();
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(if args.verbose { LevelFilter::Trace } else { LevelFilter::Info })
        .init();

    // Determine which config path to use up front for both CLI and daemon modes.
    let effective_config_path = resolve_config_path(args.config_path.clone()).await;

    if let Some(command) = &args.command {
        let mut config = Config::try_load(&effective_config_path).await?;
        match command {
            MinipxCommands::Routes { command } => match command {
                RouteCommands::AddRoute { domain, routes } => {
                    config.add_route(domain.clone(), routes.clone()).await?;
                    config.save().await?;
                }
                RouteCommands::RemoveRoute { host } => {
                    config.remove_route(host).await?;
                    config.save().await?;
                }
                RouteCommands::UpdateRoute { domain, patch } => {
                    let patch = (*patch).clone().into();
                    config.update_route(domain, patch).await?;
                    config.save().await?;
                    info!("Updated route: {}", domain);
                }
                RouteCommands::ListRoutes => {
                    println!("{}", serde_json::to_string_pretty(config.get_routes())?);
                }
                RouteCommands::ShowRoute { host } => {
                    if let Some(route) = config.lookup_host(host) {
                        println!("{}", serde_json::to_string_pretty(route)?);
                    } else {
                        error!("Route not found: {}", host);
                    }
                }
            },
            MinipxCommands::Config { command } => match command {
                ConfigCommands::Show => {
                    println!("{}", config);
                }
                ConfigCommands::Email { email } => {
                    config.set_email(email.clone());
                    config.save().await?;
                }
                ConfigCommands::ShowPath => {
                    println!("{}", config.path.to_string_lossy())
                }
            },
        }
        return Ok(());
    }

    info!("Starting minipx");
    trace!("Arguments: {:#?}", args);

    let config = Config::try_load(&effective_config_path).await?;
    if args.watch_config {
        config.watch_config_file();
    }

    // Start IPC server to advertise our config path for tooling/CLI.
    ipc::start_ipc_server(std::path::PathBuf::from(&effective_config_path));

    // Run HTTP and HTTPS servers concurrently
    tokio::try_join!(reverse_proxy::start_rp_server(), ssl_server::start_ssl_server())?;

    Ok(())
}
