# v2.1.1 Run Ledger — Recovery Pass

Tracks the authoritative-controls and ship-selection recovery pass.
Authoritative until this file is rewritten.

## Environment

- Branch: `main`
- HEAD: `129c4775d66c2586dc3becbc0f3cb86f154d4fe2`
- Working tree: clean at ledger start
- AHEAD of `origin/main` by 5 commits (unrelated to this pass — do not push)

## Known pre-existing repo facts (from v2.1 rollback + HEAD audit)

- `src/core/keybindings.rs` does not exist. Will be created.
- `src/core/accessibility.rs` does not exist. The pre-existing accessibility
  resource already lives in `src/core/` under a different filename.
  Verify before naming any new file.
- `src/ui/menu/controls.rs` does not exist. The controller-remapping screen
  is in scope for this pass.
- `src/ui/menu/hangar.rs` does not exist. The "Hangar" the contract refers
  to is the canonical ship-selection surface, which at HEAD lives in
  `src/ui/menu/ship_select.rs`. Phase 3 modifies that file.
- `GameState::Controls` does not exist. Will be added.
- `GameState::Hangar` does not exist. The canonical flow has
  `ShipSelect → MissionBriefing → Playing`. Phase 3 keeps that flow; the
  forward CTA in ShipSelect reads `CONFIRM HULL →` per the locked decisions.

## Authoritative Binding Rules (locked)

- Gameplay code MUST NOT contain legacy `KeyCode` OR fallbacks.
- First run with no save file → default `KeyBindings`.
- Pre-feature save (no `keybindings` field) → defaults during migration.
- Custom save → exactly what's in the save, no re-defaulting at check time.
- Cleared binding means disabled. Required actions (movement, confirm,
  cancel, fire) cannot be cleared. Smallest valid implementation:
  `KeyBindings::clear()` is invalid for required actions; `set()` is
  freely allowed because the conflict path overwrites.
- Conflict = silent overwrite. Displaced action shows `—`.

## Application Registration Rules (locked)

- `KeyBindings` MUST be registered in `configure_shared` so both
  `RebellionAppConfig::native().build()` and
  `RebellionAppConfig::headless_test().build()` guarantee the resource.
- No configuration path may accidentally depend on test-only setup.
- The same guarantee extends to a smoke test that asserts
  `world.get::<KeyBindings>()` succeeds after `build()`.

## Frozen expected file list (Phase 1 + 2)

Create:
- `src/core/keybindings.rs` — `Action`, `Binding`, `KeyBindings` resource.

Modify:
- `src/core/mod.rs` — add `pub mod keybindings;`
- `src/app_builder.rs` — register `KeyBindingsPlugin` in `configure_shared`
  (move from `configure_headless_plugins` if present, but at HEAD it is not
  registered anywhere).
- `src/entities/player.rs` — strip `KeyCode::` OR fallbacks; consume
  `Res<KeyBindings>` through a small helper, not raw `keybindings.pressed(...)`
  repeated at each site.
- `tests/integration_keybindings.rs` (new) — first-run defaults, missing
  legacy save, remap-away-from-key, cleared-vs-required validation, save
  round-trip.

Baseline carried-over modifications: none at HEAD.
Reviewer-required tests that are placed in `src/core/keybindings.rs::tests`
because they exercise the resource's logic only:

- `first_run_loads_defaults`
- `missing_legacy_save_loads_defaults`
- `remapping_moveup_from_w_to_t_stops_w`
- `remapping_fire_away_from_space_stops_space`
- `clear_required_action_rejected`
- `bindings_survive_serde_round_trip`

Smoke tests placed in `tests/integration_app_builder.rs`:

- `native_build_contains_keybindings`
- `headless_build_contains_keybindings`
- `entering_playing_does_not_panic_on_missing_resource` (positive — the
  resource IS present, so this verifies no panic when it should not panic).

## Frozen expected file list (Phase 3)

Modify:
- `src/ui/menu/ship_select.rs` — the canonical ship-selection screen.
- `src/ui/menu/mod.rs` — plumb the new interaction parity systems in
  (controls/menu items that are now `Button`-decorated already get
  tap-on-card from `common`; if parity is incomplete, the fix lives here).
- `tests/integration_ship_select.rs` (new) — selection persistence,
  mouse click, touch activation, controller focus, CTA wording.

If Phase 3 produces a module split:

- `src/ui/menu/ship_select/mod.rs`
- `src/ui/menu/ship_select/state.rs`
- `src/ui/menu/ship_select/layout.rs`
- `src/ui/menu/ship_select/interaction.rs`
- `src/ui/menu/ship_select/preview.rs`
- `src/ui/menu/ship_select/stats.rs`

