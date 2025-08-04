# MultiQueue2 Benchmarks

This page contains detailed performance benchmarks for MultiQueue2, generated using Criterion.rs.

## Benchmark Categories

### SPSC Comparison
Compares MPMC vs Broadcast queues in single-producer single-consumer scenarios.

### MPMC Queue
Tests various producer/consumer combinations (1p/1c, 1p/2c, 2p/1c, etc.).

### Broadcast Queue
Tests broadcast functionality with multiple streams and consumers.

### Futures Queue
Benchmarks async/await performance using tokio.

## How to Run Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suites
cargo bench --bench quick_benchmark      # Quick benchmarks (smaller dataset)
cargo bench --bench multiqueue_benchmarks # Comprehensive benchmarks

# Run specific benchmark categories
RUSTFLAGS='-C target-cpu=native' cargo bench spsc
RUSTFLAGS='-C target-cpu=native' cargo bench mpmc
RUSTFLAGS='-C target-cpu=native' cargo bench broadcast
RUSTFLAGS='-C target-cpu=native' cargo bench futures
```

## Benchmark Reports

The benchmark reports below are automatically generated from the latest Criterion output. Each report includes throughput measurements, latency distributions, and regression analysis.

*Note: Reports are updated when benchmarks are run. Use the `update-docs.sh` script to refresh with latest results.*

---

*Benchmark reports will appear here after running `./scripts/update-docs.sh`*