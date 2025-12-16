# Parallel File Processor

Rust project that concurrently processes files with a generic thread pool (no external crates).

Features:
- Generic ThreadPool (dynamic threads, shutdown)
- File analyzers: word count, line count, char frequency, file size
- Uses Arc and Mutex for shared state and progress
- Cancellation support
- Per-file progress/status, error reporting, processing time
- Unit & integration tests (see /tests)
- Simple performance bench (see /benches/simple_bench.rs)

## Build & Run

Build:
