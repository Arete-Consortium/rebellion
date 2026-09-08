# Mac playtest checkpoint

September 7 checkpoint for the free portfolio/community project. For the newer save/audio build, see [September 8 follow-up](PLAYTEST_2026-09-08.md).

## Player route

**Play → Chapter → Faction → Difficulty → Hull → Mission briefing → Launch.**

- **Minmatar / Amarr:** play either side of the existing nine-mission Elder Fleet conflict.
- **Caldari / Gallente:** play either side of the three-mission campaign slice.
- Each faction has three initially available frigates and its own additional unlockable hull definitions. The two chapter selectors share one consistent input path.

The opening screen now uses a quiet orbital backdrop without a baked-in title behind live menu text. Chapter, faction, difficulty and briefing prompts show keyboard bindings. Main menu, hull preview, briefing and result actions use the common pointer/controller input path.

## Ship and battle presentation

79 old PNGs were replaced with the owner's transparent top-down source renders. The current 80-entry registry passes the official name/ID check. Selectable advanced hull IDs and the two chapters' enemy/boss mappings were repaired. See [identity review](ship-review/IDENTITY_REVIEW.md) and [pinned source import](ship-review/source-import.json).

The Elder Fleet boss renderer now assigns its loaded texture to `Sprite.image`, fixing a direct cause of solid square ships. Enemy banking preserves hull rotation; player/wingman/boss presentation and engine direction use the orientation corrections. The runtime's preload and facing information both come from the bundled manifest.

Carriers now arrive for scheduled mission waves. A two-second warp animation stretches and resolves the carrier silhouette before deployment. Its 420-pixel hull stays behind the action at subdued opacity, then fades out. Carrier animation cannot block a wave if its texture is unavailable. The independent extra-fighter spawner was removed, and legacy waves are gated away from both chapter modules. Distant faction hulls and weapon exchanges provide battle traffic without adding collision or score entities.

The HUD reads the actual selected chapter's mission and wave state. Menu/result transitions clear combat actors, projectiles, HUD and background while pause and boss transitions retain the live scene.

## Commander, controller and sound follow-up

Opening squad commanders now use actual faction destroyers: Coercer against Minmatar, Thrasher against Amarr, Cormorant against Gallente and Catalyst against Caldari. The Elder Fleet commander previously lacked the `Enemy` marker required by normal projectile collision/damage, while the legacy boss damage system did not run in its `BossFight` state. Commanders now participate in the normal pipeline. Their phase logic and the shared boss HUD read the same authoritative health; the HUD also covers Caldari–Gallente bosses.

Xbox RT now accepts both axis and analog-button reporting, and fires along the current aim. Gamepad polling follows Bevy input processing; disconnect clears held buttons and axes. Short horizontal keyboard taps and faction-card contrast were also repaired after native inspection. See [controller controls and physical qualification](CONTROLLER_PLAYTEST.md).

The first combat audio pass supplies distinct original synthesized autocannon, laser, railgun, missile and drone effects, layered explosions and a carrier warp cue tied to each wave. Weapon sound playback is bounded to eight active voices and two new voices per frame. Cues are generated once and reused. [Audio direction and audition export](../assets/audio/README.md) records the scope; shield/armor/hull impacts, alerts, abilities and UI sounds still need the next listening pass. No EVE recordings were added.

## Automated evidence and limits

The route regression drives the actual headless gameplay, module, menu and transition schedules. It selects the chapter/faction/hull, enters combat, clears enemies through the production death/event path, visits all three CG result screens and both boss fights, returns to the main menu and starts again at mission zero. A separate route check covers all four faction choices and verifies the deployed enemies belong to the opposing faction.

Route clearance is automated by setting enemy health to zero. A separate commander regression fires real projectiles, verifies nonlethal health and HUD depletion, then verifies lethal damage and next-mission progression for both Elder Fleet sides. These checks do not prove human completion, difficulty, sustained rendered frame rate or final art direction. Physical-controller testing and a full Elder Fleet completion run remain separate qualification work.

