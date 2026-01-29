use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "telecontrol",
    about = "Control via Telegram",
    version,
    author = "Steve Reed"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate a new private key. Pass to the start command.
    Privkey,
    /// Start the bot.
    Start {
        /// The private key to use for user key encryption.TC_PRIVATE_KEY env var is also
        /// available.
        #[arg(short, long)]
        privkey: Option<String>,
        /// Telegram Bot token. TC_BOT_TOKEN env var is also available.
        #[arg(short, long)]
        token: Option<String>,
        // Path to the database file. Defaults to $PWD/tcdb.lmdb
        #[arg(short, long, default_value = DEFAULT_DATABASE_PATH)]
        database: PathBuf,
    },
}

pub const DEFAULT_DATABASE_PATH: &str = "./tcdb.lmdb";
