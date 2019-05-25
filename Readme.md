# MultiQueue2: Fast MPMC Broadcast Queue

[![Build Status](https://travis-ci.org/abbychau/multiqueue2.svg?branch=master)](https://travis-ci.org/abbychau/multiqueue2)


MultiQueue2 is a fast bounded mpmc queue that supports broadcast/broadcast style operations 

[MultiQueue](https://github.com/schets/multiqueue) was developed by Sam Schetterer, but not updated for some time. I found it very useful as it implements `futures`. However, it is with a few outdated library API and the use of spin locks is taking 100% CPU in many cases. 

## What's new in MultiQueue2

This version tries to fix these. By default, it is now using a condvar block. For `_fut_` async channels, all items are parked quickly without initial spin locks.

The use of this queue is virtually lockless but technically and strictly speaking not.
There are three kinds of lock:
1. Spin with `std::thread::yield_now`
2. Busy Spin
3. Condvar blocking

`2` is the fastest but it will take up 100% cpu. The default setting of MultiQueue2 including `_fut` channels are using Condvar blocks, this will pay about 20ns overhead. However, practically, it would not be the bottleneck of the application. One use case that could be better to change to `2` would be audio and video conversion.

All dependencies are upgraded and all warnings are fixed and upgraded to 2018.



TOC: [Overview](#over) | [Examples](#examples) | [MPMC Mode](#mpmc) | [Futures Mode](#futures) | [Benchmarks](#bench) | [FAQ](#faq)

## <a name = "over">Overview</a>


Multiqueue is based on the queue design from the LMAX Disruptor, with a few improvements:
  * futures stream/sink (implemented `futures` traits)
  * It can dynamically add/remove producers, and each [stream](#model) can have multiple consumers
  * It has fast fallbacks for whenever there's a single consumer and/or a single producer and can detect switches at runtime
  * It works on 32 bit systems without any performance or capability penalty
  * In most cases, one can view data written directly into the queue without copying it

One can think of MultiQueue as a sort of [souped up channel/sync_channel](#bench),
with the additional ability to have multiple independent consumers each receiving the same [stream](#model) of data.


Reasons to choose MultiQueue2 over the built-in channels:

  * supports broadcasting elements to multiple readers with a single push into the queue
  * allows reading elements in-place in the queue in most cases, so you can broadcast elements without lots of copying
  * can act as a futures stream and sink
  * does not allocate on push/pop unlike channel, leading to much more predictable latencies
  * is virtually lockless unlike sync_channel, and fares decently under contention

Reasons NOT to choose MultiQueue2 over the built-in channels:

  * Truly want an unbounded queue, although you should probably handle backlog instead
  * Need senders to block when the queue is full and can't use the futures api
  * Don't want the memory usage of a large buffer
  * You need a oneshot queue
  * You very frequently add/remove producers/consumers

Otherwise, in most cases, MultiQueue should be a good replacement for channels.
In general, this will function very well as normal bounded queue with performance
approaching that of hand-written queues for single/multiple consumers/producers

**even without taking advantage of the broadcast**

## <a name = "examples">Examples</a>

### Single-producer single-stream

This is about as simple as it gets for a queue. Fast, one writer, one reader, simple to use.
```rust
extern crate multiqueue2 as multiqueue;

use std::thread;

let (send, recv) = multiqueue::mpmc_queue(10);

thread::spawn(move || {
    for val in recv {
        println!("Got {}", val);
    }
});

for i in 0..10 {
    send.try_send(i).unwrap();
}

// Drop the sender to close the queue
drop(send);

// prints
// Got 0
// Got 1
// Got 2
// etc


// some join mechanics here
```

### Single-producer double stream.

Let's send the values to two different streams
```rust
extern crate multiqueue2 as multiqueue;

use std::thread;

let (send, recv) = multiqueue::broadcast_queue(4);

for i in 0..2 { // or n
    let cur_recv = recv.add_stream();
    thread::spawn(move || {
        for val in cur_recv {
            println!("Stream {} got {}", i, val);
        }
    });
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

// Drop the sender to close the queue
drop(send);

// prints along the lines of
// Stream 0 got 0
// Stream 0 got 1
// Stream 1 got 0
// Stream 0 got 2
// Stream 1 got 1
// etc

// some join mechanics here
```

### Single-producer broadcast, 2 consumers per stream
Let's take the above and make each stream consumed by two consumers
```rust
extern crate multiqueue2 as multiqueue;

use std::thread;

let (send, recv) = multiqueue::broadcast_queue(4);

for i in 0..2 { // or n
    let cur_recv = recv.add_stream();
    for j in 0..2 {
        let stream_consumer = cur_recv.clone();
        thread::spawn(move || {
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

// prints along the lines of
// Stream 0 consumer 1 got 2
// Stream 0 consumer 0 got 0
// Stream 1 consumer 0 got 0
// Stream 0 consumer 1 got 1
// Stream 1 consumer 1 got 1
// Stream 1 consumer 0 got 2
// etc

// some join mechanics here
```

### Something wacky
Has anyone really been far even as decided to use even go want to do look more like?

```rust
extern crate multiqueue2 as multiqueue;

use std::thread;

let (send, recv) = multiqueue::broadcast_queue(4);

// start like before
for i in 0..2 { // or n
    let cur_recv = recv.add_stream();
    for j in 0..2 {
        let stream_consumer = cur_recv.clone();
        thread::spawn(move || {
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

thread::spawn(move || {
    for val in single_recv.iter_with(|item_ref| 10 * *item_ref) {
        println!("{}", val);
    }
});

// Same as above, except this time we just want to iterate until the receiver is empty
let single_recv_2 = recv.add_stream().into_single().unwrap();

thread::spawn(move || {
    for val in single_recv_2.try_iter_with(|item_ref| 10 * *item_ref) {
        println!("{}", val);
    }
});

// Take notice that I drop the reader - this removes it from
// the queue, meaning that the readers in the new threads
// won't get starved by the lack of progress from recv
recv.unsubscribe();

// Many senders to give all the receivers something
for _ in 0..3 {
    let cur_send = send.clone();
    for i in 0..10 {
        thread::spawn(loop {
            if cur_send.try_send(i).is_ok() {
                break;
            }
        });
    }
}
drop(send);
```

## <a name = "mpmc">MPMC Mode</a>
One might notice that the broadcast queue modes requires that a type be `Clone`,
and the single-reader inplace variants require that a type be `Sync` as well.
This is only required for broadcast queues and not normal mpmc queues,
so there's an mpmc api as well. 

Multiqueue2 doesn't require that a type be `Clone` or `Sync` for any api, 
and also moves items directly out of the queue instead of cloning them.
There's basically no api difference aside from that, so I'm not going to have a huge
section on them.

## <a name = "futures">Futures Mode</a>
For both mpmc and broadcast, a futures mode is supported. The data-structures are quite
similar to the normal ones, except they implement the `Futures` `Sink`/`Stream` traits for
senders and receivers. This comes at a bit of a performance cost, which is why the
futures types are separated.


## <a name = "bench">Benchmarks</a>

### Throughput

The throughput is benchmarked using the condvar blocking locks, which is the default setting of the queue system. This ensures economical CPU usage even for long blocking async items.

Switching to busy spinlock can provide another 30% throughput boost.

SPSC:
`Time spent doing 10000000 push/pop pairs for 1p::1c was 292.9397618 ns per item`

SPMC:
`Time spent doing 10000000 push/pop pairs for 1p::1c_2b was 310.12774815 ns per item`
`Time spent doing 10000000 push/pop pairs for 1p::1c_3b was 317.77275306666667 ns per item`

MPSC:
`Time spent doing 10000000 push/pop pairs for 2p::1c was 378.5664167 ns per item`

MPMC:
`Time spent doing 10000000 push/pop pairs for 2p::1c_2b was 377.69721405 ns per item`
`Time spent doing 10000000 push/pop pairs for 2p::1c_3b was 414.59893453333336 ns per item`

On MacBook Pro 2018 i7, 16GB Ram.

Here is no latency benchmark tool, but latencies will be approximately the
inter core communication delay, about 40-70 ns on a single socket machine.

These will be higher with multiple producers and multiple consumers, 
since each one must perform an RMW before finishing a write or read.

## <a name = "faq">FAQ</a>

#### My type isn't Clone, can I use the queue?
You can use the MPMC portions of the queue, but you can't broadcast anything

#### Why can't senders block even though readers can?

It's sensible for a reader to block if there is truly nothing for it to do, while the equivalent
isn't true for senders. 

If a sender blocks, that means that the system is backlogged and something else has to consure the stacked up items.

Furthermore, it puts more of a performance penalty on the queue and the latency hit for notifying senders comes before the queue action is finished, while notifying readers happens after the value has sent.

#### Why can the futures sender park even though senders can't block?
It's required for futures api to work sensibly, since when futures can't send into the queue
it expects that the task will be parked and awoken by some other process (if this is wrong, please let me know!).
That makes sense as well since other events will be handled during that time instead of plain blocking.
I'm probably going to add a futures api that just spins on the queue for people who want the niceness of
the futures api but don't want the performance hit.

#### I want to know which stream is the farthest behind when there's backlog, can I do that?
As of now, that's not possible to do. In general, that sort of question is difficult to concretely
answer because any attempt to answer it will be racing against writer updates, and there's also no way
to transform the idea of 'which stream is behind' into something actionable by a program.

#### Is it possible to select from a set of MultiQueues?
No, it is not currently. All items in the queue are not shared so to have a good performance.

#### What happens if consumers of one stream fall behind?
The queue won't overwrite a datapoint until all streams have advanced past it,
so writes to the queue would fail. Depending on your goals, this is either a good or a bad thing.
On one hand, nobody likes getting blocked/starved of updates because of some dumb slow thread.
On the other hand, this basically enforces a sort of system-wide backlog control. If you want
an example why that's needed, NYSE occasionally does not keep the consolidated feed
up to date with the individual feeds and markets fall into disarray.

