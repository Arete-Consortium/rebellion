# Rebellion Ship Hull Registry Refactor Plan

## Objective

Guarantee that each EVE hull type ID resolves to one canonical name, faction,
class, sprite, orientation, scale, and hardpoint definition everywhere in
Rebellion.

The manifest owns **visual identity**. Gameplay data owns health, speed,
weapons, score, and encounter behavior. Mixing those concerns recreates the
current duplication problem.

## Current drift points

The current repository repeats hull identity across:

- `src/assets/ship_sprites.rs`
  - Hand-maintained preload list
  - Sprite path convention
- `src/entities/player.rs`
  - Type ID to name/stat mappings
- `src/entities/enemy/spawn.rs`
  - Type ID to name/class/stat mappings
- `src/entities/enemy/faction.rs`
  - Faction color, engine trail, weapon family, tint and rotation mappings
- `src/systems/spawning.rs`
  - Carrier and fighter hull selection
- `src/entities/boss.rs`
- `src/entities/wingman.rs`
- `src/ui/menu/ship_select.rs`
- Campaign-specific Rust and JSON files

This allows a hull to have the correct file but the wrong class, faction,
rotation, menu name, or gameplay assignment.

## Canonical ownership rules

### Manifest owns

- `type_id`
- canonical `name`
- `slug`
- `faction`
- hull `class`
- sprite path and source
- sprite orientation and correction
- visual center
- gameplay/UI/boss display scale
- tint behavior and interpolation
- engine, weapon, and damage hardpoints
- roles and campaign membership
- approval status and locked image hashes

### Gameplay definitions own

- shield, armor and hull values
- movement speed and acceleration
- weapon loadout
- fire rate and damage
- score value
- AI behavior
- boss phases
- unlock requirements

Gameplay definitions reference `ShipTypeId`; they do not redefine the hull's
name, faction, class, or image.

## Phase 0 — Establish the audit baseline

1. Copy:
   - `assets/ships/ship_manifest.schema.json`
   - starter manifest as `assets/ships/ship_manifest.json`
   - `scripts/audit_ship_hulls.py`
2. Install audit dependencies:
   ```bash
   python -m pip install pillow jsonschema
   ```
3. Run:
   ```bash
   python scripts/audit_ship_hulls.py \
     --root . \
     --manifest assets/ships/ship_manifest.json \
     --schema assets/ships/ship_manifest.schema.json \
     --report build/ship-audit.md \
     --json-report build/ship-audit.json \
     --contact-sheet build/ship-contact-sheet.png
   ```
4. Correct missing, duplicate, mislabeled and wrongly oriented sprites.
5. Change a hull to `approved` only after visual review.
6. Run `--print-locks` and store approved SHA-256/dHash values.

### Exit criterion

- Zero missing declared sprites
- Zero orphan numeric sprite files
- Zero exact duplicate images
- Every current code-referenced hull is registered
- Contact sheet has been manually reviewed

## Phase 1 — Add the runtime registry without behavior changes

1. Create `src/assets/ship_registry.rs` from the supplied template.
2. Export the module from `src/assets/mod.rs`.
3. Register `ShipHullRegistryPlugin` before systems that spawn ships.
4. Keep all existing gameplay match tables temporarily.
5. Add tests:
   - registry loads
   - IDs are unique
   - every sprite path follows `assets/ships/{type_id}.png`
   - every `approved` hull has both image hashes

### Exit criterion

The game behaves exactly as before, but the canonical registry is available as
a Bevy resource in native and WASM builds.

## Phase 2 — Replace the sprite preload list

Remove `SHIPS_TO_LOAD` from `src/assets/ship_sprites.rs`.

Use `ShipHullRegistry::iter()` to queue every manifest sprite. Prefer
`AssetServer` for both native and WASM so the same loading path is tested on
every platform.

Retain remote CCP downloads only as a development command, not an automatic
production fallback. A production build should fail visibly when a declared
asset is absent rather than silently showing a different image.

### Exit criterion

Adding one manifest record and one PNG is sufficient to make a hull available
to all runtime systems.

