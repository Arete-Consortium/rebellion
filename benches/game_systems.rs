use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bevy::ecs::world::World;
use bevy::math::Vec2;

use rebellion::core::{ScoreSystem, StyleGrade, SCREEN_HEIGHT, SCREEN_WIDTH};
use rebellion::systems::collision::SpatialGrid;
use rebellion::systems::scoring_v2::{ComboHeatSystem, HeatLevel};

fn bench_score_on_kill(c: &mut Criterion) {
    c.bench_function("score_on_kill_x1000", |b| {
        b.iter(|| {
            let mut score = ScoreSystem::default();
            for i in 0..1000u64 {
                score.on_kill(black_box(100 + i));
            }
            score.score
        });
    });
}

fn bench_spatial_grid(c: &mut Criterion) {
    let mut world = World::new();
    // Keep every enemy on screen; the old 16-column layout placed almost
    // half of its 500 entries beyond the grid and silently dropped them.
    let entries: Vec<_> = (0..500)
        .map(|i| {
            let x = ((i % 25) as f32 + 0.5) * SCREEN_WIDTH / 25.0 - SCREEN_WIDTH / 2.0;
            let y = ((i / 25) as f32 + 0.5) * SCREEN_HEIGHT / 20.0 - SCREEN_HEIGHT / 2.0;
            (world.spawn_empty().id(), Vec2::new(x, y))
        })
        .collect();
    let query_positions: Vec<_> = entries.iter().map(|&(_, pos)| pos).collect();
    let mut grid = SpatialGrid::new();
    for &(entity, pos) in &entries {
        grid.insert_enemy(entity, pos);
    }

    c.bench_function("spatial_grid_query_500_enemies_500_projectiles", |b| {
        b.iter(|| {
            let mut count = 0usize;
            for &pos in &query_positions {
                count += grid.get_nearby_enemies(black_box(pos)).count();
            }
            black_box(count)
        });
    });

    c.bench_function(
        "spatial_grid_rebuild_query_500_enemies_500_projectiles",
        |b| {
            b.iter(|| {
                // Production retains cell capacity and rebuilds contents each tick.
                grid.clear();
                for &(entity, pos) in &entries {
                    grid.insert_enemy(entity, black_box(pos));
                }
                let mut count = 0usize;
                for &pos in &query_positions {
                    count += grid.get_nearby_enemies(black_box(pos)).count();
                }
                black_box(count)
            });
        },
    );
}

fn bench_heat_classify(c: &mut Criterion) {
    c.bench_function("heat_classify_x10000", |b| {
        b.iter(|| {
            let mut result = HeatLevel::Cool;
            for i in 0..10000u32 {
                let heat = (i % 120) as f32;
                let was_overheated = i % 3 == 0;
                result = HeatLevel::from_heat(black_box(heat), black_box(was_overheated));
            }
            result
        });
    });
}

fn bench_combo_update(c: &mut Criterion) {
    c.bench_function("combo_update_1000_frames", |b| {
        b.iter(|| {
            let mut system = ComboHeatSystem::default();
            // Simulate gameplay: kills, firing, frame updates
            for i in 0..1000u32 {
                if i % 3 == 0 {
                    system.on_kill();
                }
                if i % 2 == 0 {
                    system.on_fire();
                }
                system.update(black_box(1.0 / 60.0));
            }
            (system.combo_count, system.heat)
        });
    });
}

fn bench_score_grade(c: &mut Criterion) {
    c.bench_function("score_get_grade_x10000", |b| {
        b.iter(|| {
            let mut last_grade = StyleGrade::D;
            for i in 0..10000u32 {
                let score = ScoreSystem {
                    multiplier: (i % 100) as f32,
                    ..Default::default()
                };
                last_grade = score.get_grade();
            }
            last_grade
        });
    });
}

criterion_group!(
    benches,
    bench_score_on_kill,
    bench_spatial_grid,
    bench_heat_classify,
    bench_combo_update,
    bench_score_grade,
);
criterion_main!(benches);
