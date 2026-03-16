use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bevy::ecs::world::World;
use bevy::math::Vec2;

use rebellion::core::{ScoreSystem, StyleGrade};
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
    c.bench_function("spatial_grid_insert_query_500", |b| {
        let mut world = World::new();
        let entities: Vec<_> = (0..500).map(|_| world.spawn_empty().id()).collect();

        b.iter(|| {
            let mut grid = SpatialGrid::new();
            // Insert 500 entities across the grid
            for (i, &entity) in entities.iter().enumerate() {
                let x = (i % 16) as f32 * 50.0 - 400.0;
                let y = (i / 16) as f32 * 50.0 - 350.0;
                grid.insert_enemy(entity, Vec2::new(x, y));
            }
            // Query from 10 positions
            let mut count = 0usize;
            for j in 0..10 {
                let qx = (j as f32 - 5.0) * 80.0;
                count += grid
                    .get_nearby_enemies(black_box(Vec2::new(qx, 0.0)))
                    .count();
            }
            count
        });
    });
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
