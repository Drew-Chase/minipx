use clap::Parser;

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(name = "minipx", about, author, version, long_about = None)]
pub struct MinipxArguments {
    #[arg(
        short = 'c',
        long = "config",
        help = "Path to the configuration file",
        default_value = "./minipx.json"
    )]
    pub(crate) config_path: String,
    #[arg(short = 'v', long = "verbose", help = "Enable verbose logging")]
    pub(crate) verbose: bool,
    #[arg(short='w', long = "watch", help = "Watch the configuration file for changes")]
    pub(crate) watch_config: bool,
}
