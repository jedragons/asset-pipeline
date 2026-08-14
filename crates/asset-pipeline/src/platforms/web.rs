use std::path::PathBuf;

use crate::{context::Context, platforms::common::add_common_tasks};

pub fn add_tasks(ctx: &mut Context) -> anyhow::Result<()> {
    add_common_tasks(ctx, assets_dir(ctx))?;
    Ok(())
}

fn assets_dir(ctx: &Context) -> PathBuf {
    ctx.output.join("web").join("assets")
}
