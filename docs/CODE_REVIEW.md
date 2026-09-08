# Code and dependency review

Reviewed 2026-09-06 against commit `226f3bc` and the current working changes.
This is a source-backed review of the shipped plugin graph, duplicate work,
and dependencies. No build-time, memory, binary-size, or FPS improvement is
claimed without a comparable measurement.

## Assessment

The project has substantial reusable gameplay: a shared simulation pipeline,
four registered chapter modules, faction weapons, bosses, procedural audio,
and deterministic regression infrastructure. The main completion risk is
inconsistent wiring between those systems and the native player-facing route.
This pass found defects that the existing headless suite did not exercise,
including a native startup panic, blocked CG progression and pause resets.

The cleanup and targeted repairs passed the local validation recorded in
[`COMPLETION_PLAN.md`](COMPLETION_PLAN.md). The game is not yet release-ready:
Elder Fleet selection/save defects, inconsistent ship art and an unverified
end-to-end playthrough remain. Detailed gameplay evidence is in
[`GAMEPLAY_REVIEW.md`](GAMEPLAY_REVIEW.md); additional performance defects and
measurement requirements are in [`PERFORMANCE_AUDIT.md`](PERFORMANCE_AUDIT.md).

## Completed cleanup

| Change | Evidence and resulting behavior |
| --- | --- |
| Remove direct Tokio dependency | No application, test, example, or benchmark references Tokio. Sprite downloads use `reqwest::blocking::get` from a `std::thread`. Reqwest still brings its own Tokio requirements transitively; this does not remove all Tokio code. |
| Remove Reqwest JSON feature | The only request reads image bytes and status; no Reqwest JSON API is used. Save/replay JSON serialization keeps its separate Serde dependencies. |
| Remove duplicate effects starfield | `EffectsPlugin` added 150 `Star` sprites while `BackgroundPlugin` independently added 248 configurable `BackgroundStar` sprites. The UI background implementation remains the single starfield owner; the legacy effects file, registrations, exports and cleanup query were removed. |
| Disable state-hash instrumentation during normal play | Native/WASM app construction now disables `SimulationDiagnostics`; headless app construction enables it. Both SimId assignment and state hashing honor the flag. Collision, movement and damage systems remain scheduled. |
| Remove empty collision plugin | `CollisionPlugin::build` did nothing. Its sole registration was inside `SimulationPlugin`, which already registers the real detection/resolution systems. Removed the wrapper and registration. |

The state-hash gate addresses real extra work: `assign_sim_ids` visits every
entity without a `SimId`, including presentation entities, and
`compute_state_hash_system` collects/sorts all `(SimId, Transform)` entries
each fixed tick. The only consumers of `SimStateHash` found in this
repository are tests. The native app does not register `ReplayPlugin`.
Hashing remains available for regression checks and explicit developer use.

Enable instrumentation before the first gameplay tick:

```rust
use rebellion::app_builder::RebellionAppConfig;
use rebellion::simulation::SimulationDiagnostics;

let mut app = RebellionAppConfig::native().build();
app.insert_resource(SimulationDiagnostics { enabled: true });
// Add the application's camera/startup systems, then app.run().
```

When disabled, the last stored hash remains unchanged; it must not be
interpreted as a current snapshot. Enabling mid-session starts assigning
IDs then, so enable before gameplay when comparing deterministic runs.

Sources: [`Cargo.toml`](../Cargo.toml),
[`src/assets/ship_sprites.rs`](../src/assets/ship_sprites.rs),
[`src/systems/effects/mod.rs`](../src/systems/effects/mod.rs),
[`src/ui/backgrounds.rs`](../src/ui/backgrounds.rs),
[`src/simulation/mod.rs`](../src/simulation/mod.rs),
[`src/simulation/sim_id.rs`](../src/simulation/sim_id.rs),
[`src/simulation/state_hash.rs`](../src/simulation/state_hash.rs),
[`src/app_builder.rs`](../src/app_builder.rs).

## Next candidates, ordered by scope and confidence

### Narrow dependency features

