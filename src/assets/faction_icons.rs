//! Faction emblem icons — 256×256 PNGs loaded at startup.

#![allow(dead_code)]

use crate::core::Faction;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct FactionIconsPlugin;

impl Plugin for FactionIconsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionIconCache>()
            .add_systems(Startup, load_faction_icons);
    }
}

#[derive(Resource, Default)]
pub struct FactionIconCache {
    pub icons: HashMap<Faction, Handle<Image>>,
    /// Non-empire emblems: "triglavian", "deathless".
    pub extra: HashMap<&'static str, Handle<Image>>,
}

impl FactionIconCache {
    pub fn get(&self, f: Faction) -> Option<Handle<Image>> {
        self.icons.get(&f).cloned()
    }
    pub fn get_extra(&self, key: &str) -> Option<Handle<Image>> {
        self.extra.get(key).cloned()
    }
}

fn filename(f: Faction) -> &'static str {
    match f {
        Faction::Amarr => "amarr.png",
        Faction::Caldari => "caldari.png",
        Faction::Gallente => "gallente.png",
        Faction::Minmatar => "minmatar.png",
    }
}

fn load_faction_icons(mut cache: ResMut<FactionIconCache>, mut images: ResMut<Assets<Image>>) {
    let dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("assets")
        .join("factions");

    if !dir.exists() {
        warn!("Faction icons directory not found: {:?}", dir);
        return;
    }

    for faction in [
        Faction::Amarr,
        Faction::Caldari,
        Faction::Gallente,
        Faction::Minmatar,
    ] {
        let path = dir.join(filename(faction));
        match load_png(&path) {
            Ok(image) => {
                let handle = images.add(image);
                cache.icons.insert(faction, handle);
                info!("Loaded faction icon: {:?}", faction);
            }
            Err(e) => warn!("Failed to load {:?}: {}", faction, e),
        }
    }

    for key in ["triglavian", "deathless"] {
        let path = dir.join(format!("{}.png", key));
        match load_png(&path) {
            Ok(image) => {
                let handle = images.add(image);
                cache.extra.insert(key, handle);
                info!("Loaded extra emblem: {}", key);
            }
            Err(e) => warn!("Failed to load {}: {}", key, e),
        }
    }

    info!(
        "Loaded {} faction + {} extra emblems",
        cache.icons.len(),
        cache.extra.len()
    );
}

fn load_png(path: &PathBuf) -> Result<Image, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| e.to_string())?
        .into_rgba8();
    let (w, h) = img.dimensions();
    let data = img.into_raw();
    Ok(Image::new(
        bevy::render::render_resource::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}
