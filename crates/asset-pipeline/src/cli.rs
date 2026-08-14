use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Config {
        /// Config key to set
        key: String,

        /// Value for given key
        value: String,
    },

    Build {
        /// Path to Resources directory
        #[arg(long, short)]
        input: Option<PathBuf>,

        /// Path to generated assets directory
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    Watch {
        /// Path to Resources directory
        #[arg(long, short)]
        input: Option<PathBuf>,

        /// Path to generated assets directory
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}

impl Default for Command {
    fn default() -> Self {
        Self::Watch {
            input: None,
            output: None,
        }
    }
}