`bevy_egui` 0.31.1 enables `manage_clipboard`, `open_url`, `default_fonts`
and `render` by default. The game's only Egui consumer is the painted
capacitor/health wheel in `src/ui/capacitor.rs`; the menus use Bevy UI.
Explicitly selecting `default_fonts` and `render` is a candidate for
removing clipboard/browser-launch support while retaining the HUD. Check
native and WASM builds and the visible wheel before accepting it. The
WASM `web_sys_unstable_apis` configuration currently cites clipboard
support as its reason and should be reevaluated with this change.

Do not remove Egui outright: `UiPlugin` registers `CapacitorWheelPlugin`,
and the game draws real health/capacitor information with it.

### Optional developer overlays

`bevy_gizmos` is used only by
`presentation::debug_overlay::draw_environment_colliders`. Its two flags
default to false, and no writer/toggle was found in the shipped source.
A developer-overlay feature can retain this useful inspection code while
omitting its engine feature and registration from regular builds. Validate
both feature configurations; do not leave a registered `Gizmos` system
after disabling the dependency feature.

### Optional 3D model path

`ShipModelsPlugin` queues 11 GLB scenes during loading. Player/enemy ships
use sprites, but `spawn_wingman` and `spawn_boss` still prefer cached
`SceneRoot` handles. The entry point creates only `Camera2d`; no `Camera3d`
or lighting setup was found. This is an unfinished rendering path rather
than proven dead content.

Consider a `ship-models` feature that keeps source assets and model support
available while using existing sprite fallbacks by default. Gate model
loading, scene branches and Bevy GLTF/scene features together. Verify all
boss and wingman visuals before claiming the default is equivalent. Do not
remove the model cache alone: callers currently use the presence of its
handle as the decision to skip sprite creation, without checking load
success.

### Unused compatibility code and misleading scaffolding

- `get_sprite_cache_dir`, legacy `spawn_random_powerup`, and public
  `spawn_explosion` have no callers in the checked source/tests. The latter
  bypasses event-based particle budgeting. Remove only after confirming
  these exported helpers are not a supported external API.
- `despawn_trig_boss_intro` is an empty registered function; the same exit
  hook also invokes the real `despawn_trig_boss_intro_ui`. The empty stub
  can be removed without removing the Triglavian chapter or its UI.
- `SoundAssets.menu_confirm` is generated at startup but has no consumer.
  Either connect it to actual confirmation feedback or remove its unused
  generation/handle; other menu and powerup sounds do have consumers.
- Cargo's `headless = []` feature has no feature-conditional source and
  does not avoid renderer/audio dependency compilation. The actual test
  behavior is selected by `RuntimeMode::Headless`. A truly smaller headless
  compile target requires a separate feature design because simulation
  code still directly references sprites and effect helpers.
- `src/assets/ship_registry.rs.template` is not compiled. It is referenced
  by the registry migration package, so treat it as pending design work,
  not runtime code or an implemented authoritative registry.

### Allocation claim requiring correction

The capacitor wheel's `POINT_BUF` comment claims zero steady-state vector
allocations. Each draw passes `std::mem::take(&mut *points)` into an Egui
shape; ownership and capacity move to the shape, leaving an empty vector
for the next draw. The comment is therefore not supported by the code.
Remove the misleading scratch-buffer claim or redesign around batched
geometry/cache ownership, then measure allocations. Replacing the thread
local with a local vector alone is a simplification, not an allocation
optimization.

## Preserve intended game systems

All four registered chapter modules have their own entry/progression
handlers and integration coverage. The generic campaign remains a tested
fallback for modules without custom handlers. These implementations are
not safe deletion candidates merely because the public slice covers three
Caldari-Gallente missions.

Scoring and `scoring_v2` are also not duplicate plugins: `ScoringPlugin`
owns `ScoreSystem`, `SaltMinerSystem`, and the shared `ComboHeatSystem`
update. Consolidating files would not by itself remove duplicate work.

