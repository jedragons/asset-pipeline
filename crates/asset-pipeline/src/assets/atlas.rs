use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use aseprite::{AsepriteFile, Tag};
use resvg::{
    tiny_skia::{self, PixmapPaint},
    usvg::Transform,
};
use serde::{Deserialize, Serialize};

struct Sprite {
    pixmap: tiny_skia::Pixmap,
    name: String,
    duration: u32,
}

#[derive(Debug, Clone)]
pub struct Atlas {
    pub meta: AtlasMeta,
    pub image: tiny_skia::Pixmap,
}

impl Atlas {
    pub fn from_files(files: &[PathBuf]) -> anyhow::Result<Atlas> {
        let mut sprites = Vec::new();
        let mut animations = BTreeMap::new();
        read_sprites(files, &mut sprites, &mut animations).context("Reading sprites")?;
        let (atlas_size, sprites, frames) = pack_atlas(sprites);

        let mut image = tiny_skia::Pixmap::new(atlas_size, atlas_size).unwrap();

        for sprite in &sprites {
            let frame = &frames[&sprite.name];
            image.draw_pixmap(
                frame.x as i32,
                frame.y as i32,
                sprite.pixmap.as_ref(),
                &PixmapPaint::default(),
                Transform::default(),
                None,
            );
        }

        Ok(Atlas {
            meta: AtlasMeta { animations, frames },
            image,
        })
    }

    #[allow(unused)]
    pub fn from_folder(path: impl AsRef<Path>) -> anyhow::Result<Atlas> {
        let path = path.as_ref();
        let files: Vec<PathBuf> = path
            .read_dir()
            .with_context(|| format!("Reading directory {path:?}"))?
            .filter_map(|e| e.ok().map(|p| p.path()))
            .collect();

        Self::from_files(&files)
    }

    pub fn save(
        &self,
        json_path: impl AsRef<Path>,
        png_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let png_path = png_path.as_ref();
        let json_path = json_path.as_ref();

        self.image
            .save_png(png_path)
            .with_context(|| format!("Saving atlas PNG to {png_path:?}"))?;

        let json =
            serde_json::to_string(&self.meta).with_context(|| format!("Serializing atlas JSON"))?;
        fs::write(&json_path, json)
            .with_context(|| format!("Saving atlas JSON to {json_path:?}"))?;

        Ok(())
    }
}

fn read_sprites(
    paths: &[PathBuf],
    sprites: &mut Vec<Sprite>,
    animations: &mut BTreeMap<String, Vec<String>>,
) -> Result<(), anyhow::Error> {
    for path in paths {
        if path.extension() != Some(OsStr::new("ase"))
            && path.extension() != Some(OsStr::new("aseprite"))
        {
            println!("WARNING: `{path:?}` is not aseprite file");
            continue;
        }

        let file_name = path.file_stem().unwrap_or_default().to_string_lossy();

        let file = fs::File::open(&path).with_context(|| format!("Could not open `{path:?}`"))?;

        let ase = AsepriteFile::from_reader(file)
            .with_context(|| format!("`{path:?} is not valid aseprite file"))?;

        ase_read_sprites(ase, &file_name, sprites, animations)?;
    }

    Ok(())
}

fn ase_read_sprites(
    ase: AsepriteFile,
    name: &str,
    sprites: &mut Vec<Sprite>,
    animations: &mut BTreeMap<String, Vec<String>>,
) -> anyhow::Result<()> {
    if ase.tags().is_empty() {
        let mut animation = Vec::new();
        for (frame_index, frame) in ase.frames().iter().enumerate() {
            let name = read_frame(&ase, name, sprites, frame_index, frame, None)?;
            animation.push(name);
        }

        // TODO: check for duplicates
        animations.insert(name.to_owned(), animation);
    } else {
        for tag in ase.tags() {
            let mut animation = Vec::new();
            for frame_index in tag.from_frame..=tag.to_frame {
                let frame = &ase.frames()[frame_index];
                let name = read_frame(&ase, name, sprites, frame_index, frame, Some(tag))?;
                animation.push(name);
            }
            let animation_name = format!("{}-{}", name, tag.name.clone());

            // TODO: check for duplicates
            animations.insert(animation_name, animation);
        }
    }

    Ok(())
}

