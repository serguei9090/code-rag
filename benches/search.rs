use code_rag::bm25::BM25Index;
use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn bench_search(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Seed index with some dummy data
    // Note: Creating the index is expensive, so we do it outside the measurement loop

    // For now, we simulate a "hot" search bench on a pre-loaded index
    // Note: In a real scenario, we'd need a populated DB.
    // This serves as a placeholder structure for the BM25 logic specifically.

    // Since BM25Index::new is what we want to test load time for, or specific query logic.
    // However, without a real large index, raw BM25 lookup is trivial.
    // We'll bench `BM25Index::new` initialization time as a proxy for "startup latency".

    c.bench_function("bm25_load_empty", |b| {
        b.iter(|| BM25Index::new(db_path, false, "log").unwrap())
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