Damage/death handling has a real consolidation opportunity: discrete
projectiles use the simulation resolution pipeline, while the continuous
disintegrator beam directly changes enemy health and emits death/explosion
events in `entities/projectile.rs`. This affects gameplay semantics and
must be migrated with weapon/death/scoring tests, not deleted as bloat.

## Materials review

### P1 — Preloaded ship art does not share a usable visual standard

Directly inspected the four images below and the retained v4 contact sheet.
These type IDs all appear in `SHIPS_TO_LOAD` in
[`src/assets/ship_sprites.rs`](../src/assets/ship_sprites.rs). Bundled files
are decoded by `load_image_file` without background removal; its comment
assumes they are already transparent. The separate downloaded-image path
does perform background processing, so it does not repair bundled art.

| Runtime file | Observed source image | Release work needed |
| --- | --- | --- |
| [583.png](../assets/ships/583.png) | Oblique dark ship over a blue nebula background | Replace or prepare a consistent cutout; verify silhouette at gameplay scale |
| [608.png](../assets/ships/608.png) | Dark oblique ship with a visible brown/green space background | Resolve background, view angle and edge readability |
| [587.png](../assets/ships/587.png) | Clean rendered cutout, with its nose pointing down in the image | Verify loader/spawn rotation against the chosen forward direction |
| [17713.png](../assets/ships/17713.png) | Flat purple cartoon ship with bright engine circles | Resolve the style mismatch and validate intended hull identity |

These observations establish inconsistent source materials; a native
combat capture is still needed to assess their final scale, rotation and
readability. This pass did not validate every hull's identity or release
provenance. The live manifest remains unchanged: 72 `pending_review`,
two `missing`, zero approved.

The file named
[`ships_audit_v4_contact_sheet.png`](ship-review/contact-sheets/ships_audit_v4_contact_sheet.png)
still displays a v3 title. Its candidates also mix oblique/side views and
background remnants. Do not promote an iteration based on its version name.
Approve each hull against one standard: identity, transparent crop, forward
direction, pivot, gameplay size, weapon hardpoints and visible hitbox.

### P2 — Optional audio overrides are absent and disagree with enabled formats

Native startup requested five missing `audio/music/*.ogg` files and logged
asset errors while successfully generating procedural music and sound
effects. [`src/systems/music.rs`](../src/systems/music.rs),
`load_file_music`, unconditionally requests those paths. Its documentation
also promises OGG/MP3 support, while the manifest enables Bevy's `wav`
decoder. Make override discovery/configuration agree with the actual files
and enabled decoders, then test both absent and supplied overrides. The
procedural tracks are active content and are not a deletion candidate.

### Completed storage cleanup

Runtime `assets/` now contains 13,704,976 bytes, down from 54,516,725 bytes.
The review archive reconstructs all 588 original files from 441 unique
stored files, verified against the original Git blobs. Exact duplicate
and generated report removal eliminated 8,185,279 bytes of content before
the small new archive index, documentation and recovery tool. All 11 editable
STLs remain under [`tools/ship-models/`](../tools/ship-models/); all runtime
GLBs remain byte-identical. Git history was not rewritten.

See the [archive manifest and recovery instructions](ship-review/README.md)
for exact counts and preservation evidence. Source relocation reduces what
the existing asset-copy packaging commands include; it is not a measured
compressed download, executable-size or repository-clone reduction.

## Validation and review boundaries

`tests/simulation_diagnostics.rs` exercises real headless scheduling with
instrumentation disabled, enabled, disabled again and reenabled, asserting
SimId assignment and hash updates. Existing golden-hash tests retain their
headless default behavior. Full verification of the current working tree
is recorded in [`COMPLETION_PLAN.md`](COMPLETION_PLAN.md).

No tracked WASM binaries or generated browser glue were found by the
bounded tracked-file inventory. Source assets, current runtime assets, and
reproducible review outputs now have separate documented locations.

Follow the rendered timing and visual checks in
[`PERFORMANCE_AUDIT.md`](PERFORMANCE_AUDIT.md) after rebuilding. Headless
tests cannot establish starfield appearance, ship visibility, controller
behavior, or native/mobile rendering performance.
