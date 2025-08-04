# MultiQueue2 Documentation

MultiQueue2 is a fast bounded MPMC (Multi-Producer Multi-Consumer) broadcast queue implementation in Rust, based on the LMAX Disruptor design.

## Quick Links

- [GitHub Repository](https://github.com/abbychau/multiqueue2)
- [API Documentation](https://docs.rs/multiqueue2)
- [Crates.io](https://crates.io/crates/multiqueue2)
- [Benchmarks](benchmarks/)

## Overview

MultiQueue2 provides lockless high-performance message passing with broadcast capabilities and futures support. It supports two main queue types:

- **Broadcast Queue**: Supports broadcasting messages to multiple independent streams
- **MPMC Queue**: Traditional multi-producer multi-consumer queue that moves items directly

## Performance Characteristics

- ~300ns per operation for SPSC scenarios
- ~400ns per operation for MPMC scenarios  
- Latency approximates inter-core communication delay (40-70ns on single socket)
- 30% throughput boost possible by switching from condvar to busy spin locks
- No allocations on push/pop operations

## Queue Configuration

MultiQueue2 uses hybrid locking with configurable wait strategies:

- `broadcast_queue(capacity)` - Default settings with condvar blocking
- `broadcast_queue_with(capacity, try_spins, yield_spins)` - Custom spin configuration
- `futures_multiqueue_with(capacity, try_spins, yield_spins)` - Futures variant with custom settings

## Examples

### Simple MPMC Queue

```rust
use multiqueue2::mpmc_queue;
use std::thread;

let (sender, receiver) = mpmc_queue(10);

thread::spawn(move || {
    for val in receiver {
        println!("Got {}", val);
    }
});

for i in 0..10 {
    sender.try_send(i).unwrap();
}
```

### Broadcast Queue

```rust
use multiqueue2::broadcast_queue;
use std::thread;

let (sender, receiver) = broadcast_queue(10);

// Create multiple streams
for i in 0..2 {
    let stream = receiver.add_stream();
    thread::spawn(move || {
        for val in stream {
            println!("Stream {} got {}", i, val);
        }
    });
}

receiver.unsubscribe(); // Remove original receiver

for i in 0..10 {
    loop {
        if sender.try_send(i).is_ok() {
            break;
        }
    }
}
```

## Benchmarks

View detailed performance benchmarks [here](benchmarks/).