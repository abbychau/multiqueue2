use multiqueue2 as multiqueue;

use crossbeam::scope;
use multiqueue::broadcast_queue;

fn spsc_example() {
    let (send, recv) = broadcast_queue(4);
    scope(|scope| {
        scope.spawn(move |_| {
            for val in recv {
                println!("Got {}", val);
            }
        });

        for i in 0..10 {
            // Don't do this busy loop in real stuff unless you're really sure
            loop {
                if send.try_send(i).is_ok() {
                    break;
                }
            }
        }
        drop(send);
    })
    .unwrap();
}

fn spsc_bcast_example() {
    let (send, recv) = broadcast_queue(4);
    scope(|scope| {
        for i in 0..2 {
            // or n
            let cur_recv = recv.add_stream();
            for j in 0..2 {
                let stream_consumer = cur_recv.clone();
                scope.spawn(move |_| {
                    for val in stream_consumer {
                        println!("Stream {} consumer {} got {}", i, j, val);
                    }
                });
            }
        }

        // Take notice that I drop the reader - this removes it from
        // the queue, meaning that the readers in the new threads
        // won't get starved by the lack of progress from recv
        recv.unsubscribe();

        for i in 0..10 {
            // Don't do this busy loop in real stuff unless you're really sure
            loop {
                if send.try_send(i).is_ok() {
                    break;
                }
            }
        }
        drop(send);
    })
    .unwrap();
}

fn spmc_bcast_example() {
    let (send, recv) = broadcast_queue(4);
    scope(|scope| {
        for i in 0..2 {
            let cur_recv = recv.add_stream();
            for j in 0..2 {
                let stream_consumer = cur_recv.clone();
                scope.spawn(move |_| {
                    for val in stream_consumer {
                        println!("Stream {} consumer {} got {}", i, j, val);
                    }
                });
            }
            // cur_recv is dropped here
        }

        // Take notice that I drop the reader - this removes it from
        // the queue, meaning that the readers in the new threads
        // won't get starved by the lack of progress from recv
        recv.unsubscribe();

        for i in 0..10 {
            // Don't do this busy loop in real stuff unless you're really sure
            loop {
                if send.try_send(i).is_ok() {
                    break;
                }
            }
        }
        drop(send);
    })
    .unwrap();
}

fn wacky_example() {
    let (send, recv) = broadcast_queue(4);
    scope(|scope| {
        for i in 0..2 {
            let cur_recv = recv.add_stream();
            for j in 0..2 {
                let stream_consumer = cur_recv.clone();
                scope.spawn(move |_| {
                    for val in stream_consumer {
                        println!("Stream {} consumer {} got {}", i, j, val);
                    }
                });
            }
            // cur_recv is dropped here
        }

        // On this stream, since there's only one consumer,
        // the receiver can be made into a SingleReceiver
        // which can view items inline in the queue
        let single_recv = recv.add_stream().into_single().unwrap();

        scope.spawn(move |_| {
            for val in single_recv.iter_with(|item_ref| 10 * *item_ref) {
                println!("{}", val);
            }
        });

        // Same as above, except this time we just want to iterate until the receiver is empty
        let single_recv_2 = recv.add_stream().into_single().unwrap();

        scope.spawn(move |_| {
            for val in single_recv_2.try_iter_with(|item_ref| 10 * *item_ref) {
                println!("{}", val);
            }
        });

        // Take notice that I drop the reader - this removes it from
        // the queue, meaning that the readers in the new threads
        // won't get starved by the lack of progress from recv
        recv.unsubscribe();

        for _ in 0..3 {
            for i in 0..10 {
                // Don't do this busy loop in real stuff unless you're really sure
                loop {
                    if send.try_send(i).is_ok() {
                        break;
                    }
                }
            }
        }
        drop(send);
    })
    .unwrap();
}

fn main() {
    println!("SPSC example");
    spsc_example();
    println!("\n\nSPSC Broadcast example");
    spsc_bcast_example();
    println!("\n\nSPMC Broadcast example");
    spmc_bcast_example();
    println!("\n\nWacky example");
    wacky_example();
}
