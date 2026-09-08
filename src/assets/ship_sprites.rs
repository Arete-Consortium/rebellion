//! Ship Sprite Loading
//!
//! Loads ship sprites from bundled assets, with fallback to CCP's Image Server.
//! Priority: assets/ships/{type_id}.png -> cache -> download

#![allow(dead_code)]

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core::*;

/// Image Server base URL (fallback only)
const IMAGE_SERVER: &str = "https://images.evetech.net";

/// Default render size for downloads
const RENDER_SIZE: u32 = 128;

/// Bundled assets directory
const BUNDLED_SHIPS_DIR: &str = "assets/ships";

/// Ship sprites plugin
pub struct ShipSpritesPlugin;

impl Plugin for ShipSpritesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipSpriteCache>()
            .add_systems(Startup, setup_sprite_cache)
            .add_systems(OnEnter(GameState::Loading), start_loading_sprites)
            .add_systems(
                Update,
                check_sprite_loading.run_if(in_state(GameState::Loading)),
            );
    }
}

/// Cache of loaded ship sprite handles
#[derive(Resource, Default)]
pub struct ShipSpriteCache {
    /// Map of type_id -> texture handle
    pub sprites: HashMap<u32, Handle<Image>>,
    /// Ships currently being loaded
    pub loading: Vec<u32>,
    /// Whether initial load is complete
    pub ready: bool,
    /// Cache directory path
    pub cache_dir: PathBuf,
}

impl ShipSpriteCache {
    /// Get sprite for a ship type, returns None if not loaded
    pub fn get(&self, type_id: u32) -> Option<Handle<Image>> {
        self.sprites.get(&type_id).cloned()
    }
}

/// The bundled registry owns the preload list, so new hulls cannot silently
/// fall back to a colored box because a second list was not updated.
fn ships_to_load() -> &'static [u32] {
    static IDS: std::sync::OnceLock<Vec<u32>> = std::sync::OnceLock::new();
    IDS.get_or_init(|| {
        #[derive(serde::Deserialize)]
        struct Manifest {
            ships: Vec<Hull>,
        }
        #[derive(serde::Deserialize)]
        struct Hull {
            type_id: u32,
        }
        let manifest: Manifest =
            serde_json::from_str(include_str!("../../assets/ships/ship_manifest.json"))
                .expect("bundled ship manifest must be valid");
        manifest
            .ships
            .into_iter()
            .map(|hull| hull.type_id)
            .collect()
    })
}

/// Setup the sprite cache directory (native only)
#[cfg(not(target_arch = "wasm32"))]
fn setup_sprite_cache(mut cache: ResMut<ShipSpriteCache>) {
    // Use home directory cache
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rebellion")
        .join("sprites");

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&cache_dir) {
        warn!("Failed to create sprite cache dir: {}", e);
    }

    cache.cache_dir = cache_dir;
    info!("Sprite cache directory: {:?}", cache.cache_dir);
}

/// WASM: No cache directory needed
#[cfg(target_arch = "wasm32")]
fn setup_sprite_cache(_cache: ResMut<ShipSpriteCache>) {
    info!("WASM mode: using bundled sprites only");
}

/// Start loading ship sprites (native - with cache and downloads)
#[cfg(not(target_arch = "wasm32"))]
fn start_loading_sprites(mut cache: ResMut<ShipSpriteCache>, mut images: ResMut<Assets<Image>>) {
    // Ensure cache_dir is set (in case setup hasn't run yet)
    if cache.cache_dir.as_os_str().is_empty() {
        cache.cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rebellion")
            .join("sprites");
        if let Err(e) = fs::create_dir_all(&cache.cache_dir) {
            warn!("Failed to create sprite cache dir: {}", e);
        }
    }

    info!("Loading {} ship sprites...", ships_to_load().len());

    let bundled_dir = PathBuf::from(BUNDLED_SHIPS_DIR);
    let mut loaded_bundled = 0;
    let mut loaded_cached = 0;

    for &type_id in ships_to_load() {
        // Priority 1: Check bundled assets (fastest, works offline)
        let bundled_path = bundled_dir.join(format!("{}.png", type_id));
        if bundled_path.exists() {
            match load_image_file(&bundled_path) {
                Ok(image) => {
                    let handle = images.add(image);
                    cache.sprites.insert(type_id, handle);
                    loaded_bundled += 1;
                    continue;
                }
                Err(e) => {
                    warn!("Failed to load bundled sprite {}: {}", type_id, e);
                }
            }
        }

        // Priority 2: Check cache directory
        let cache_path = cache.cache_dir.join(format!("{}.png", type_id));
        if cache_path.exists() {
            match load_image_file(&cache_path) {
                Ok(image) => {
                    let handle = images.add(image);
                    cache.sprites.insert(type_id, handle);
                    loaded_cached += 1;
                    continue;
                }
                Err(e) => {
                    warn!("Failed to load cached sprite {}: {}", type_id, e);
                }
            }
        }

        // Priority 3: Need to download
        cache.loading.push(type_id);
    }

    info!(
        "Loaded {} bundled, {} cached sprites",
        loaded_bundled, loaded_cached
    );

    // If nothing to download, we're ready
    if cache.loading.is_empty() {
        cache.ready = true;
        info!("All sprites loaded!");
    } else {
        info!("Need to download {} sprites", cache.loading.len());
        // Spawn download task
        download_sprites(cache.loading.clone(), cache.cache_dir.clone());
    }
}

