use std::fs;
use std::path::Path;

use anyhow::{Context as _, bail};
use resvg::{
    tiny_skia::{self, Pixmap},
    usvg::{self, Transform},
};

pub fn render_svgs_square_scaled(px: u32, svgs: &[(&usvg::Tree, f32)]) -> anyhow::Result<Pixmap> {
    let mut pixmap = tiny_skia::Pixmap::new(px, px).context("Creating pixmap")?;

    for (svg, scale) in svgs {
        if svg.size().width() != svg.size().height() {
            bail!("SVG should have square size");
        }

        let px_scale = px as f32 / svg.size().width();
        let tr = (1.0 - scale) * 0.5 * px as f32;
        let scale = px_scale * scale;

        resvg::render(
            svg,
            Transform::from_scale(scale, scale).post_translate(tr, tr),
            &mut pixmap.as_mut(),
        );
    }

    Ok(pixmap)
}

pub fn render_svgs_square(px: u32, svgs: &[&usvg::Tree]) -> anyhow::Result<Pixmap> {
    let svgs: Vec<_> = svgs.iter().map(|svg| (*svg, 1.0f32)).collect();
    render_svgs_square_scaled(px, &svgs)
}

pub fn read_svg(path: impl AsRef<Path>) -> anyhow::Result<usvg::Tree> {
    let path = path.as_ref();

    let data = fs::read(path).context(format!("Reading SVG `{:?}`", path))?;
    let svg = usvg::Tree::from_data(&data, &usvg::Options::default())
        .context(format!("Parsing SVG `{:?}`", path))?;

    Ok(svg)
}
