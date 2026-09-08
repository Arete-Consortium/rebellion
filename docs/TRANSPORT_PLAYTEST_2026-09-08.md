# Hedion transport playtest — September 8

This candidate adds a playable objective to the second Elder Fleet mission on both faction routes. It follows the [save and audio checkpoint](PLAYTEST_2026-09-08.md).

## Play the encounter

Open `dist/transport-20260908/Rebellion.app`. Choose **Play → Minmatar / Amarr → faction → difficulty → hull** and complete the opening commander encounter. Continue to mission 2:

- **Minmatar — Slave Revolt:** stay within the Bestower's marked ring for 12 cumulative seconds to board and liberate it. Progress is retained when you move out to dodge. The transport keeps moving toward its departure point.
- **Amarr — Convoy Defense:** stay within the ring to guide the convoy to evacuation, requiring 40 cumulative seconds in range. One raider from each scheduled carrier wave targets the transport; an amber targeting line appears before its shot.

Both operations have a 90-second deadline and a visible hull percentage. Player weapons do not damage the transport. Enemy projectiles do; overlapping the transport allows the pilot to intercept those shots. After success, the transport warps out. All three scheduled waves must clear before the commander arrives. Defeating that commander completes the mission. Transport loss or timeout produces an objective-specific failure screen and allows a fresh attempt at the same mission.

Pause freezes the objective. Retry resets its timer, hull and progress. The save format is unchanged; partial boarding progress is local to the current attempt.

## Related repairs

- Invulnerability now has one outlined hexagonal shield around the player, with six child edges. Previously seven entities shared a single-entity marker, so the update query failed and spawned seven more every frame. Overlapping filled sprites obscured the hull and the effect count grew without a cap. The center is now transparent and expiration removes the entire outline.
- Elder Fleet waves spawn within the existing offscreen margin; the previous random Y range could cause an incoming ship to be culled immediately.
- The failure screen shows the correct Elder Fleet mission and explains objective failure. Retry/Exit also accept pointer input, and keyboard/controller confirmation hints are visible.
- Empty inventory panels are hidden, and the bottom HUD sizes to its content so the ability label remains visible. Combat clears to dark space behind the existing starfield and fleet traffic.
- The first Amarr-side commander name now identifies its Thrasher hull.

## Verification

- **533 tests pass**, zero failed or ignored, including eight new transport schedule tests and the sustained-shield regression. The complete existing CG route suite still passes.
- `cargo clippy --offline --locked --all-targets -- -D warnings`, formatting and diff checks pass. The existing `block 0.1.6` future-Rust compatibility notice remains.
- The 80-hull identity verifier reports zero mismatches. No sprite files changed in this pass.
- Both faction briefings, encounters and objective-failure screens were captured from the native renderer with isolated muted saves. The transport uses a transparent hull, the player remains visible inside the outlined invulnerability shield, and the Amarr preview shows its designated attacker warning. Captures and logs are in `build/playtest-review/transport-20260908/{minmatar,amarr}/`.
- The short debug Minmatar fixture measured p50/p95 frame intervals of 18.51/21.97 ms before the shield fix and 16.64/18.65 ms afterward. These are two brief diagnostic runs, not a controlled release benchmark. The regression provides the stronger evidence: one root with six edges across 180 updates, with full cleanup afterward.
- Native review logs also contain B0003 warnings when menu entities are cleaned up; no render-review panic or missing-asset error occurred. This remains cleanup work for release qualification.
- **Packaged native startup passes:** the clean ZIP was extracted under `/private/tmp`, its ad-hoc signature verified, and its app launched through LaunchServices with an isolated muted save. It initialized, stayed alive and had no panic or missing-asset error. The executable matches the app in `dist/transport-20260908/`. Only the test instance was closed.
- ARM64 app: **57,739,559 bytes**, 126 files. ZIP: **32,066,054 bytes**. The runtime asset set is unchanged from the preceding candidate.
- Executable SHA-256: `b3480e693246188724ffa23a267e01e3be6b121d4ea7b66c45e6ec5fe80efd00`.
- Archive SHA-256: `71d716160c489cb815b5ffce7a6b6d6596db5d334efbabde9aedacd2b90f9eed`.

The initial LaunchServices check failed with error -10810 while its stdout/stderr destinations were in Documents. The same extracted executable ran directly, and LaunchServices succeeded with those logs inside the temporary validation folder. The evidence logs were copied back afterward; no production workaround or permission change was required.

The 533-test suite covers the gameplay and shield changes. The final adjustment from a fixed bottom-HUD height to automatic sizing was subsequently compiled, checked by Clippy and verified in both native faction captures. Candidate source digests are recorded in `source-manifest.json`, and package hashes/launch results in `bundle-manifest.json`. Earlier playable builds are preserved. No commit, push, publication or notarization was performed.

The scripted native review is reproducible with `cargo run --offline --locked --example transport_review -- minmatar` (or `amarr`) from the checkout. It creates its own muted temporary save, uses an invulnerable scripted pilot, captures the briefing/combat/failure screens and exits. Outputs go to `build/playtest-review/transport-20260908/`. Its short debug frame-time sample is useful for local diagnosis; it is not the release-build performance gate or a human difficulty test.

The new transport uses the existing Bestower asset, type 1944, and its existing 180-degree manifest correction. No image files, dependencies or runtime asset weight were added. The captured hull has a transparent background and its nose points along its upward route. Final bow/stern and hardpoint approval for the entire roster remains open.

## Next acceptance pass

1. Play both mission-2 objectives on Newbro with keyboard, then repeat with the physical controller when detected. Check whether progress, dangerous shots and failure causes are understandable without extra explanation.
2. Tune the 12/40/90-second starting values and 400-point transport hull from actual completion times and deaths. They are initial encounter tuning, not a proven difficulty target.
3. Complete the three-mission Caldari–Gallente packaged run from both sides, including pause, death/retry and relaunch. Automated route checks are separate evidence.
4. Capture release-build frame times in ordinary waves, dense combat and boss fights. Finish the per-hull art review and crowded-combat sound mix before public qualification.