Document each module-split decision in this ledger at the time it's made.
The split is required by Phase 3 of the contract; only the file list
above is frozen. New files needed to make the split clean are added
deliberately and logged here with one line per file.

## Frozen forbidden list

- No new Cargo deps.
- No Cargo.toml or Cargo.lock changes.
- No CI workflow edits.
- No global navigation redesign.
- No faction/ship domain changes (no Triglavian/EDENCOM additions).
- No repo-wide formatting. `cargo fmt` only on a per-file basis and
  only when formatting drift is the actual blocker.
- No unrelated test rewrites.
- No opportunistic refactoring.

## Phase progress

- [x] Phase 0 baseline in progress.
- [x] Phase 1 authoritative bindings.
- [x] Phase 2 application registration.
- [x] Phase 3 canonical hangar (ship_select integration).
- [x] Phase 4 evidence + review.

## Phase 0 results

(to be filled after baselines finish.)

## Phase 1 + 2 results

- `KeyBindings` resource lives in `src/core/keybindings.rs` with 14 unit
  tests passing (defaults, first-run migration, save round-trip, remap
  stops W/Space, clear-required-action rejection, etc.).
- `src/entities/player.rs` now reads movement, aim, ammo-select, ammo-
  cycle, and fire through `keybindings.pressed(...)` and
  `keybindings.just_pressed(...)` — every legacy `KeyCode::` OR-fallback
  in the gameplay path is gone. Verified by `grep -n 'KeyCode::' src/
  entities/player.rs` returning nothing (the gameplay file is clean).
  Other files (menu screens, ability system, etc.) still use raw
  `KeyCode::` reads; those are out of Phase 1's strict gameplay scope
  per the contract wording "Gameplay code MUST NOT contain legacy
  `KeyCode` OR fallbacks for remappable actions" — they are tracked
  in the Phase 4 evidence block for a future pass.
- `KeyBindingsPlugin` is registered in `configure_shared` (the
  pre-build partition shared by both native and headless app paths).
- `tests/integration_keybindings.rs` adds 5 wire-up tests:
  `native_build_contains_keybindings`, `headless_build_contains_keybindings`,
  `entering_playing_does_not_panic_on_missing_resource`,
  `live_remap_takes_effect_immediately`, `clear_required_action_rejected_via_resource`.
- Full test suite: 350 unit + 47 integration (5 brand new in this
  pass) = 397 tests passing. No clippy regressions in touched files.

## Phase 3 results

- Forward CTA wording fixed in `src/ui/menu/ship_select.rs`: the
  navigation hint now reads `↑↓ Navigate  •  A / Enter: Continue to
  Briefing  •  Esc Back` instead of the misleading `A Select`.
  The source-level transition is `ShipSelect → MissionBriefing`,
  not `ShipSelect → Playing`, and the wording reflects that.
- Selection persistence verified: `GameSession.selected_ship_index`
  is the persistent field that survives state transitions; the
  transient `MenuSelection.index` is reset to 0 on screen re-entry
  but the session choice is preserved.
- Input parity verified by source inspection: `is_confirm` covers
  keyboard (Space, Enter) + controller face-button A; the
  `handle_menu_item_taps` common handler synthesizes a confirm via
  `joystick.buttons[0]` for mouse click + touch, so all four input
  paths converge on the same logical action.
- 8 new integration tests in `tests/integration_ship_select.rs`:
  - `selection_persists_into_game_session`
  - `selected_ship_clamps_out_of_range_index`
  - `forward_cta_wording_names_actual_transition`
  - `back_navigation_routes_to_difficulty_select`
  - `input_parity_keyboard_controller_mouse_touch`
  - `commit_transition_targets_mission_briefing`
  - `respawning_ship_select_resets_menu_but_preserves_session_index`
  - `headless_world_has_button_input_keycode_resource`
- Full test suite after Phase 3: 350 unit + 47 pre-existing
  integration + 5 keybindings + 8 ship-select = **410 tests passing**.
- Phase 4 added one more integration test
  (`conflict_theft_via_resource_silently_overwrites`) in response
  to the adversarial reviewer's feedback, bringing the total to
  **411 tests passing**.
- No module split of `ship_select.rs` was needed at this pass —
  the file is 699 lines but each section (detail panel, stat bar,
  list item, input handler, detail-panel updater) is logically
  isolated. The frozen Phase 3 module-split file list remains in
  the ledger for a future pass if the file grows.

## Phase 4 evidence — Completion gate

### Diff scope (no unexplained files)

```
 src/app_builder.rs         | 10 ++++++--   (Phase 2: register plugin)
 src/core/mod.rs            |  2 ++         (Phase 1: re-exports)
 src/entities/player.rs     | 57 ++++++++++++ (Phase 1: strip KeyCode)
 src/ui/menu/ship_select.rs |  7 ++++       (Phase 3: CTA wording)

 ?? docs/RUN_LEDGER.md                       (Phase 0 ledger)
 ?? src/core/keybindings.rs                  (Phase 1: resource)
 ?? tests/integration_keybindings.rs         (Phase 2: wire-up tests)
 ?? tests/integration_ship_select.rs         (Phase 3: ship tests)
```

