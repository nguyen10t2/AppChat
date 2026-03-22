use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use dashmap::DashMap;

const THREADS: usize = 4;
const READS_PER_THREAD: usize = 2_000;
const WRITES_PER_THREAD: usize = 800;

fn build_dashmap(users: usize) -> Arc<DashMap<u64, Vec<u64>>> {
    let map = Arc::new(DashMap::new());
    for user_id in 0..users as u64 {
        map.insert(user_id, vec![user_id, user_id + 1]);
    }
    map
}

fn build_mutex_map(users: usize) -> Arc<Mutex<HashMap<u64, Vec<u64>>>> {
    let mut inner = HashMap::with_capacity(users);
    for user_id in 0..users as u64 {
        inner.insert(user_id, vec![user_id, user_id + 1]);
    }
    Arc::new(Mutex::new(inner))
}

fn bench_parallel_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("ws_map_parallel_reads");

    for users in [10usize, 100usize, 500usize] {
        let dash = build_dashmap(users);
        group.bench_with_input(BenchmarkId::new("dashmap", users), &users, |b, users| {
            b.iter(|| {
                std::thread::scope(|scope| {
                    for thread_idx in 0..THREADS {
                        let dash = dash.clone();
                        scope.spawn(move || {
                            for i in 0..READS_PER_THREAD {
                                let key = ((thread_idx * READS_PER_THREAD) + i) % *users;
                                let len = dash.get(&(key as u64)).map(|entry| entry.len()).unwrap_or(0);
                                black_box(len);
                            }
                        });
                    }
                });
            });
        });

        let mutex_map = build_mutex_map(users);
        group.bench_with_input(BenchmarkId::new("mutex_hashmap", users), &users, |b, users| {
            b.iter(|| {
                std::thread::scope(|scope| {
                    for thread_idx in 0..THREADS {
                        let mutex_map = mutex_map.clone();
                        scope.spawn(move || {
                            for i in 0..READS_PER_THREAD {
                                let key = ((thread_idx * READS_PER_THREAD) + i) % *users;
                                let len = mutex_map
                                    .lock()
                                    .expect("mutex poisoned")
                                    .get(&(key as u64))
                                    .map(std::vec::Vec::len)
                                    .unwrap_or(0);
                                black_box(len);
                            }
                        });
                    }
                });
            });
        });
    }

    group.finish();
}

fn bench_parallel_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("ws_map_parallel_writes");

    for users in [10usize, 100usize, 500usize] {
        group.bench_with_input(BenchmarkId::new("dashmap", users), &users, |b, users| {
            b.iter(|| {
                let dash = build_dashmap(*users);
                std::thread::scope(|scope| {
                    for thread_idx in 0..THREADS {
                        let dash = dash.clone();
                        scope.spawn(move || {
                            for i in 0..WRITES_PER_THREAD {
                                let key = ((thread_idx * WRITES_PER_THREAD) + i) % *users;
                                dash.insert(key as u64, vec![i as u64, (i + 1) as u64]);
                            }
                        });
                    }
                });
                black_box(dash.len());
            });
        });

        group.bench_with_input(BenchmarkId::new("mutex_hashmap", users), &users, |b, users| {
            b.iter(|| {
                let mutex_map = build_mutex_map(*users);
                std::thread::scope(|scope| {
                    for thread_idx in 0..THREADS {
                        let mutex_map = mutex_map.clone();
                        scope.spawn(move || {
                            for i in 0..WRITES_PER_THREAD {
                                let key = ((thread_idx * WRITES_PER_THREAD) + i) % *users;
                                mutex_map
                                    .lock()
                                    .expect("mutex poisoned")
                                    .insert(key as u64, vec![i as u64, (i + 1) as u64]);
                            }
                        });
                    }
                });
                black_box(mutex_map.lock().expect("mutex poisoned").len());
            });
        });
    }

    group.finish();
}

criterion_group!(
    websocket_benches,
    bench_parallel_reads,
    bench_parallel_writes,
);
criterion_main!(websocket_benches);
