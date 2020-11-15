use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

/// The goal of this data structure is to provide an explicit ordering of async events in a multi-threaded
/// process. It holds a sequence of event handles with a defined total ordering between them. A
/// thread can then call `wait_until` and will be blocked until all the event handles before it
/// are resolved.
pub struct OrdSemaphore<T: Ord> {
    events: Mutex<BinaryHeap<Reverse<Waiter<T>>>>,
}

impl<T: Ord> OrdSemaphore<T> {
    pub fn new() -> OrdSemaphore<T> {
        OrdSemaphore {
            events: Mutex::new(BinaryHeap::new()),
        }
    }

    pub fn create_task(&self, task: T) -> Client {
        let (client, waiter) = create_pair(task);
        let mut queue = self.events.lock().unwrap();
        queue.push(Reverse(waiter));

        client
    }

    /// Blocks the caller until all tasks before `event` have been completed.
    pub fn wait_until(&self, event: &T) {
        loop {
            {
                let mut queue = self.events.lock().unwrap();
                loop {
                    match queue.peek() {
                        None => return,
                        Some(w) if w.0.event >= *event => return,
                        Some(w) if w.0.watcher.is_consumed() => drop(queue.pop()),
                        Some(w) => break w.0.watcher.clone(),
                    }
                }
            }
                .wait_until_consumed()
        }
    }
}

fn create_pair<T: Ord>(event: T) -> (Client, Waiter<T>) {
    let (client, watcher) = Client::new();
    let waiter = Waiter::new(event, watcher);

    (client, waiter)
}

///
/// The `Waiter` stays inside our semaphore.
struct Waiter<T: Ord + Eq + PartialEq> {
    event: T,
    watcher: Watcher,
}

impl<T: Ord> Waiter<T> {
    fn new(event: T, watcher: Watcher) -> Waiter<T> {
        Waiter { event, watcher }
    }
}

impl<T: Ord> Ord for Waiter<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.event.cmp(&other.event)
    }
}

impl<T: Ord> PartialOrd for Waiter<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.event.cmp(&other.event))
    }
}

impl<T: Ord> PartialEq for Waiter<T> {
    fn eq(&self, other: &Self) -> bool {
        self.event.eq(&other.event)
    }
}

impl<T: Ord> Eq for Waiter<T> {}

use std::sync::{
    atomic::{self, AtomicBool},
    Arc, Condvar, Mutex,
};

///
/// The `Client` is given to the creator of the event and belongs to the execution context.
///
#[derive(Clone)]
pub struct Client(Arc<Inner>);
struct Inner {
    done: AtomicBool,
    done_mutex: Mutex<bool>,
    cv: Condvar,
}

impl Client {
    /// Indicates the client is done.
    /// Only the first call to [`consume`][`Client::consume`] will have any effect.
    /// Subsequent calls or calls on this [`Client`] or any of its clones are allowed but ineffective.
    pub fn consume(&self) {
        if !(self.0.done.load(atomic::Ordering::Relaxed)) {
            self.0.done.store(true, atomic::Ordering::Release);
            *self.0.done_mutex.lock().unwrap() = true;
            self.0.cv.notify_all();
        }
    }

    fn new() -> (Self, Watcher) {
        let i = Inner {
            done: false.into(),
            done_mutex: Mutex::new(false),
            cv: Condvar::new(),
        };
        let client = Client(Arc::new(i));
        let watcher = Watcher(client.0.clone());
        (client, watcher)
    }
}

#[derive(Clone)]
struct Watcher(Arc<Inner>);
impl Watcher {
    fn is_consumed(&self) -> bool {
        self.0.done.load(atomic::Ordering::Acquire)
    }
    fn wait_until_consumed(self) {
        if !self.is_consumed() {
            drop(
                self.0
                    .cv
                    .wait_while(self.0.done_mutex.lock().unwrap(), |&mut done| !done)
                    .unwrap(),
            )
        }
    }
}
