//! Benchmarks for the event queue hot path (roadmap A3).
//!
//! The event loop is the engine's hottest path: every action and every
//! triggered effect pushes and pops events. These benchmarks measure the
//! push/pop and drain throughput of `EventQueue` with a realistic mix of
//! priorities (what the engine produces during combat resolution).

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use orange_stone::core::entity::Entity;
use orange_stone::core::event::{Event, EventQueue, Priority};
use orange_stone::core::player::PlayerId;

/// 混合优先级的"战斗结算"式事件流 — 每 3 个事件按 Highest/Normal/Lowest 分布。
fn fill_mixed(queue: &mut EventQueue, n: usize) {
    for i in 0..n {
        let priority = match i % 3 {
            0 => Priority::Lowest,
            1 => Priority::Normal,
            _ => Priority::Highest,
        };
        queue.push_with_priority(
            Event::DamageDealt {
                source: Entity::new(i as u32, 0),
                target: Entity::new(i as u32 + 1, 0),
                amount: (i % 7) as i32,
            },
            priority,
        );
    }
}

/// 全部 Normal 优先级的事件流（最简单的对局路径，如抽牌阶段）。
fn fill_normal(queue: &mut EventQueue, n: usize) {
    for i in 0..n {
        queue.push(Event::CardDrawn {
            player: PlayerId::Player1,
            card: Entity::new(i as u32, 0),
        });
    }
}

fn bench_push_pop_mixed(c: &mut Criterion) {
    const N: usize = 10_000;
    c.bench_function("event_queue/push+pop_mixed_10k", |b| {
        b.iter_batched(
            EventQueue::new,
            |mut q| {
                fill_mixed(&mut q, N);
                while let Some(event) = q.pop_front() {
                    black_box(event);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_push_pop_normal(c: &mut Criterion) {
    const N: usize = 10_000;
    c.bench_function("event_queue/push+pop_normal_10k", |b| {
        b.iter_batched(
            EventQueue::new,
            |mut q| {
                fill_normal(&mut q, N);
                while let Some(event) = q.pop_front() {
                    black_box(event);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_drain(c: &mut Criterion) {
    const N: usize = 10_000;
    c.bench_function("event_queue/drain_mixed_10k", |b| {
        b.iter_batched(
            EventQueue::new,
            |mut q| {
                fill_mixed(&mut q, N);
                black_box(q.drain().len());
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_push_pop_mixed,
    bench_push_pop_normal,
    bench_drain
);
criterion_main!(benches);