## Candidate packaging

The updated candidate is `dist/playtest-20260907/Rebellion.app` for Apple Silicon, with a clean `Rebellion.app.zip` beside it. The earlier `dist/Rebellion.app` is preserved so an active playtest is not replaced. Each app carries its own assets and resolves them from `Contents/Resources/assets` independently of Finder's working directory. The packaging script performs a fresh locked offline build, stages/signs in a temporary folder, validates Info.plist and creates the archive before moving the app into the output directory. It refuses to overwrite an existing app or archive. Developer ID signing, notarization and clean-machine distribution are not completed.

On this Mac, the Documents file provider reapplies Finder metadata to the app directory after installation, causing a later strict signature check to reject that directory even though the app launches. The archive was extracted in `/private/tmp` and its ad-hoc signature verified with `codesign --verify --deep --strict`. Use that archive as the preserved candidate artifact.

## Next production work

1. Human playtest both CG sides and the opening missions of both Elder Fleet sides. Record deaths, time to first shot, unreadable attacks and objective clarity.
2. Finalize bow/stern, weapon and engine anchors for every offered hull. The import is consistently transparent and top-down, but manifest visual approvals remain pending. Replace the legacy Gila placeholder before exposing its mode.
3. Bring Elder Fleet objectives beyond its current wave-and-commander structure: escorts, transport interception and faction-specific mission actions should match their briefings.
4. Qualify unlock/save/relaunch behavior, physical controllers and layouts at other window sizes. Finish the keyboard rebinding editor and controller remap/reconnect qualification.
5. Listen to the new effects at normal and crowded combat levels, then revise impacts, warnings and UI cues for the same industrial direction. Preserve threat audibility during simultaneous fire and explosions.
6. Measure rendered frame times under busy waves, then address measured bottlenecks before expanding content.

## Verification record

- **520 tests pass**, zero failures or ignored tests, including both three-mission CG faction routes, all four chapter/faction entry paths, carrier timing/stress, destroyer projectile damage/HUD depletion and Xbox trigger/disconnect handling. Command: `cargo test --offline --locked --no-fail-fast`. Local log: `build/playtest-review/tests.log`.
- `cargo clippy --offline --locked --all-targets -- -D warnings` passes for all targets, including the updated carrier tests. Rust reports a dependency future-compatibility notice for `block 0.1.6`; this is separate from current Clippy warnings.
- `scripts/verify_ship_identity.py`: **80 hulls, zero identity mismatches**. All 79 imported renders have transparent corners; final visual facing approvals remain pending.
- Seven new synthesized combat cues pass WAV, signal-level and edge checks. Exported peaks range from 0.286 to 0.614 full scale, with zero clipped samples in the individual previews. This does not measure the combined live mix. Local audition: `build/audio-review/preview.wav`; measured levels: `build/audio-review/levels.json`.
- An earlier native candidate launched through macOS LaunchServices from `/private/tmp`, loaded bundled assets and displayed the new main menu and chapter selector. The owner played that build and reported the commander/health issues addressed above. The new candidate's automated checks do not substitute for another human playthrough.
- **Current candidate startup passes:** LaunchServices opened the exact new bundle hidden from `/private/tmp` with an isolated muted save. Logs confirm initialization, bundled asset resolution, sound generation and saved settings application; the process remained alive without a panic and only that temporary instance was closed. Native startup still logs five absent optional Ogg music slots and uses procedural music. Imported music needs decoder/loading cleanup, as documented in the audio README.
- **Release bundle:** ARM64, 57,656,216 bytes across 126 files (31 MiB compressed archive). Info.plist is valid. The extracted archive passes strict ad-hoc signature verification. Local logs and per-file hashes: `build/playtest-review/`.
- Executable SHA-256: `da0415139e380b888ecdd236a6de5a51668050b0590309441beab1926d2e2708`.
- Archive SHA-256: `17905995e29a21a1fc7fd19afcd0b992510dc052f0ea6c9583d0abca28ce5109`.
