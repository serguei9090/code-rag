# Rust Performance Best Practices

Detailed technical guidelines for optimizing Rust applications.

## 1. Data Structures & Layout

### Contiguous Memory
- **Prefer `Vec<T>`**: Most operations are fastest on contiguous memory due to cache locality.
- **Avoid `LinkedList`**: Almost never faster than `Vec` due to cache misses.
- **Small-Size Optimizations**: If a collection is usually small, consider `SmallVec` or `TinyVec`.

### Map Selection
- **`HashMap`**: Fast average-case access. Use a faster hasher (`fxhash` or `ahash`) if protection against HashDoS isn't needed.
- **`BTreeMap`**: Good for range queries and better cache locality for small-to-medium datasets.

## 2. Memory Optimization

### Allocation Reuse
Instead of:
```rust
for item in items {
    let mut buffer = Vec::new();
    // work...
}
```
Use:
```rust
let mut buffer = Vec::with_capacity(expected_size);
for item in items {
    buffer.clear();
    // work...
}
```

### Passing Data
- Prefer `&[T]` over `&Vec<T>` and `&str` over `&String`.
- Use `Box<[T]>` instead of `Vec<T>` for static data to save a `usize` (capacity).
- Use `Arc<T>` for sharing read-only data across threads without cloning.

## 3. String & I/O

### Buffered I/O
- Always wrap `File` or `TcpStream` in `BufReader` or `BufWriter`.
- Standard `print!` is slow; use `io::stdout().lock()` for high-volume output.

### String Concatenation
- Prefer `format_args!` or `writeln!` over repeated `+` or `.clone()`.
- Use `String::with_capacity()` if you know the approximate length of the final string.

## 4. Advanced Techniques

### Static vs Dynamic Dispatch
- Prefer generics (`impl Trait` or `<T: Trait>`) for static dispatch (monomorphization), which allows inlining.
- Avoid `Box<dyn Trait>` in hot loops as it incurs vtable lookup overhead and prevents inlining.

### Branch Prediction
- Write "greener" code: make the common case the straight-line path.
- Avoid unpredictable branches in tight loops.

### Link-Time Optimization (LTO)
- Enable `lto = "fat"` or `lto = "thin"` in `Cargo.toml` for production releases.
- Set `codegen-units = 1` for maximum optimization potential at the cost of compile time.
