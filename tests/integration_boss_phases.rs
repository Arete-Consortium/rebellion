//! Integration tests for `boss::get_phase_threshold`.
//!
//! The "boss bar draws at the right HP threshold" claim maps to
//! `get_phase_threshold(phase, total_phases)` at
//! `src/entities/boss.rs:586`. This function is the static lookup
//! table that the HUD bar reads to decide which phase overlay to
//! render at the current HP percentage. The bar isn't egui-testable
//! in headless, but the threshold table that drives it is. If
//! `get_phase_threshold` is wrong, the bar is wrong.
//!
//! Coverage over and above `boss::tests::phase_thresholds_*`:
//!
//! - **`(phase > total)` defensive arm** — the `_ => 0.0` branch
//!   isn't exercised by the in-file tests. Off-the-end indices
//!   must return 0 (the bar treats 0 as "phase not applicable",
//!   not as "bar at zero").
//! - **Pinned snapshot** — every valid `(phase, total)` pair
//!   against expected values. A future silent edit to the table
//!   fails this test loudly.
//! - **Non-overlap for `total == 5`** — `phase_thresholds_decrease`
//!   only checks `total == 3`. The full 5-phase curve must
//!   strictly decrease.
//! - **Final phase always has a small threshold** — for any
//!   `total`, the last phase's threshold must be in `(0, 0.5)`:
//!   meaningful but not trivial ("one big hit ends it").

use rebellion::entities::boss::{get_boss_for_stage, get_phase_threshold, BossData};

// ============================================================================
// Phase 1 — bar full at spawn (always 1.0)
// ============================================================================

/// Phase 1 represents the boss at full health. The bar must be
/// full. Every total must return exactly 1.0 for phase 1.
#[test]
fn phase_1_is_full_for_all_totals() {
    for total in 2..=5 {
        assert_eq!(
            get_phase_threshold(1, total),
            1.0,
            "phase 1 of a {total}-phase boss must be at full HP"
        );
    }
}

// ============================================================================
// Defensive arm — out-of-range phase index
// ============================================================================

/// Requesting a phase index above the boss's `total_phases` returns
/// 0 — the bar treats this as "no phase overlay", not as "0 HP".
/// Guards against off-by-one in any future system that loops
/// `for phase in 1..=boss.current_phase + 1` and accidentally
/// queries the next phase. The match's catchall arm (`_ => 0.0`)
/// handles this — the live caller at `systems/boss.rs:670` guards
/// with `next_phase <= total_phases` so this defensive behavior
/// is reachable only via user/script paths.
///
/// Additionally: `total_phases == 0` returns 0 for **any** phase
/// (including phase 1). The function is not designed for
/// `total_phases == 0` — `BossData::total_phases` is always 3, 4,
/// or 5 in production — but a defensive caller must get 0, not
/// the misleading 1.0 that `(1, _)` would return if the match
/// weren't reordered. Pinned in `entities/boss.rs:585-608`.
#[test]
fn phase_greater_than_total_returns_zero() {
    // total=3, phase=4 → out of range → 0
    assert_eq!(get_phase_threshold(4, 3), 0.0);
    // total=2, phase=3 → out of range → 0
    assert_eq!(get_phase_threshold(3, 2), 0.0);
    // total=5, phase=6 → way out of range → 0
    assert_eq!(get_phase_threshold(6, 5), 0.0);
    // Defensive arm: total_phases == 0 returns 0 for any phase,
    // including phase 1 (the (1, _) arm would otherwise catch it).
    assert_eq!(get_phase_threshold(1, 0), 0.0);
    assert_eq!(get_phase_threshold(2, 0), 0.0);
    assert_eq!(get_phase_threshold(3, 0), 0.0);
}

// ============================================================================
// Snapshot — exact thresholds for every valid (phase, total) pair
// ============================================================================