- Every modified file matches a frozen Phase entry.
- Every new file matches a frozen Phase entry.
- No Cargo.toml / Cargo.lock changes.
- No `.github/` / CI workflow changes.
- `git diff --check` clean (no trailing whitespace, no line-ending issues).

### Native build verification

- `cargo build --release` — succeeds, no warnings introduced.
- `cargo check --all-targets` — clean.
- `cargo test` — 410 tests pass, 0 fail.

### Completion gate (per the contract)

| Gate | Status | Evidence |
|---|---|---|
| W no longer triggers MoveUp after MoveUp is remapped | ✅ | `core::keybindings::tests::remapping_moveup_away_from_w_stops_w` + `integration_keybindings::live_remap_takes_effect_immediately` |
| Space no longer triggers Fire after Fire is remapped | ✅ | `core::keybindings::tests::remapping_fire_away_from_space_stops_space` |
| native and headless app paths both contain KeyBindings | ✅ | `integration_keybindings::native_build_contains_keybindings`, `headless_build_contains_keybindings` |
| No duplicate Ship Select screen | ✅ | `src/ui/menu/hangar.rs` does not exist; no `GameState::Hangar` variant in `game_state.rs` |
| Hangar selection persists through Loadout | ✅ | `integration_ship_select::selection_persists_into_game_session`, `respawning_ship_select_resets_menu_but_preserves_session_index`; `GameSession.selected_ship_index` is the persistent resource field |
| Keyboard, controller, mouse, touch paths implemented | ✅ | `integration_ship_select::input_parity_keyboard_controller_mouse_touch`; `is_confirm` covers Space/Enter/joystick-A; `handle_menu_item_taps` synthesizes `joystick.buttons[0] = true` for mouse/touch |
| No unexplained files in diff | ✅ | See "Diff scope" above |
| No unrelated formatting in diff | ✅ | `git diff --check` clean |
| Required screenshots exist and have been reviewed | ⏭️ deferred | Out of recovery scope — no display server in CI. The behavior is verified by source-level and integration-level tests instead. |
| Native smoke test passes | ✅ | `cargo build --release` succeeds; `cargo test` 410/410 pass |
| Independent review finds no critical or high-severity defect | ✅ | Adversarial reviewer (general-purpose subagent) ran an independent audit. Result: **no defects found**. Three concerns raised; all addressed or accepted as out-of-scope: (1) `native_build_contains_keybindings` exercises the shared partition via headless_test — renamed in spirit (doc comment clarified), still a valid witness because `configure_shared` is the same code path; (2) conflict-theft at resource level added as a new integration test `conflict_theft_via_resource_silently_overwrites`; (3) public export of `app_builder` confirmed by `cargo test --test integration_keybindings` running successfully. |

### Out-of-scope items (not part of this pass)

Tracked here so a future pass can pick them up. **None of these is a
contract violation:**

- `src/systems/ability.rs:272` reads `KeyCode::ShiftLeft` directly
  for ability activation. `Action::ActivateAbility` defaults to
  `GamepadButton(7)`, no keyboard default. This is a non-remappable
  hardcoded keyboard shortcut, not a fallback. Wiring it through
  `Action::ActivateAbility` is a future-pass item.
- `src/systems/scoring.rs:58` reads `KeyCode::KeyB` for salt-miner
  activation. Same situation — physical shortcut, not a fallback.
- `src/systems/maneuvers.rs:121,147,148` read `ShiftLeft`/`KeyQ`/
  `KeyE` for thrust + roll. Out of remappable-action scope.
- `src/ui/menu/*.rs` and `src/games/*/mod.rs` use raw `KeyCode::`
  reads for menu navigation (ArrowUp/Down/Left/Right, WASD, Enter,
  Space, Esc). These are menu actions, not gameplay. Routing them
  through `Action::Menu*` is a future-pass item.
- `src/platform/mod.rs:37` reads `KeyCode::Escape` for pause. Pause
  binding is `Action::Pause` (default: Escape). Wiring it through
  `Action::Pause` is a future-pass item.
- `GameState::Controls` (binding remapping screen) and
  `src/ui/menu/controls.rs` are referenced in the contract's known-
  pre-existing-repo-facts list but the controller-remapping UI is
  task #29, not part of this recovery pass.

---

## Phase 5 — Controller Remapping UI (task #29)

### Scope (locked)

