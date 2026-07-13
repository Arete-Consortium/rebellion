# Mission 3 — Deterministic Integration Tests: PR Checklist

**Mission:** Build behavioral integration tests using the Mission 2 determinism infrastructure. Validate that core game mechanics produce expected outcomes in a headless, deterministic environment.

**Baseline commit:** `739ee54` (post-Mission 2)

**Branch prefix:** `arch/m03-*`

**Acceptance gate (every PR):**
- `cargo test` — all 284+ existing tests + new tests pass
- `cargo build`
- `cargo fmt --check`
- `cargo clippy -D warnings`

---

## PR #1 — `arch/m03-headless-gameplay`: Expand headless app with GameplayPlugin

**Goal:** The headless app can run gameplay systems (not just simulation).

**Files:**
```
src/app_builder.rs          // Add GameplayPlugin, event registrations, state transition support
```

**Changes:**
- Add `GameplayPlugin` to `build_headless_app()`
- Move campaign event registrations from `main.rs` into a plugin (or replicate in app_builder)
- Add `NextState<GameState>` resource so tests can transition into `Playing`
- Ensure no asset-loading plugins (ContentPlugin, PresentationPlugin) are pulled in
- Add test: transition to `GameState::Playing`, run 10 ticks, assert no panic

---

## PR #2 — `arch/m03-enemy-spawn-test`: Deterministic enemy spawn & movement

**Goal:** Verify enemies spawn and move predictably.

**Files:**
```
tests/enemy_spawn.rs        // New integration test
```

**Changes:**
- Build headless app, transition to `Playing`
- Spawn an enemy at a known position via `SpawnEnemyEvent`
- Run 60 fixed ticks (1 second)
- Query enemy `Transform`, assert it moved (not zero delta)
- Capture state hash before and after, assert it changed
- Run identical setup again, assert state hashes match (determinism)

---

## PR #3 — `arch/m03-projectile-e2e`: Projectile → collision → damage end-to-end

**Goal:** Verify the full combat pipeline works in headless mode.

**Files:**
```
tests/projectile_e2e.rs     // New integration test
```

**Changes:**
- Build headless app, transition to `Playing`
- Spawn player + enemy with known positions
- Spawn a player projectile heading toward enemy
- Run fixed ticks until collision detected
- Verify:
  - `ContactDetected` event was emitted
  - Enemy `EnemyStats.health` decreased
  - State hash changed after collision

---

## PR #4 — `arch/m03-scoring-test`: Scoring chain integration test

**Goal:** Verify scoring systems update correctly on enemy destruction.

**Files:**
```
tests/scoring_chain.rs      // New integration test
```

**Changes:**
- Build headless app, transition to `Playing`
- Spawn enemy with low health
- Directly apply damage to kill it (or use projectile test approach)
- Verify:
  - `ScoreSystem.chain_timer` is non-zero after kill
  - `ScoreSystem.multiplier` increased
  - `SaltMinerSystem.meter` increased
  - `SimStateHash` changed

---

## PR #5 — `arch/m03-golden-hash`: Golden state hash regression test

**Goal:** Capture a "known good" simulation fingerprint for regression detection.

**Files:**
```
tests/golden_hash.rs        // New integration test
tests/fixtures/golden.replay // Golden replay file (JSON)
```

**Changes:**
- Create a short replay (~180 frames = 3 seconds) that exercises:
  - Player movement
  - Enemy spawn
  - Projectile fire
  - Collision
- Run replay through headless app
- Capture `SimStateHash` at frames 60, 120, 180
- Hardcode expected hashes in test
- Add `UPDATE_GOLDEN` env var to regenerate hashes when intentional gameplay changes occur
- Document the regeneration procedure

**Rationale:** This is the capstone PR. It proves the Mission 2 infrastructure (replay + state hash) is actually useful for catching regressions.

---

## Post-Mission 3 Success Criteria

- [ ] Headless app includes GameplayPlugin and supports `GameState::Playing`
- [ ] At least 4 new integration tests exercising distinct mechanics
- [ ] Golden hash test captures a regression fingerprint
- [ ] All tests run in CI under `cargo test`
- [ ] `cargo test && cargo build && cargo fmt --check && cargo clippy -D warnings` passes
- [ ] Test runtime remains under 30 seconds total

---

*Append-only during Mission 3.*