fn read_frame(
    ase: &AsepriteFile,
    name: &str,
    sprites: &mut Vec<Sprite>,
    frame_index: usize,
    frame: &aseprite::Frame,
    tag: Option<&Tag>,
) -> Result<String, anyhow::Error> {
    let mut pixmap = tiny_skia::Pixmap::new(ase.width() as u32, ase.height() as u32).unwrap();
    for (layer_index, layer_meta) in ase.layers().iter().enumerate() {
        let layer = ase
            .layer_ref(layer_index)
            .with_context(|| format!("Ase does not contain layer with index {layer_index}"))?;

        let cel = ase.cel(layer, frame_index).with_context(|| {
            format!("Ase does not contain cel for layer {layer_index} and frame {frame_index}")
        })?;

        let (pixels, x, y) = match cel.kind {
            aseprite::CelKind::Raw { ref pixels, x, y } => (pixels, x, y),
            aseprite::CelKind::Compressed {
                ref pixels, x, y, ..
            } => (pixels, x, y),
            _ => {
                println!(
                    "WARNING: cel at layer {} and frame {} has unsupported kind",
                    layer_meta.name, frame_index
                );
                continue;
            }
        };

        // TODO: sort by z-order
        let frame_pixmap = tiny_skia::Pixmap::try_from(pixels.clone())?;
        pixmap.as_mut().draw_pixmap(
            x as i32,
            y as i32,
            frame_pixmap.as_ref(),
            &PixmapPaint::default(),
            Transform::default(),
            None,
        );
    }

    let name = match tag {
        None => format!("{name}-{frame_index}"),
        Some(tag) => format!("{name}-{}-{}", tag.name, frame_index - tag.from_frame),
    };

    sprites.push(Sprite {
        pixmap,
        name: name.clone(),
        duration: frame.duration_ms as u32,
    });

    Ok(name)
}

fn try_pack_shelf(sprites: &[Sprite], atlas_size: u32) -> Option<BTreeMap<String, AtlasFrame>> {
    let mut frames = BTreeMap::new();

    let mut cursor_x = 0u32;
    let mut cursor_y = 0u32;
    let mut shelf_height = 0u32;

    for sprite in sprites {
        let (w, h) = (sprite.pixmap.width(), sprite.pixmap.height());

        if w > atlas_size {
            return None;
        }

        if cursor_x + w > atlas_size {
            cursor_x = 0;
            cursor_y += shelf_height;
            shelf_height = 0;
        }

        if cursor_y + h > atlas_size {
            return None;
        }

        // TODO: check for duplicates
        frames.insert(
            sprite.name.clone(),
            AtlasFrame {
                x: cursor_x,
                y: cursor_y,
                w,
                h,
                duration: sprite.duration,
            },
        );

        cursor_x += w;
        shelf_height = shelf_height.max(h);
    }

    Some(frames)
}

fn pack_atlas(mut sprites: Vec<Sprite>) -> (u32, Vec<Sprite>, BTreeMap<String, AtlasFrame>) {
    sprites.sort_by_key(|b| std::cmp::Reverse(b.pixmap.height()));

    let mut size = 16u32;
    loop {
        if let Some(frames) = try_pack_shelf(&sprites, size) {
            return (size, sprites, frames);
        }
        size *= 2;

        if size > 2048 {
            panic!("Sprites do not fit into an 8192x8192 atlas");
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AtlasMeta {
    animations: BTreeMap<String, Vec<String>>,
    frames: BTreeMap<String, AtlasFrame>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AtlasFrame {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub duration: u32,
}