/// Start loading ship sprites (WASM - use Bevy AssetServer, not filesystem)
#[cfg(target_arch = "wasm32")]
fn start_loading_sprites(mut cache: ResMut<ShipSpriteCache>, asset_server: Res<AssetServer>) {
    info!(
        "Loading {} ship sprites (WASM mode via AssetServer)...",
        ships_to_load().len()
    );

    for &type_id in ships_to_load() {
        let path = format!("ships/{}.png", type_id);
        let handle: Handle<Image> = asset_server.load(&path);
        cache.sprites.insert(type_id, handle);
        cache.loading.push(type_id);
    }

    info!(
        "Queued {} sprites for loading via AssetServer",
        ships_to_load().len()
    );
}

/// Load an image file (JPEG or PNG) and convert to Bevy Image
fn load_image_file(path: &PathBuf) -> Result<Image, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;

    // Use image crate to auto-detect format and decode
    let img = image::load_from_memory(&bytes)
        .map_err(|e| e.to_string())?
        .into_rgba8();

    // Note: Bundled sprites already have transparent backgrounds
    // Downloaded sprites from CCP's server need background removal (handled separately)

    let (width, height) = img.dimensions();
    let data = img.into_raw();

    Ok(Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

/// Load an image from downloaded bytes (needs background removal) - native only
#[cfg(not(target_arch = "wasm32"))]
fn load_downloaded_image(path: &PathBuf) -> Result<Image, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;

    let mut img = image::load_from_memory(&bytes)
        .map_err(|e| e.to_string())?
        .into_rgba8();

    // Remove black background from CCP server images
    remove_black_background(&mut img);

    let (width, height) = img.dimensions();
    let data = img.into_raw();

    Ok(Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

/// Remove black background from ship sprites and smooth edges - native only
#[cfg(not(target_arch = "wasm32"))]
fn remove_black_background(img: &mut image::RgbaImage) {
    let (width, height) = img.dimensions();

    // First pass: identify background pixels and make them transparent
    // Ship renders have a dark/black background
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            // Calculate brightness (luminance)
            let brightness = 0.299 * r + 0.587 * g + 0.114 * b;

            // If pixel is very dark, make it transparent
            // Use a threshold that catches the black background but keeps ship details
            if brightness < 15.0 {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            } else if brightness < 40.0 {
                // Semi-transparent for edge smoothing
                let alpha = ((brightness - 15.0) / 25.0 * 255.0) as u8;
                img.put_pixel(x, y, image::Rgba([pixel[0], pixel[1], pixel[2], alpha]));
            }
        }
    }
}

/// Download sprites in background (native only)
#[cfg(not(target_arch = "wasm32"))]
fn download_sprites(type_ids: Vec<u32>, cache_dir: PathBuf) {
    std::thread::spawn(move || {
        for type_id in type_ids {
            let url = format!(
                "{}/types/{}/render?size={}",
                IMAGE_SERVER, type_id, RENDER_SIZE
            );
            let cache_path = cache_dir.join(format!("{}.png", type_id));

            info!("Downloading sprite for type {} from {}", type_id, url);

            match download_image(&url, &cache_path) {
                Ok(_) => info!("Downloaded sprite for type {}", type_id),
                Err(e) => warn!("Failed to download sprite for type {}: {}", type_id, e),
            }
        }
    });
}

/// Download a single image (native only)
#[cfg(not(target_arch = "wasm32"))]
fn download_image(
    url: &str,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let bytes = response.bytes()?;
    fs::write(path, &bytes)?;

    Ok(())
}

