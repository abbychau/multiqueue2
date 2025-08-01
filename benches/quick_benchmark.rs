use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use multiqueue2 as multiqueue;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const QUEUE_SIZE: usize = 64;
const MESSAGE_COUNT: usize = 10_000;

fn bench_spsc_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("SPSC_Comparison");
    group.throughput(Throughput::Elements(MESSAGE_COUNT as u64));
    
    group.bench_function("mpmc_queue", |b| {
        b.iter(|| {
            let (sender, receiver) = multiqueue::mpmc_queue(QUEUE_SIZE as u64);
            let counter = Arc::new(AtomicUsize::new(0));
            let counter_clone = counter.clone();
            
            let handle = thread::spawn(move || {
                for item in receiver {
                    counter_clone.fetch_add(item, Ordering::Relaxed);
                }
            });
            
            for i in 0..MESSAGE_COUNT {
                while sender.try_send(black_box(i)).is_err() {
                    thread::yield_now();
                }
            }
            drop(sender);
            
            handle.join().unwrap();
            assert_eq!(counter.load(Ordering::Relaxed), (0..MESSAGE_COUNT).sum::<usize>());
        });
    });
    
    group.bench_function("broadcast_queue", |b| {
        b.iter(|| {
            let (sender, receiver) = multiqueue::broadcast_queue(QUEUE_SIZE as u64);
            let counter = Arc::new(AtomicUsize::new(0));
            let counter_clone = counter.clone();
            
            let handle = thread::spawn(move || {
                for item in receiver {
                    counter_clone.fetch_add(item, Ordering::Relaxed);
                }
            });
            
            for i in 0..MESSAGE_COUNT {
                while sender.try_send(black_box(i)).is_err() {
                    thread::yield_now();
                }
            }
            drop(sender);
            
            handle.join().unwrap();
            assert_eq!(counter.load(Ordering::Relaxed), (0..MESSAGE_COUNT).sum::<usize>());
        });
    });
    
    group.finish();
}

fn bench_queue_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Queue_Sizes");
    
    for &queue_size in [32, 64, 128, 256].iter() {
        group.throughput(Throughput::Elements(MESSAGE_COUNT as u64));
        group.bench_with_input(
            BenchmarkId::new("size", queue_size),
            &queue_size,
            |b, &size| {
                b.iter(|| {
                    let (sender, receiver) = multiqueue::mpmc_queue(size as u64);
                    let counter = Arc::new(AtomicUsize::new(0));
                    let counter_clone = counter.clone();
                    
                    let handle = thread::spawn(move || {
                        for item in receiver {
                            counter_clone.fetch_add(item, Ordering::Relaxed);
                        }
                    });
                    
                    for i in 0..MESSAGE_COUNT {
                        while sender.try_send(black_box(i)).is_err() {
                            thread::yield_now();
                        }
                    }
                    drop(sender);
                    
                    handle.join().unwrap();
                    assert_eq!(counter.load(Ordering::Relaxed), (0..MESSAGE_COUNT).sum::<usize>());
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_spsc_comparison, bench_queue_sizes);
criterion_main!(benches);