use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    task::{Context, Poll, Wake, Waker},
    thread,
    time::Duration,
};

fn main() {
    let executor = Executor::new();
    executor.spawn(async_main());

    // executor.spawn(async_());

    // executor.spawn(async_main());
    // executor.spawn(async_main1());
    executor.run();
}

// async fn async_() {
//     async_main().await;
//     async_main1().await;
// }

async fn async_main() {
    println!("Task started...");
    let point = Point {
        x: Arc::new(Mutex::new(1)),
    };
    point.await;
    println!("Task finished after 2 seconds!");
}

async fn async_main1() {
    println!("Task started...");
    let point = Point {
        x: Arc::new(Mutex::new(1)),
    };
    point.await;
    println!("Task finished after 2 seconds!");
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
                println!("Changing takes some time...");
                thread::sleep(Duration::new(5, 0));
                let mut x1 = thread_x.lock().unwrap();
                *x1 = 10;
                println!("Value updated, waking task...");
                waker.wake();
            });
            println!("Pending...");
            Poll::Pending
        }
    }
}

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    task_queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
    wake_sender: Sender<()>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        println!("Task Wake called");

        println!("Before attempting to send wake signal...");
        if let Err(e) = self.wake_sender.send(()) {
            println!("Failed to send wake signal: {:?}", e);
        }

        println!("Attempting to add task to queue...");
        self.task_queue.lock().unwrap().push_back(self.clone());
        println!("Task added to queue");
    }
}

struct Executor {
    task_queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
    wake_receiver: Receiver<()>,
    wake_sender: Sender<()>,
    active_tasks: Arc<AtomicUsize>, // Counter for pending tasks
}

impl Executor {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            wake_receiver: rx,
            wake_sender: tx,
            active_tasks: Arc::new(AtomicUsize::new(0)), // Start with 0 tasks
        }
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.active_tasks.fetch_add(1, Ordering::SeqCst); // Increase task count

        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            task_queue: self.task_queue.clone(),
            wake_sender: self.wake_sender.clone(),
        });

        self.task_queue.lock().unwrap().push_back(task);
    }

    fn run(&self) {
        loop {
            while let Some(task) = self.task_queue.lock().unwrap().pop_front() {
                println!("Processing task...");
                let waker = Waker::from(task.clone());
                let mut context = Context::from_waker(&waker);

                let mut future = task.future.lock().unwrap();
                if let Poll::Ready(()) = future.as_mut().poll(&mut context) {
                    println!("Task completed!");
                    self.active_tasks.fetch_sub(1, Ordering::SeqCst); // Decrease task count
                } else {
                    println!("Task is still pending...");
                }
            }

            // Exit only when both queue is empty AND no active tasks remain
            if self.task_queue.lock().unwrap().is_empty()
                && self.active_tasks.load(Ordering::SeqCst) == 0
            {
                println!("All tasks completed, executor exiting...");
                break;
            }

            // Wait for a wake signal
            println!(
                "Waiting for wake signal... ({} active tasks remaining)",
                self.active_tasks.load(Ordering::SeqCst)
            );
            if self.wake_receiver.recv().is_err() {
                println!("Wake channel closed, exiting...");
                break;
            }
            println!("Received wake signal!");
        }
    }
}