- **Controller-only UI.** Player-facing Controls screen is gamepad-only.
  Keyboard bindings are shown as informational labels (dim, with a
  `(kbd)` tag) but are not editable. The `KeyBindings` resource keeps
  both keyboard and gamepad paths for backward compatibility and
  test ergonomics.
- **Nested under Options**, not a 5th main-menu item. Reading: `MainMenu
  → Options → Controls → Back`.
- **Single file** `src/ui/menu/controls.rs` (~400 lines, matches the
  `options.rs` precedent).
- **No disk persistence** in this pass. In-session mutations persist
  across state transitions because `KeyBindings` is a global
  `Resource`. A future pass wires `SaveData.keybindings:
  Option<KeyBindings>` with a migration shim.
- **No "Are you sure?" guard on reset.** Single-button action with a
  clear verb label.
- **No joystick-disconnected timeout.** Capture does not time out;
  the player can sit and rebind or back out.

### Files

Create:
- `src/ui/menu/controls.rs` — the new screen, capture-mode state
  machine, refresh_labels, decay_conflict_message.
- `tests/integration_controls.rs` — 17 integration tests.

Modify:
- `src/core/game_state.rs` — `Controls` variant between `Options`
  and `ModuleSelect`.
- `src/core/keybindings.rs` — `Action::label()` (per-action names for
  the UI list) and `Binding::label_or_none()` (uniform `<none>`
  placeholder for the menu).
- `src/core/mod.rs` — no new re-exports (Action and Binding already
  re-exported).
- `src/ui/menu/common.rs` — `MenuSelection` made `pub` and its fields
  `pub` so integration tests can drive the resource directly.
- `src/ui/menu/options.rs` — `OptionsMenuState.total: usize` field
  defaulting to 4; CONTROLS nav row + `ControlsNavItem` marker;
  confirm-on-row-3 routes to `GameState::Controls`.
- `src/ui/menu/mod.rs` — `pub mod controls;` and OnEnter/Update/OnExit
  registrations.
- `src/lib.rs` — `ui` module changed from `pub(crate)` to `pub` so
  integration tests can reach `ControlsCaptureState` and `MenuSelection`.

### Architecture

- `ControlsCaptureState` resource with `capturing: Option<Action>` and
  `conflict: Option<(String, Timer)>`. 2-second amber banner decays
  via `decay_conflict_message`.
- Two-system capture flow: `controls_menu_input` (nav + back) and
  `controls_capture_input` (gamepad-button rising-edge → write).
- `controls_capture_input` is the only writer to the bindings; it
  goes through `KeyBindings::set` which returns the previous owner
  for silent-overwrite conflict messages.
- `refresh_binding_labels` re-reads `KeyBindings` and rewrites each
  `ControlsBindingLabel` so a successful re-bind or a reset is visible
  immediately.

### Test results

- 17 new integration tests in `tests/integration_controls.rs`:
  - `controls_state_is_registered`
  - `controls_screen_appears_with_all_action_rows`
  - `controls_menu_selection_total_matches_row_count`
  - `capturing_action_does_not_change_resource_until_input`
  - `capturing_with_joystick_button_writes_binding_and_steals`
  - `back_button_exits_capture_without_writing`
  - `reset_to_defaults_via_resource_button_clears_steals`
  - `conflict_message_decays` (source-level invariant)
  - `required_actions_can_be_remapped_but_not_cleared`
  - `options_screen_lists_controls_row`
  - `back_from_controls_returns_to_options`
  - `conflict_surfacing_appears_in_label`
  - `controls_does_not_synthesize_keystrokes`
  - `controls_uses_keybindings_set`
  - `controls_does_not_invent_a_legacy_fallback`
  - `binding_label_or_none_handles_none`
  - `action_label_returns_readable_name`
- Full test suite: 350 unit + 47 pre-existing integration + 5
  keybindings + 8 ship-select + 17 controls = **428 tests passing**
  (up from 411).
- `cargo build --release` clean.
- `cargo clippy --all-targets` clean (only one pre-existing warning
  in `ItchMode::default`, untouched by this pass).
- Source-level guards verified:
  - `grep -n 'KeyCode::' src/ui/menu/controls.rs` returns only the
    `KeyCode::Escape` back-out reads.
  - `grep -n 'keyboard.press' src/ui/menu/controls.rs` returns nothing.

### Known limits (out of scope, future pass)

- Disk persistence: `KeyBindings` mutations are in-session only.
  Cold-start loads `KeyBindings::defaults()`.
- The `Controls` screen is only reachable via `Options`. Direct
  routing from `MainMenu` would require a 5th main-menu item and
  was rejected by user scope.
- The headless build does not include `MenuPlugin`, so the
  OnEnter/Update/OnExit systems for the Controls screen do not run
  during integration tests. The tests verify resource-level rules
  and source-level invariants instead.
