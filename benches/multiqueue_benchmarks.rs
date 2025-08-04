use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use multiqueue2 as multiqueue;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const QUEUE_SIZE: usize = 1024;
const MESSAGE_COUNT: usize = 1_000_000;

fn bench_spsc_mpmc(c: &mut Criterion) {
    let mut group = c.benchmark_group("SPSC_MPMC");
    group.throughput(Throughput::Elements(MESSAGE_COUNT as u64));
    
    group.bench_function("spsc_mpmc", |b| {
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
    
    group.finish();
}

fn bench_spsc_broadcast(c: &mut Criterion) {
    let mut group = c.benchmark_group("SPSC_Broadcast");
    group.throughput(Throughput::Elements(MESSAGE_COUNT as u64));
    
    group.bench_function("spsc_broadcast", |b| {
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

fn bench_mpmc_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("MPMC_Queue");
    
    for &producer_count in [1, 2, 4].iter() {
        for &consumer_count in [1, 2, 4].iter() {
            group.throughput(Throughput::Elements(MESSAGE_COUNT as u64));
            group.bench_with_input(
                BenchmarkId::new("mpmc", format!("{}p_{}c", producer_count, consumer_count)),
                &(producer_count, consumer_count),
                |b, &(producers, consumers)| {
                    b.iter(|| {
                        let (sender, receiver) = multiqueue::mpmc_queue(QUEUE_SIZE as u64);
                        let counter = Arc::new(AtomicUsize::new(0));
                        let messages_per_producer = MESSAGE_COUNT / producers;
                        
                        let mut handles = Vec::new();
                        
                        // Spawn consumers
                        for _ in 0..consumers {
                            let recv = receiver.clone();
                            let counter_clone = counter.clone();
                            handles.push(thread::spawn(move || {
                                for item in recv {
                                    counter_clone.fetch_add(item, Ordering::Relaxed);
                                }
                            }));
                        }
                        drop(receiver);
                        
                        // Spawn producers
                        for _ in 0..producers {
                            let send = sender.clone();
                            handles.push(thread::spawn(move || {
                                for i in 0..messages_per_producer {
                                    while send.try_send(black_box(i)).is_err() {
                                        thread::yield_now();
                                    }
                                }
                            }));
                        }
                        drop(sender);
                        
                        for handle in handles {
                            handle.join().unwrap();
                        }
                        
                        let expected_sum: usize = (0..messages_per_producer).sum::<usize>() * producers;
                        assert_eq!(counter.load(Ordering::Relaxed), expected_sum);
                    });
                },
            );
        }
    }
    
    group.finish();
}

fn bench_broadcast_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("Broadcast_Queue");
    
    for &stream_count in [1, 2, 4].iter() {
        for &consumers_per_stream in [1, 2].iter() {
            group.throughput(Throughput::Elements(MESSAGE_COUNT as u64 * stream_count as u64));
            group.bench_with_input(
                BenchmarkId::new(
                    "broadcast", 
                    format!("{}s_{}c", stream_count, consumers_per_stream)
                ),
                &(stream_count, consumers_per_stream),
                |b, &(streams, consumers_per_stream)| {
                    b.iter(|| {
                        let (sender, receiver) = multiqueue::broadcast_queue(QUEUE_SIZE as u64);
                        let counter = Arc::new(AtomicUsize::new(0));
                        let mut handles = Vec::new();
                        
                        // Create streams and consumers
                        for _ in 0..streams {
                            let stream = receiver.add_stream();
                            for _ in 0..consumers_per_stream {
                                let consumer = stream.clone();
                                let counter_clone = counter.clone();
                                handles.push(thread::spawn(move || {
                                    for item in consumer {
                                        counter_clone.fetch_add(item, Ordering::Relaxed);
                                    }
                                }));
                            }
                        }
                        receiver.unsubscribe();
                        
                        // Send messages
                        for i in 0..MESSAGE_COUNT {
                            while sender.try_send(black_box(i)).is_err() {
                                thread::yield_now();
                            }
                        }
                        drop(sender);
                        
                        for handle in handles {
                            handle.join().unwrap();
                        }
                        
                        let expected_sum: usize = (0..MESSAGE_COUNT).sum::<usize>() * streams;
                        assert_eq!(counter.load(Ordering::Relaxed), expected_sum);
                    });
                },
            );
        }
    }
    
    group.finish();
}

fn bench_futures_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("Futures_Queue");
    group.throughput(Throughput::Elements(MESSAGE_COUNT as u64));
    
    group.bench_function("futures_mpmc", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                use futures::{SinkExt, StreamExt};
                
                let (mut sender, mut receiver) = multiqueue::mpmc_fut_queue(QUEUE_SIZE as u64);
                let counter = Arc::new(AtomicUsize::new(0));
                let counter_clone = counter.clone();
                
                let consumer_task = tokio::spawn(async move {
                    while let Some(item) = receiver.next().await {
                        counter_clone.fetch_add(item, Ordering::Relaxed);
                    }
                });
                
                let producer_task = tokio::spawn(async move {
                    for i in 0..MESSAGE_COUNT {
                        sender.send(black_box(i)).await.unwrap();
                    }
                });
                
                producer_task.await.unwrap();
                consumer_task.await.unwrap();
                
                assert_eq!(counter.load(Ordering::Relaxed), (0..MESSAGE_COUNT).sum::<usize>());
            });
        });
    });
    
    group.finish();
}

fn bench_throughput_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Throughput_Variations");
    
    for &queue_size in [64, 256, 1024, 4096].iter() {
        group.throughput(Throughput::Elements(MESSAGE_COUNT as u64));
        group.bench_with_input(
            BenchmarkId::new("queue_size", queue_size),
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

criterion_group!(
    benches,
    bench_spsc_mpmc,
    bench_spsc_broadcast,
    bench_mpmc_queue,
    bench_broadcast_queue,
    bench_futures_queue,
    bench_throughput_variations
);
criterion_main!(benches);