## Phase 3 — Migrate canonical identity lookups

Replace identity match arms in this order:

1. `get_ship_rotation_correction(type_id)`
   - return `registry.require(type_id).sprite.rotation_correction_degrees`
2. `get_enemy_color(type_id)`
   - derive fallback color from manifest faction
3. `get_faction_engine_trail(type_id)`
   - derive engine family from manifest faction, with explicit optional
     presentation override only where canon requires it
4. Player/enemy displayed name
   - read canonical manifest name
5. Ship class and sprite size
   - derive class from manifest
   - derive visual size from `presentation.gameplay_size`

Delete each old match table immediately after its callers migrate. Do not keep
two active sources of truth.

### Exit criterion

A search for canonical hull names and type IDs finds definitions in the
manifest and gameplay rosters, not repeated identity tables.

## Phase 4 — Separate gameplay stats

Create a gameplay record keyed by `ShipTypeId`, for example:

```rust
pub struct EnemyCombatSpec {
    pub hull: ShipTypeId,
    pub health: f32,
    pub speed: f32,
    pub score_value: u64,
    pub weapon: WeaponType,
}
```

`spawn_enemy` should:

1. Resolve `EnemyCombatSpec`
2. Resolve `ShipHullSpec`
3. Spawn with:
   - canonical name/class/faction/sprite from the hull registry
   - health/speed/score/weapon from combat data

Variants can override gameplay values but never hull identity.

### Exit criterion

No combat table returns `(name, class, sprite)` tuples.

## Phase 5 — Migrate menus and campaigns

- Ship select cards receive a `ShipTypeId`
- Menu image, name and faction come from the registry
- Campaign rosters contain IDs or slugs only
- Carrier selection returns `ShipTypeId`
- Boss definitions reference hull IDs and encounter profiles separately
- Wingman definitions reference hull IDs and wingman behavior separately

### Exit criterion

The same hull always produces the same visual asset in gameplay, menus,
briefings, cinematics, bosses, and background traffic.

## Phase 6 — Add CI enforcement

Add this before `cargo test`:

```yaml
- name: Audit ship hull registry
  run: |
    python -m pip install pillow jsonschema
    python scripts/audit_ship_hulls.py \
      --root . \
      --manifest assets/ships/ship_manifest.json \
      --schema assets/ships/ship_manifest.schema.json \
      --report build/ship-audit.md \
      --json-report build/ship-audit.json \
      --strict
```

Upload the Markdown report and contact sheet as workflow artifacts when the
audit fails.

## Mandatory tests

### Data tests

- manifest schema validates
- type IDs, slugs and sprite paths are unique
- approved hashes match
- every gameplay hull ID exists in the registry
- every menu hull ID exists in the registry
- every campaign hull ID exists in the registry
- every numeric PNG has a registry entry

### Visual tests

- PNG decode succeeds
- alpha channel exists
- sprite is not empty
- visible pixels do not touch the canvas edge
- canvas size matches any declared expectation
- no exact duplicate hull images
- perceptual near-duplicates require manual approval

### Runtime tests

- native and WASM load the same set of hull IDs
- missing required hull causes an explicit loading error
- player, enemy, boss, carrier and menu rendering resolve through the registry

## Known high-value checks in the current code

The audit should immediately compare the manifest against:

- `src/entities/enemy/spawn.rs`
  - canonical names
  - broad ship classes
- `src/entities/enemy/faction.rs`
  - faction color and engine mapping
  - rotation corrections
- `src/assets/ship_sprites.rs`
  - preload coverage

A particularly important rule is that class and faction data must come from
the manifest. Hand-maintained grouped ID match arms are extremely easy to
misclassify.

## Definition of done

A hull is complete only when:

1. Its canonical manifest record exists.
2. `assets/ships/{type_id}.png` exists.
3. The PNG passes technical validation.
4. Its contact-sheet image is visually confirmed.
5. Orientation and visual center are correct.
6. Menu and gameplay display the same hull.
7. Approved hashes are locked.
8. CI prevents future drift.

At that point, replacing a sprite is an intentional, reviewable content change
rather than an invisible side effect.
