use anyhow::Context as _;

use crate::context::Context;

mod android;
mod common;
mod desktop;
mod ios;
mod web;

pub fn add_tasks(ctx: &mut Context) -> anyhow::Result<()> {
    android::add_tasks(ctx)?;
    ios::add_tasks(ctx)?;
    desktop::add_tasks(ctx).context("Adding desktop tasks")?;
    web::add_tasks(ctx)?;
    Ok(())
}
