# Performance Audit Checklist

Use this checklist when reviewing code for performance bottlenecks.

## 🏁 Baseline
- [ ] Has the code been benchmarked/profiled?
- [ ] Is optimization being applied to a "hot path"?

## 🧠 Memory
- [ ] Are we cloning large data structures unnecessarily?
- [ ] Can we use references (`&T`) instead of transfers of ownership?
- [ ] Are we reusing buffers in loops?
- [ ] Are we using `with_capacity` for collections?
- [ ] Are there excessive heap allocations where stack would suffice?

## 🏃 Iteration
- [ ] Are we using `Iterator` methods effectively (lazy vs greedy)?
- [ ] Can we avoid intermediate collections (`.collect()`)?
- [ ] Is loop unrolling or SIMD appropriate for this data?

## 🌐 I/O
- [ ] is I/O buffered (`BufReader`/`BufWriter`)?
- [ ] Are we locking `stdout` for batch writes?
- [ ] is networked I/O using appropriate packet sizes?

## ⚙️ Concurrency
- [ ] Is work distributed effectively with `Rayon`?
- [ ] Are we avoiding lock contention (e.g., using `parking_lot` or lock-free structures)?
- [ ] Are we blocking async executors?

## 🛠️ Build & Config
- [ ] is `lto` enabled?
- [ ] Are `codegen-units` set to 1 for release?
- [ ] is `panic = "abort"` appropriate to reduce binary size and potentially improve performance?
