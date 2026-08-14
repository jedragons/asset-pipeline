use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, bail};

use crate::assets::atlas::Atlas;
use crate::context::Context;

pub fn sprites_dir(resource_dir: impl AsRef<Path>) -> PathBuf {
    resource_dir.as_ref().join("sprites")
}

fn add_atlas_tasks(ctx: &mut Context, output: impl AsRef<Path>) -> anyhow::Result<()> {
    for entry in fs::read_dir(sprites_dir(&ctx.input)).context("Reading sprites directory")? {
        let path = match entry {
            Ok(e) => e.path(),
            Err(e) => {
                println!("WARNING: could not read entry: {e:?}");
                continue;
            }
        };

        if !path.is_dir() {
            println!("WARNING: `{path:?}` is not a directory");
        }

        let files: Vec<PathBuf> = path
            .read_dir()
            .with_context(|| format!("Reading entry {path:?}"))?
            .filter_map(|e| e.ok().map(|p| p.path()))
            .collect();

        if files.is_empty() {
            continue;
        }

        let name = path.file_name().unwrap_or_default();
        let output = output.as_ref().join("sprites").join(name);

        ctx.task_store
            .add(
                atlas_task,
                &files,
                &[output.with_extension("json"), output.with_extension("png")],
            )
            .with_context(|| format!("Adding atlas task for {path:?}"))?;
    }

    Ok(())
}

pub fn add_common_tasks(ctx: &mut Context, output: impl AsRef<Path>) -> anyhow::Result<()> {
    add_atlas_tasks(ctx, output).context("Adding atlas tasks")?;
    Ok(())
}

fn atlas_task(inputs: &[PathBuf], outputs: &[PathBuf]) -> anyhow::Result<()> {
    let atlas = Atlas::from_files(inputs).context("Loading atlas")?;

    let [json, png] = outputs else {
        bail!("Atlas task must produce exactly two outputs");
    };

    if let Some(parent) = json.parent() {
        fs::create_dir_all(parent)?;
    }

    atlas
        .save(json, png)
        .with_context(|| format!("Saving atlas"))?;

    println!("TASK: generated atlas image {png:?}",);
    println!("TASK: generated atlas metadata {json:?}",);

    Ok(())
}
