extern crate crossbeam;
extern crate multiqueue2 as multiqueue;
extern crate time;

use crate::multiqueue::{broadcast_queue_with, wait, BroadcastReceiver, BroadcastSender};

use time::precise_time_ns;

use crossbeam::scope;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Barrier;

#[inline(never)]
fn recv(barrier: &Barrier, mreader: BroadcastReceiver<u64>, sum: &AtomicUsize, check: bool) {
    let reader = mreader.into_single().unwrap();
    barrier.wait();
    let start = precise_time_ns();
    let mut cur = 0;
    while let Ok(pushed) = reader.recv() {
        if cur != pushed && check {
            panic!("Got {}, expected {}", pushed, cur);
        }
        cur += 1;
    }

    sum.fetch_add((precise_time_ns() - start) as usize, Ordering::SeqCst);
}

fn send(barrier: &Barrier, writer: BroadcastSender<u64>, num_push: usize) {
    barrier.wait();
    for i in 0..num_push as u64 {
        loop {
            let topush = i;
            if writer.try_send(topush).is_ok() {
                break;
            }
        }
    }
}

fn runit(name: &str, n_senders: usize, n_readers: usize) {
    let num_do = 100_000_000;
    let (writer, reader) = broadcast_queue_with(20000, wait::BusyWait::new());
    let barrier = Barrier::new(1 + n_senders + n_readers);
    let bref = &barrier;
    let ns_atomic = AtomicUsize::new(0);
    scope(|scope| {
        for _ in 0..n_senders {
            let w = writer.clone();
            scope.spawn(move |_| {
                send(bref, w, num_do);
            });
        }
        writer.unsubscribe();
        for _ in 0..n_readers {
            let aref = &ns_atomic;
            let r = reader.add_stream();
            let check = n_senders == 1;
            scope.spawn(move |_| {
                recv(bref, r, aref, check);
            });
        }
        reader.unsubscribe();
        barrier.wait();
    }).unwrap();
    let ns_spent = (ns_atomic.load(Ordering::Relaxed) as f64) / n_readers as f64;
    let ns_per_item = ns_spent / (num_do as f64);
    println!(
        "Time spent doing {} push/pop pairs for {} was {} ns per item",
        num_do, name, ns_per_item
    );
}

fn main() {
    runit("1p::1c", 1, 1);
    runit("1p::1c_2b", 1, 2);
    runit("1p::1c_3b", 1, 3);
    runit("2p::1c", 2, 1);
    runit("2p::1c_2b", 2, 2);
    runit("2p::1c_3b", 2, 3);
}
