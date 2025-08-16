mod command_line_arguments;
mod config;
mod reverse_proxy;
mod ssl_server;

use crate::command_line_arguments::{ConfigCommands, MinipxArguments, MinipxCommands, RouteCommands};
use crate::config::Config;
use anyhow::Result;
use clap::Parser;
use log::{LevelFilter, info, trace, error};

#[tokio::main]
async fn main() -> Result<()> {
    let args = MinipxArguments::parse();
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(if args.verbose { LevelFilter::Trace } else { LevelFilter::Info })
        .init();

    if let Some(command) = &args.command {
        let mut config = Config::try_load(&args.config_path).await?;
        match command {
            MinipxCommands::Routes { command } => {
                match command {
                    RouteCommands::AddRoute{domain, routes} => {
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
                }
            },
            MinipxCommands::Config {command}=>{
                match command {
                    ConfigCommands::Show => {
                        println!("{}", config);
                    },
                    ConfigCommands::Email{email}=> {
                        config.set_email(email.clone());
                        config.save().await?;
                    }
                }
            }
        }
        return Ok(());
    }


    info!("Starting minipx");
    trace!("Arguments: {:#?}", args);

    let config = Config::try_load(&args.config_path).await?;
    if args.watch_config {
        config.watch_config_file();
    }

    // Run HTTP and HTTPS servers concurrently
    tokio::try_join!(reverse_proxy::start_rp_server(), ssl_server::start_ssl_server())?;

    Ok(())
}
