// For the most part, shamelessly copied from carllerche futures mpsc tests
extern crate futures;
extern crate multiqueue2 as multiqueue;

use futures::future::lazy;
use futures::{SinkExt, StreamExt};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn is_send<T: Send>() {}

#[tokio::test]
async fn bounds() {
    is_send::<multiqueue::BroadcastFutSender<i32>>();
    is_send::<multiqueue::BroadcastFutReceiver<i32>>();
}

#[tokio::test]
async fn send_recv() {
    let (mut tx, rx) = multiqueue::broadcast_fut_queue::<i32>(16);

    tx.send(1).await.unwrap();

    assert_eq!(rx.try_recv().unwrap(), 1);
}

#[tokio::test]
async fn send_shared_recv() {
    let (mut tx1, rx) = multiqueue::broadcast_fut_queue::<i32>(16);
    let mut tx2 = tx1.clone();

    tx1.send(1).await.unwrap();
    assert_eq!(rx.try_recv().unwrap(), 1);

    tx2.send(2).await.unwrap();
    assert_eq!(rx.try_recv().unwrap(), 2);
}

#[tokio::test]
async fn send_recv_threads() {
    let (mut tx, rx) = multiqueue::broadcast_fut_queue::<i32>(16);

    tokio::spawn(async move {
        tx.send(1).await.unwrap();
    })
    .await
    .unwrap();

    assert_eq!(rx.try_recv().unwrap(), 1);
}

#[tokio::test]
async fn send_recv_threads_no_capacity() {
    let (mut tx, mut rx) = multiqueue::broadcast_fut_queue::<i32>(0);

    let t = tokio::spawn(async move {
        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(rx.next().await.unwrap(), 1);

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(rx.next().await.unwrap(), 2);

    t.await.unwrap();
}

#[tokio::test]
async fn recv_close_gets_none() {
    let (tx, rx) = multiqueue::broadcast_fut_queue::<i32>(10);

    // Run on a task context
    lazy(move |_| {
        rx.unsubscribe();

        drop(tx);

        Ok::<(), ()>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn tx_close_gets_none() {
    let (_, rx) = multiqueue::broadcast_fut_queue::<i32>(10);

    // Run on a task context
    lazy(move |_| {
        assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
        assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));

        Ok::<(), ()>(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn stress_shared_bounded_hard() {
    const AMT: u32 = 10000;
    const NTHREADS: u32 = 8;
    let (tx, mut rx) = multiqueue::broadcast_fut_queue::<i32>(0);

    let t = tokio::spawn(async move {
        for _ in 0..AMT * NTHREADS {
            assert_eq!(rx.next().await.unwrap(), 1);
        }

        if rx.next().await.is_some() {
            panic!();
        }
    });

    for _ in 0..NTHREADS {
        let mut tx = tx.clone();

        tokio::spawn(async move {
            for _ in 0..AMT {
                tx.send(1).await.unwrap();
            }
        });
    }

    drop(tx);

    t.await.unwrap();
}

#[tokio::test]
async fn stress_receiver_multi_task_bounded_hard() {
    const AMT: usize = 10_000;
    const NTHREADS: u32 = 2;

    let (mut tx, rx) = multiqueue::broadcast_fut_queue::<usize>(0);
    let rx = Arc::new(Mutex::new(Some(rx)));
    let n = Arc::new(AtomicUsize::new(0));

    let mut th = vec![];

    for _ in 0..NTHREADS {
        let rx = rx.clone();
        let n = n.clone();

        let t = tokio::spawn(async move {
            let mut i = 0;

            loop {
                i += 1;
                let rcv_rx: Option<_> = {
                    let rx = Arc::clone(&rx);
                    let mut lock = rx.lock().ok().unwrap();
                    lock.take()
                };

                match rcv_rx {
                    Some(rcv_rx) => {
                        if i % 5 == 0 {
                            let (item, rest) = rcv_rx.into_future().await;

                            if item.is_none() {
                                break;
                            }

                            n.fetch_add(1, Ordering::Relaxed);
                            {
                                let mut lock = rx.lock().ok().unwrap();
                                *lock = Some(rest);
                            }
                        } else {
                            // Just poll
                            let n = n.clone();
                            let rx = Arc::clone(&rx);
                            let r = lazy(move |_| {
                                let r = match rcv_rx.try_recv() {
                                    Ok(_) => {
                                        n.fetch_add(1, Ordering::Relaxed);
                                        {
                                            let mut lock = rx.lock().ok().unwrap();
                                            *lock = Some(rcv_rx);
                                        }
                                        false
                                    }
                                    Err(TryRecvError::Empty) => true,
                                    Err(TryRecvError::Disconnected) => {
                                        {
                                            let mut lock = rx.lock().ok().unwrap();
                                            *lock = Some(rcv_rx);
                                        }
                                        false
                                    }
                                };

                                Ok::<bool, ()>(r)
                            })
                            .await
                            .unwrap();

                            if r {
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }
        });

        th.push(t);
    }

    for i in 0..AMT {
        tx.send(i).await.unwrap();
    }

    drop(tx);

    for t in th {
        t.await.unwrap();
    }

    assert_eq!(AMT, n.load(Ordering::Relaxed));
}