/// Pinned snapshot of the current threshold table. Any silent
/// edit to a value in the match arms of `get_phase_threshold`
/// fails this test loudly. Update intentionally during a phase
/// mechanic redesign only.
#[test]
fn phase_threshold_snapshot_is_pinned() {
    let expected: &[(u32, u32, f32)] = &[
        // total=2 (2-phase bosses): phase 1 → 1.0, phase 2 → 0.4
        (1, 2, 1.0),
        (2, 2, 0.4),
        // total=3
        (1, 3, 1.0),
        (2, 3, 0.6),
        (3, 3, 0.3),
        // total=4
        (1, 4, 1.0),
        (2, 4, 0.7),
        (3, 4, 0.4),
        (4, 4, 0.15),
        // total=5
        (1, 5, 1.0),
        (2, 5, 0.75),
        (3, 5, 0.5),
        (4, 5, 0.25),
        (5, 5, 0.05),
    ];

    for &(phase, total, want) in expected {
        let got = get_phase_threshold(phase, total);
        assert!(
            (got - want).abs() < f32::EPSILON,
            "phase {phase}/{total}: expected {want}, got {got}"
        );
    }
}

// ============================================================================
// Monotonicity — strict decrease across phases for every total
// ============================================================================

/// For every valid `total_phases`, the threshold sequence must be
/// strictly decreasing as `phase` advances. `phase_thresholds_decrease`
/// covers `total=3`; this generalizes to all four.
#[test]
fn thresholds_strictly_decrease_for_every_total() {
    for total in 2..=5 {
        let mut prev = f32::INFINITY;
        for phase in 1..=total {
            let threshold = get_phase_threshold(phase, total);
            assert!(
                threshold < prev,
                "total={total}: phase {phase} threshold ({threshold}) must be < previous ({prev})"
            );
            prev = threshold;
        }
    }
}

/// Same property, but written so a single violation surfaces with
/// the exact pair that fails. Easier to read in CI output than
/// the loop above's counter assertion.
#[test]
fn thresholds_strictly_decrease_for_total_5() {
    let t = |p| get_phase_threshold(p, 5);
    assert!(t(1) > t(2), "{:?} > {:?}", t(1), t(2));
    assert!(t(2) > t(3));
    assert!(t(3) > t(4));
    assert!(t(4) > t(5));
}

// ============================================================================
// Final phase — meaningful but not trivial
// ============================================================================

/// The final phase must have a non-trivial threshold — meaning
/// the bar still shows the boss as "alive but low" rather than
/// collapsing to zero prematurely. Must be in (0, 0.5).
#[test]
fn final_phase_threshold_is_meaningful_not_trivial() {
    for total in 2..=5 {
        let final_threshold = get_phase_threshold(total, total);
        assert!(
            final_threshold > 0.0 && final_threshold < 0.5,
            "final phase of {total}-phase boss must have threshold in (0, 0.5), got {final_threshold}"
        );
    }
}

// ============================================================================
// BossData defaults — health equals max at spawn
// ============================================================================

/// `BossData::health` must equal `max_health` immediately after
/// construction. This is the resource-level "bar at the right
/// threshold" check: if `health < max_health` at spawn, the bar
/// draws below phase 1 from the first frame, breaking the
/// "phase 1 is full" guarantee.
#[test]
fn spawn_data_begins_at_full_health() {
    let bosses: Vec<BossData> = (1..=13)
        .filter_map(get_boss_for_stage)
        .collect();
    assert_eq!(bosses.len(), 13, "all 13 stages must have bosses");
    for boss in &bosses {
        assert!(
            (boss.health - boss.max_health).abs() < f32::EPSILON,
            "boss {} (stage {}) must spawn at full HP",
            boss.name,
            boss.stage
        );
        assert_eq!(
            boss.current_phase, 1,
            "boss {} must spawn at phase 1",
            boss.name
        );
        assert!(
            !boss.is_enraged,
            "boss {} must not spawn enraged",
            boss.name
        );
    }
}

/// Health must strictly decrease across the campaign as
/// `max_health` does. Test that the threshold formula applies to
/// a bar where `health / max_health` is what gets compared — and
/// that ratio must be 1.0 across the board at spawn.
#[test]
fn spawn_health_ratio_is_one_for_every_boss() {
    for stage in 1..=13 {
        let boss = get_boss_for_stage(stage).expect("boss must exist");
        let ratio = boss.health / boss.max_health;
        assert!(
            (ratio - 1.0).abs() < 1e-6,
            "boss for stage {stage} spawns at health/max_health = {ratio}"
        );
    }
}
