use code_rag::indexer::CodeChunker;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn bench_chunking(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunking");

    // Test payload: A moderate Rust file
    let code = r#"
        pub struct CodeChunker;
        impl CodeChunker {
            pub fn chunk_file(&self, path: &str, content: &str, mtime: u64) -> Vec<Chunk> {
                // ... implementation ...
                vec![]
            }
        }
        fn large_dummy_function() {
            // ... a lot of lines ...
            println!("Hello");
        }
    "#
    .repeat(100); // Scale up to make it measurable

    group.throughput(Throughput::Bytes(code.len() as u64));

    group.bench_function("chunk_rust_file", |b| {
        let chunker = CodeChunker::default();
        let mut reader = std::io::Cursor::new(code.as_bytes());
        b.iter(|| {
            reader.set_position(0);
            chunker.chunk_file("bench.rs", &mut reader, 0)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_chunking);
criterion_main!(benches);
