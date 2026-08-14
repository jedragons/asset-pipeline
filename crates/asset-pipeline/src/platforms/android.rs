use std::{fs, path::PathBuf};

use anyhow::Context as _;

use crate::{
    assets::svg::{read_svg, render_svgs_square, render_svgs_square_scaled},
    context::Context,
    platforms::common::add_common_tasks,
};

const ANDROID_DENSITIES: &[(&str, f32)] = &[
    ("mdpi", 1.0),
    ("hdpi", 1.5),
    ("xhdpi", 2.0),
    ("xxhdpi", 3.0),
    ("xxxhdpi", 4.0),
];

const ADAPTIVE_ICON_DP: f32 = 108.0;
const LEGACY_ICON_DP: f32 = 48.0;
const SAFE_SCALE: f32 = 0.55;

pub fn add_tasks(ctx: &mut Context) -> anyhow::Result<()> {
    add_common_tasks(ctx, assets_dir(ctx))?;
    ctx.task_store.add(
        xml_values_task,
        &[],
        &[res_dir(ctx).join("values").join("strings.xml")],
    )?;
    add_android_icon_tasks(ctx)?;
    Ok(())
}

fn xml_values_task(_inputs: &[PathBuf], outputs: &[PathBuf]) -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="app_name">Alienwave</string>
</resources>"#;

    for output in outputs {
        if let Some(values_dir) = output.parent() {
            fs::create_dir_all(&values_dir)
                .with_context(|| format!("Creating directory `{values_dir:?}`"))?;
        }

        fs::write(output, xml)?;
        println!("TASK: generated Android values {output:?}");
    }

    Ok(())
}

fn add_android_icon_tasks(ctx: &mut Context) -> anyhow::Result<()> {
    let bg_path = ctx.input.join("art").join("icon-background.svg");
    let fg_path = ctx.input.join("art").join("icon-foreground.svg");
    let res = res_dir(ctx);

    let inputs = [bg_path.clone(), fg_path.clone()];
    let mut outputs = Vec::new();
    for (density, _) in ANDROID_DENSITIES {
        let mipmap_dir = res.join(format!("mipmap-{density}"));
        outputs.push(mipmap_dir.join("ic_launcher_background.png"));
        outputs.push(mipmap_dir.join("ic_launcher_foreground.png"));
        outputs.push(mipmap_dir.join("ic_launcher.png"));
    }

    let anydpi_dir = res.join("mipmap-anydpi-v26");
    let anydpi_ic = anydpi_dir.join("ic_launcher.xml");
    let anydpi_ic_round = anydpi_dir.join("ic_launcher_round.xml");
    outputs.push(anydpi_ic.clone());
    outputs.push(anydpi_ic_round.clone());

    ctx.task_store.add(
        move |_, _| {
            let icon_bg = read_svg(&bg_path)?;
            let icon_fg = read_svg(&fg_path)?;

            for (density, scale) in ANDROID_DENSITIES {
                let mipmap_dir = res.join(format!("mipmap-{density}"));
                let ic_bg = mipmap_dir.join("ic_launcher_background.png");
                let ic_fg = mipmap_dir.join("ic_launcher_foreground.png");
                let ic = mipmap_dir.join("ic_launcher.png");

                fs::create_dir_all(&mipmap_dir)
                    .with_context(|| format!("Creating `mipmap-{density}` directory"))?;

                let adaptive_px = (ADAPTIVE_ICON_DP * scale).round() as u32;
                render_svgs_square(adaptive_px, &[&icon_bg])?.save_png(&ic_bg)?;
                render_svgs_square_scaled(adaptive_px, &[(&icon_fg, SAFE_SCALE)])?
                    .save_png(&ic_fg)?;

                let legacy_px = (LEGACY_ICON_DP * scale).round() as u32;
                render_svgs_square(legacy_px, &[&icon_bg, &icon_fg])?.save_png(&ic)?;

                println!("TASK: generated Android icon {ic_bg:?}");
                println!("TASK: generated Android icon {ic_fg:?}");
                println!("TASK: generated Android icon {ic:?}");
            }

            fs::create_dir_all(&anydpi_dir).context("Creating `mipmap-anydpi-v26` directory")?;
            let adaptive_icon_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@mipmap/ic_launcher_background"/>
    <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
</adaptive-icon>
"#;
            fs::write(&anydpi_ic, adaptive_icon_xml).context("Writing ic_launcher.xml")?;
            fs::write(&anydpi_ic_round, adaptive_icon_xml)
                .context("Writing ic_launcher_round.xml")?;

            println!("TASK: generated Android XML icon {anydpi_ic:?}");
            println!("TASK: generated Android XML round icon {anydpi_ic_round:?}");

            Ok(())
        },
        &inputs,
        &outputs,
    )?;

    Ok(())
}

fn assets_dir(ctx: &Context) -> PathBuf {
    ctx.output.join("android").join("assets")
}

fn res_dir(ctx: &Context) -> PathBuf {
    ctx.output.join("android").join("res")
}
