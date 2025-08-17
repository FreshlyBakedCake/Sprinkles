use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "notifications")]
    Notification {
        #[command(subcommand)]
        subcommand: NotificationSubcommand,
    },
}

#[derive(Subcommand)]
pub enum NotificationSubcommand {
    Server,
    Query {
        query: String,
        #[arg(long, conflicts_with = "raw", action = ArgAction::SetTrue, default_value_t = false)]
        json: bool,
        #[arg(long, conflicts_with = "json", action = ArgAction::SetTrue, default_value_t = false)]
        raw: bool,
    },
}
