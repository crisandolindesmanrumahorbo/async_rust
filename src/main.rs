use futures::executor::block_on;
use futures::task::ArcWake;
use futures::task::waker_ref;
use std::pin::pin;
use std::task::Waker;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    thread::{self, sleep},
    time::Duration,
};

fn main() {
    let f = asycn_main();
    block_on(f);
}

async fn asycn_main() {
    let point = Point {
        x: Arc::new(Mutex::new(1)),
    };
    point.await
}

struct Point {
    x: Arc<Mutex<u32>>,
}

impl Future for Point {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let x = self.x.lock().unwrap();
        if *x == 10 {
            println!("Ready");
            Poll::Ready(())
        } else {
            let waker = cx.waker().clone();
            let thread_x = self.x.clone();
            thread::spawn(move || {
                println!("Changing take sometime ...");
                sleep(Duration::new(2, 0));
                let mut x1 = thread_x.lock().unwrap();
                *x1 = 10;

                // wake tell the executor there is progress in future, so poll fn will called again
                waker.wake();
            });
            println!("Pending");
            Poll::Pending
        }
    }
}
