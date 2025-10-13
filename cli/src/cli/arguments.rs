use crate::config::{Config, ProxyRoute, RoutePatch};
use anyhow::Result;
use clap::{ArgAction, Args, Parser, Subcommand};
use log::{error, info};

#[derive(Parser, Debug, Clone)]
#[command(name = "minipx", about, author, version, long_about = None, propagate_version = true)]
pub struct MinipxArguments {
    #[arg(short = 'c', long = "config", help = "Path to the configuration file (overrides running instance)")]
    pub(crate) config_path: Option<String>,
    #[arg(short = 'v', long = "verbose", help = "Enable verbose logging")]
    pub(crate) verbose: bool,
    #[arg(short = 'w', long = "watch", help = "Watch the configuration file for changes")]
    pub(crate) watch_config: bool,
    #[command(subcommand)]
    pub(crate) command: Option<MinipxCommands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum MinipxCommands {
    #[clap(name = "routes", about = "Manage proxy routes")]
    Routes {
        #[clap(subcommand)]
        command: RouteCommands,
    },
    #[clap(name = "config", about = "Manage the configuration file")]
    Config {
        #[clap(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum RouteCommands {
    #[clap(name = "add", about = "Add a new proxy route")]
    AddRoute {
        #[clap(flatten)]
        routes: ProxyRoute,
        domain: String,
    },
    #[clap(name = "remove", about = "Remove a proxy route")]
    RemoveRoute { host: String },
    #[clap(name = "list", about = "List all proxy routes")]
    ListRoutes,
    #[clap(name = "show", about = "Show a proxy route")]
    ShowRoute { host: String },
    #[clap(name = "update", about = "Update a proxy route (partial)")]
    UpdateRoute {
        /// Domain of the route to update (the route key, e.g., example.com)
        domain: String,
        #[clap(flatten)]
        patch: UpdateRouteOptions,
    },
    #[clap(name = "addsub", about = "Add a subroute to an existing proxy route")]
    AddSubroute {
        /// Domain of the existing route to add the subroute to
        domain: String,
        /// Path for the subroute (e.g. /path/to/subroute)
        path: String,
        /// Port to route the subroute to
        port: u16,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    #[clap(name = "show", about = "Show the current configuration")]
    Show,
    #[clap(name = "email", about = "Set the email address to use for SSL certificates")]
    Email { email: String },
    #[clap(name = "show-path", about = "Show the path to the configuration file")]
    ShowPath,
}

// Optional fields for partial updates. Only provided flags will be applied.
#[derive(Args, Debug, Clone, Default)]
pub struct UpdateRouteOptions {
    /// Backend host (e.g. 127.0.0.1)
    #[arg(id = "backend-host", short = 'j', long = "host")]
    pub host: Option<String>,
    /// Backend path (e.g., web or api/v1) — do not start with '/'
    #[arg(short = 'p', long = "path")]
    pub path: Option<String>,
    /// Backend port (1..=65535, not 80/443)
    #[arg(short = 'P', long = "port")]
    pub port: Option<u16>,

    /// Enable SSL for this route (frontend terminates TLS)
    #[arg(short = 's', long = "ssl", action = ArgAction::SetTrue, conflicts_with = "no_ssl")]
    pub ssl: bool,
    /// Disable SSL for this route
    #[arg(long = "no-ssl", action = ArgAction::SetTrue)]
    pub no_ssl: bool,

    /// Redirect HTTP to HTTPS for this route
    #[arg(short = 'r', long = "redirect", action = ArgAction::SetTrue, conflicts_with = "no_redirect")]
    pub redirect: bool,
    /// Disable HTTP to HTTPS redirect
    #[arg(long = "no-redirect", action = ArgAction::SetTrue)]
    pub no_redirect: bool,
}

impl From<UpdateRouteOptions> for RoutePatch {
    fn from(o: UpdateRouteOptions) -> Self {
        RoutePatch {
            host: o.host,
            path: o.path,
            port: o.port,
            ssl_enable: if o.ssl {
                Some(true)
            } else if o.no_ssl {
                Some(false)
            } else {
                None
            },
            redirect_to_https: if o.redirect {
                Some(true)
            } else if o.no_redirect {
                Some(false)
            } else {
                None
            },
            listen_port: None,
        }
    }
}

impl MinipxArguments {
    pub async fn handle_arguments(&self) -> Result<()> {
        if let Some(command) = &self.command {
            let effective_config_path = Config::resolve_config_path(self.config_path.clone()).await;
            let mut config = Config::try_load(&effective_config_path).await?;
            match command {
                // ---
                // Routes subcommand
                // ---
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
                        for (domain, route) in config.get_routes() {
                            println!(
                                "\x1b[1;36m{}\x1b[0m: \x1b[1;33m{}\x1b[0m -> \x1b[1;32m{}:{}\x1b[0m/\x1b[1;35m{}\x1b[0m",
                                domain,
                                match (route.get_listen_port(), route.is_ssl_enabled()) {
                                    (Some(port), _) => port.to_string(),
                                    (_, true) => "HTTPS".to_string(),
                                    (_, false) => "HTTP".to_string(),
                                },
                                route.get_host(),
                                route.get_port(),
                                route.get_path()
                            );
                        }
                    }
                    RouteCommands::ShowRoute { host } => {
                        if let Some(route) = config.lookup_host(host) {
                            println!(
                                "\x1b[1;36m{}\x1b[0m: \x1b[1;33m{}\x1b[0m -> \x1b[1;32m{}:{}\x1b[0m/\x1b[1;35m{}\x1b[0m",
                                host,
                                match (route.get_listen_port(), route.is_ssl_enabled()) {
                                    (Some(port), _) => port.to_string(),
                                    (_, true) => "HTTPS".to_string(),
                                    (_, false) => "HTTP".to_string(),
                                },
                                route.get_host(),
                                route.get_port(),
                                route.get_path()
                            );
                        } else {
                            error!("Route not found: {}", host);
                        }
                    }
                    RouteCommands::AddSubroute { domain, path, port } => {
                        config.add_subroute(domain, path.clone(), *port).await?;
                        config.save().await?;
                        info!("Added subroute to {}: {} -> port {}", domain, path, port);
                    }
                },

                // ---
                // Config subcommand
                // ---
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
            // Exit after the command has been executed
            std::process::exit(0);
        }
        Ok(())
    }
}