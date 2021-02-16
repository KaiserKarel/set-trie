use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};

fn bench_subsets(seed: usize) -> Duration {
    let mut trie = set_trie::SetTrie::new();

    let mut current = trie.entry(0..1).or_insert(0);
    for i in 1..seed {
        current = current.entry(i - 1..i).or_insert(i)
    }

    let now = Instant::now();
    trie.subsets(&(0..seed).into_iter().collect::<Vec<_>>())
        .count();
    now.elapsed()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("SetTrie::subsets 20000", |b| {
        b.iter_custom(|_| bench_subsets(black_box(200000)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
