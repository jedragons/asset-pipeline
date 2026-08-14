use std::sync::mpsc::channel;
use std::{fs, path::PathBuf};

use anyhow::{Context as _, bail};
use clap::Parser as _;
use notify::{EventHandler, Watcher};
use serde::{Deserialize, Serialize};

use crate::{cli::Cli, context::Context};

mod assets;
mod cli;
mod context;
mod platforms;
mod task;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Config {
    input: Option<String>,
    output: Option<String>,
}

struct FileEventHandler {
    ctx: Context,
}

impl EventHandler for FileEventHandler {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        let Ok(_event) = event else {
            return;
        };

        // TODO: If this operation becomes expensive, implement better solution
        self.ctx.task_store.clear();
        if let Err(e) = platforms::add_tasks(&mut self.ctx) {
            println!("ERROR: Could not recreate task list: {e:?}");
            return;
        }
        self.ctx.task_store.execute_all();
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    match args.command.unwrap_or_default() {
        cli::Command::Config { key, value } => {
            let mut config = read_config().context("Reading config").unwrap_or_default();
            match key.as_str() {
                "input" => config.input = Some(value),
                "output" => config.output = Some(value),
                _ => bail!("Unknown config key {key:?}"),
            }
            write_config(&config)?;
        }

        cli::Command::Build { input, output } => {
            build_assets(input, output)?;
        }

        cli::Command::Watch { input, output } => {
            println!("Rebuilding assets");
            let ctx = build_assets(input, output)?;
            let input = ctx.input.clone();

            println!("Watching directory {input:?}");
            let mut handler = notify::recommended_watcher(FileEventHandler { ctx })?;
            handler.watch(&input, notify::RecursiveMode::Recursive)?;

            let (tx, rx) = channel();
            ctrlc::set_handler(move || {
                let _ = tx.send(());
            })?;
            rx.recv()?;
        }
    };

    Ok(())
}

fn build_assets(
    mut input: Option<PathBuf>,
    mut output: Option<PathBuf>,
) -> anyhow::Result<Context> {
    if let Ok(config) = read_config() {
        input = input.or_else(|| config.input.map(PathBuf::from));
        output = output.or_else(|| config.output.map(PathBuf::from));
    }

    let input = input.context("You must specify Resource directory with `-i/--input` flag")?;
    let output = output.context("You must specify output directory with `-o/--output` flag")?;

    let mut ctx = Context::new(input, output);
    platforms::add_tasks(&mut ctx).context("Adding tasks")?;
    ctx.task_store.execute_all();
    Ok(ctx)
}

fn read_config() -> anyhow::Result<Config> {
    let path = get_config_dir()?.join("assets.json");
    let data = fs::read(path)?;
    let config: Config = serde_json::from_slice(&data)?;
    Ok(config)
}

fn write_config(config: &Config) -> anyhow::Result<()> {
    let path = get_config_dir()?.join("assets.json");
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

fn get_config_dir() -> anyhow::Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("Could not get config directory")?
        .join("alienwave");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
