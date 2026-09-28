# Rebellion playtest follow-up — 2026-09-28

Human report on browser artifact built from 6bf8ca15: “played ok”, with requests for visual polish, enemy coordination, UI polish, and reliable damage from every enemy. This confirms basic controller play only; it does not certify all campaigns or reconnect behavior.

## Candidate scope

- Generic Triglavian hulls receive a real disintegrator beam instead of a zero-damage projectile weapon. Advanced hull mappings are included.
- Enemy ships inflict kinetic contact damage. A shared 0.75-second impact grace prevents stacked ships draining the player every fixed tick; dodge and invulnerability still apply. Kamikaze ships die on a successful ram.
- Beam damage requires a 0.65-second lock. Leaving range resets it. Amber acquisition lines precede damage.
- Escorts take separate alternating wing positions around carriers and tanks. Rushing ships pursue independently. Weaver movement integrates velocity once instead of twice.
- Muzzle cues, rush chevrons, and damaged-enemy integrity bars explain threats. Ability labels show READY / ACTIVE / RECHARGING; wave text has stronger contrast.

## Art and interaction direction

Visual thesis: dark fleet combat, restrained amber warnings, clear silhouettes and open combat space.

Content hierarchy: battlefield first; threat cues at the enemy; survival and ability state in the existing HUD; score secondary.

Motion: contracting muzzle rings signal an imminent shot; contracting beam acquisition rings precede sustained fire; directional chevrons expose the rush vector without rapid flashing.

## Validation

New tests exercise real spawn and attack systems followed by collision, enrichment and damage resolution across 32 generic hull IDs (including fallback) and all 15 specialized variants. Additional tests cover beam acquisition/range reset, invulnerability, ram destruction, contact grace, and distinct escort slots. These tests are not a complete campaign/boss playthrough.

The local Mac has no Rust toolchain and system Git is blocked by an unaccepted Xcode license. Validation runs in the existing Platform Playtest workflow on an isolated candidate branch. Record the exact result before replacing the preserved, human-played package. New visuals and difficulty still need browser/controller review.