/// Check if sprites are loaded and transition state (native - handles downloads)
#[cfg(not(target_arch = "wasm32"))]
fn check_sprite_loading(
    mut cache: ResMut<ShipSpriteCache>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut check_timer: Local<f32>,
    mut elapsed: Local<f32>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let dt = time.delta_secs();
    *check_timer += dt;
    *elapsed += dt;

    // Check every 0.5 seconds
    if *check_timer < 0.5 {
        return;
    }
    *check_timer = 0.0;

    if cache.ready {
        next_state.set(GameState::MainMenu);
        return;
    }

    // Check if downloads completed
    let mut all_loaded = true;
    for &type_id in &cache.loading.clone() {
        let cache_path = cache.cache_dir.join(format!("{}.png", type_id));
        if cache_path.exists() && !cache.sprites.contains_key(&type_id) {
            // Load downloaded image (needs background removal)
            match load_downloaded_image(&cache_path) {
                Ok(image) => {
                    let handle = images.add(image);
                    cache.sprites.insert(type_id, handle);
                    info!("Loaded downloaded sprite for type {}", type_id);
                }
                Err(e) => {
                    warn!("Failed to load downloaded sprite {}: {}", type_id, e);
                    all_loaded = false;
                }
            }
        } else if !cache_path.exists() {
            all_loaded = false;
        }
    }

    if all_loaded && !cache.loading.is_empty() {
        cache.loading.clear();
        cache.ready = true;
        info!("All sprites loaded!");
    }

    // Timeout after 10 seconds - proceed anyway
    if *elapsed > 10.0 && !cache.ready {
        warn!("Sprite loading timeout, proceeding without all sprites");
        cache.ready = true;
    }
}

/// Check if sprites are loaded and transition state (WASM - wait for AssetServer)
#[cfg(target_arch = "wasm32")]
fn check_sprite_loading(
    mut cache: ResMut<ShipSpriteCache>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut elapsed: Local<f32>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if cache.ready {
        next_state.set(GameState::MainMenu);
        return;
    }

    *elapsed += time.delta_secs();

    // Don't check too frequently
    if (*elapsed * 4.0) as u32 == ((*elapsed - time.delta_secs()) * 4.0) as u32 {
        return; // Same quarter-second bucket
    }

    // Count loaded vs total
    let mut loaded = 0;
    let mut failed = 0;
    for handle in cache.sprites.values() {
        match asset_server.get_load_state(handle.id()) {
            Some(bevy::asset::LoadState::Loaded) => loaded += 1,
            Some(bevy::asset::LoadState::Failed(_)) => failed += 1,
            _ => {} // Still loading or not tracked
        }
    }

    let total = cache.sprites.len();
    let done = loaded + failed;

    // Transition when all done OR after 10s timeout
    if done >= total || *elapsed > 10.0 {
        info!(
            "Sprites ready: {}/{} loaded, {} failed, {:.1}s elapsed",
            loaded, total, failed, *elapsed
        );
        cache.loading.clear();
        cache.ready = true;
    }
}

/// Helper to get cache dir (native only)
#[cfg(not(target_arch = "wasm32"))]
pub fn get_sprite_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rebellion")
        .join("sprites")
}

/// WASM stub
#[cfg(target_arch = "wasm32")]
pub fn get_sprite_cache_dir() -> PathBuf {
    PathBuf::new()
}

#[cfg(test)]
mod registry_tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_selectable_faction_hull_has_matching_identity_and_transparency() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../assets/ships/ship_manifest.json")).unwrap();
        let entries = manifest["ships"].as_array().unwrap();
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(BUNDLED_SHIPS_DIR);
        for faction in [
            Faction::Minmatar,
            Faction::Amarr,
            Faction::Caldari,
            Faction::Gallente,
        ] {
            for hull in faction.player_ships() {
                let entry = entries
                    .iter()
                    .find(|e| e["type_id"].as_u64() == Some(hull.type_id as u64))
                    .expect("preloaded faction hull");
                assert_eq!(entry["name"].as_str(), Some(hull.name));
                assert_eq!(
                    entry["faction"].as_str(),
                    Some(faction.short_name().to_ascii_lowercase().as_str())
                );
                let bytes = fs::read(dir.join(format!("{}.png", hull.type_id))).unwrap();
                let image = image::load_from_memory(&bytes).unwrap().into_rgba8();
                for (x, y) in [
                    (0, 0),
                    (image.width() - 1, 0),
                    (0, image.height() - 1),
                    (image.width() - 1, image.height() - 1),
                ] {
                    assert_eq!(
                        image.get_pixel(x, y)[3],
                        0,
                        "{} has a visible square corner",
                        hull.name
                    );
                }
                assert!(
                    image.pixels().filter(|p| p[3] > 0).count() > 100,
                    "empty ship art"
                );
            }
        }
    }

    #[test]
    fn every_bundled_hull_is_preloaded_and_decodes() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(BUNDLED_SHIPS_DIR);
        let files: BTreeSet<u32> = fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
            .filter_map(|path| path.file_stem().unwrap().to_str().unwrap().parse().ok())
            .collect();
        let ids: BTreeSet<u32> = ships_to_load().iter().copied().collect();
        assert_eq!(ids.len(), ships_to_load().len(), "duplicate registry IDs");
        assert_eq!(
            files, ids,
            "the loader must cover the complete bundled roster"
        );
        for id in ids {
            load_image_file(&dir.join(format!("{id}.png")))
                .unwrap_or_else(|e| panic!("hull {id}: {e}"));
        }
    }
}
