use std::{fs, path::PathBuf};

use anyhow::Context as _;

use crate::{
    assets::svg::{read_svg, render_svgs_square},
    context::Context,
};

pub fn add_tasks(ctx: &mut Context) -> anyhow::Result<()> {
    add_ios_icon_tasks(ctx)?;
    Ok(())
}

const IOS_ICON_PX: u32 = 1024;

fn add_ios_icon_tasks(ctx: &mut Context) -> anyhow::Result<()> {
    let bg_path = ctx.input.join("art").join("icon-background.svg");
    let fg_path = ctx.input.join("art").join("icon-foreground.svg");
    let inputs = [bg_path.clone(), fg_path.clone()];

    let output_png = appiconset_dir(ctx).join("icon-1024.png");
    let output_json = appiconset_dir(ctx).join("Contents.json");
    let outputs = [output_png.clone(), output_json.clone()];

    ctx.task_store.add(
        move |_, _| {
            let icon_bg = read_svg(&bg_path)?;
            let icon_fg = read_svg(&fg_path)?;

            if let Some(parent) = output_png.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Creating directory `{parent:?}`"))?;
            }

            render_svgs_square(IOS_ICON_PX, &[&icon_bg, &icon_fg])?.save_png(&output_png)?;

            let contents_json = r#"{
  "images" : [
    {
      "filename" : "icon-1024.png",
      "idiom" : "universal",
      "platform" : "ios",
      "size" : "1024x1024"
    }
  ],
  "info" : {
    "author" : "xcode",
    "version" : 1
  }
}
"#;
            fs::write(&output_json, contents_json).context("Writing Contents.json")?;

            println!("TASK: generated iOS icon {output_png:?}");
            println!("TASK: generated iOS icon description {output_json:?}");

            Ok(())
        },
        &inputs,
        &outputs,
    )?;

    Ok(())
}

fn appiconset_dir(ctx: &Context) -> PathBuf {
    ctx.output
        .join("ios")
        .join("Assets.xcassets")
        .join("AppIcon.appiconset")
}
