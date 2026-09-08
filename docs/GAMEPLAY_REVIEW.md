# Gameplay review and remaining fixes

Reviewed: 2026-09-06. Scope: native menu input, CG mission progression,
pause/resume, and Elder Fleet selection/progression wiring in the current
standalone Rebellion checkout. Findings below distinguish source-confirmed
defects from validation still awaiting execution.

## Changes made in this pass

- CG waves now spawn when their inter-wave delay expires. Warnings appear
  once and use the same enemy count as the upcoming wave.
- CG boss completion reads the simulation destruction event after the boss
  entity is removed, allowing the mission to reach Stage Complete.
- Pause/resume preserves the prior gameplay state and avoids mission-entry
  resets; the corresponding lifecycle changes are covered separately.
- CG restart clears both player and enemy projectiles. Returning to the main
  menu clears CG combat entities, while pause/resume preserves them. Bosses
  carrying both boss and enemy markers are queued for cleanup only once.
- Options queries are disjoint, preventing the native startup query panic.
  Slider fills have an explicit marker because Bevy 0.15 automatically adds
  `BorderColor` to every UI `Node`.
- Options uses vertical input to select rows and horizontal input to adjust
  the selected slider, including left-stick X. Vertical navigation takes
  precedence when both axes are pressed.
- Controls capture ignores buttons held when capture begins until they are
  released. A distinct new button can be captured immediately; A can be
  selected deliberately by releasing and pressing it again.

## Open findings

### P2 — Several advertised remappings do not affect runtime input

**Evidence:** `src/core/keybindings.rs:183-190` defines Pause, Confirm,
Cancel and Menu direction bindings; `src/ui/menu/controls.rs:141-147` exposes
the complete action list. Their production handlers still read physical
inputs directly:

- `src/platform/mod.rs:31-38`: pause reads Escape or controller Start.
- `src/ui/menu/common.rs:134-170`: menu navigation reads fixed keys,
  D-pad and stick directions.
- `src/ui/menu/common.rs:205-209`: confirm reads Space, Enter or controller A.
- `src/ui/menu/pause.rs:459-461`: quick resume reads Escape or Start.

A source search for these `Action` variants finds definitions, defaults and
tests, but no runtime binding lookups for them. Editing their rows changes
the displayed and persisted table without changing those controls.

**Impact:** Players cannot rely on the Controls screen to customize menu
navigation or pause. This is independent of the activation/capture fix.

**Next fix:** Route the offered menu and pause actions through `KeyBindings`
at their actual consumers, and define how analogue navigation and a recovery
input coexist with remapped discrete buttons. Do not keep an apparently
editable row for a binding that the runtime ignores.

**Required regression:** Drive the actual menu and pause handlers after a
rebind, verifying that the new button performs the action and the displaced
button no longer does. Exercise cancel, reset, and persisted reload as part
of that flow. Resource-only assertions do not establish input behavior.

### P1 — Elder Fleet stage selection targets the wrong campaign state

**Evidence:** The Elder Fleet campaign route enters Stage Select at
`src/ui/menu/faction_select.rs:374-379`. Selection writes the generic
`CampaignState.act` and `mission_index` at
`src/ui/menu/stage_select.rs:450-480`. Actual Elder Fleet mission startup
reads `ElderFleetCampaignState.current_mission` at
`src/games/elder_fleet/ef_campaign.rs:315-321`; it never reads the selected
generic mission. The selector uses a 13-stage, 4/5/4 act mapping, while
`ElderFleetCampaignState::total_missions` declares nine missions at
`src/games/elder_fleet/ef_campaign.rs:55-57`.

**Impact:** Selecting an unlocked stage does not select the corresponding
Elder Fleet mission. Re-entering the campaign can retain its earlier
module-specific mission, including a completed campaign index, while the
selector and briefing show generic campaign content.

**Next fix:** Make the Elder Fleet mission list and state authoritative for
its selector, briefing, retry and new-run entry. Map stage numbers to its
nine mission definitions and reset or preserve its mission index explicitly
for each entry path.

**Required regression:** Seed an unlocked third mission, select it through
the native handler, start play, and assert the module actually starts index
2 with that mission's content. Then quit, select the first mission, and
verify index 0. Repeat after completing the ninth mission.

### P1 — Elder Fleet completion does not persist selector unlocks

**Evidence:** `src/core/save.rs:293-315` owns the stage-completion update.
Its only non-test caller is `src/systems/campaign.rs:551-558`, whose generic
boss-completion system is disabled for Elder Fleet at
`src/systems/campaign.rs:48-60`. The active Elder Fleet completion handler
at `src/games/elder_fleet/ef_campaign.rs:706-749` advances its in-memory
index and changes screens without writing stage progress. Stage Select
still enforces persisted unlocks at `src/ui/menu/stage_select.rs:450-455`.

**Impact:** Completing missions does not unlock their continuation in the
selector through the existing save API. This also prevents a meaningful
stage-selection regression unless its fixture seeds an unlock explicitly.

**Next fix:** Persist the completed Elder Fleet mission through the same
authoritative completion path that advances its campaign. Keep completion
idempotent and use its own mission numbering.

**Required regression:** Complete a mission through the active module's
boss-resolution path, serialize and reload the save, and assert the next
mission becomes selectable. Reprocessing the same completion must not
advance or reward it twice.

## Validation boundary

The final `cargo test --offline --locked` run passed all 506 tests.
`tests/cg_campaign_progression.rs` exercises actual chained wave systems and
projectile collision/damage/death systems instead of only mutating campaign
resources. The current Options and Controls unit tests exercise native
input handlers, including same-update activation followed by capture.

`app_builder::tests::native_game_system_queries_initialize_without_a_renderer`
adds native content, presentation, diagnostics, analytics, achievement and
module plugins to the headless application, then initializes their Update
and FixedUpdate schedules. It catches query-access conflicts even when the
corresponding menu or module is inactive. It does not run rendering, audio,
asset loading or platform input.

The nine pause tests include real projectile preservation through resume,
removal on explicit restart, and combat/HUD cleanup when quitting. Strict
Clippy and formatting checks also passed. A final native startup check
initialized Metal, loaded 48 bundled sprites and remained running for 20
seconds without a panic. Missing optional music overrides still produced
asset errors; procedural audio initialized.

An end-to-end campaign playthrough and physical controller check remain
required. Schedule initialization and native startup do not prove
finishability. See `COMPLETION_PLAN.md` for the commands and validation limits.
