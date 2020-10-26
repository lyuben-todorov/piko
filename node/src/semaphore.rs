use std::collections::{BinaryHeap};
use std::sync::RwLock;
use std::cmp::{Ordering, Reverse};
use crossbeam_channel::{bounded, Sender, Receiver, RecvError};
use std::ptr;

/// The goal of this data structure is to provide an explicit ordering of async events in a multi-threaded
/// process. It holds a sequence of event handles with a defined total ordering between them. A
/// thread can then call `wait_until` and will be blocked until all the event handles before it
/// are resolved.
pub struct OrdSemaphore<T: Ord> {
    events: RwLock<BinaryHeap<Reverse<Waiter<T>>>>,
}

impl<T: Ord> OrdSemaphore<T> {
    pub fn new() -> OrdSemaphore<T> {
        OrdSemaphore { events: RwLock::new(BinaryHeap::new()) }
    }

    pub fn create_task(&self, task: T) -> Client {
        let (client, waiter) = create_pair(task);
        let mut queue = self.events.write().unwrap();
        queue.push(Reverse(waiter));

        client
    }

    /// Blocks the caller until all tasks before `event` have been completed.
    pub fn wait_until(&self, event: &T) {
        let mut depth: u32 = 0;
        loop {
            let queue = self.events.read().unwrap();
            match queue.peek() {
                None => {
                    // explicit return, queue is empty
                    return;
                }
                Some(task) => {
                    // if event happened after latest task
                    if event.gt(&task.0.event) {
                        // clone channel and drop read lock
                        let channel = task.0.channel.clone();

                        // get raw pointer
                        let pointer = task as *const Reverse<Waiter<T>>;
                        drop(queue);

                        // this blocks until the respective Client has released
                        let  block = channel.recv();

                        match block {
                            Ok(_)=>{
                                // at this point, we want to do a queue.pop(), however we should prevent
                                // multiple threads from removing multiple elements
                                let mut queue = self.events.write().unwrap();
                                // comparing raw pointers probably isn't the best idea.
                                let before = queue.len();
                                queue.retain(|x| !ptr::eq(x, pointer));
                                let after = queue.len();
                                println!("{} {}", before, after);
                                depth+=1;
                            }
                            Err(_) => {
                                return;
                            }
                        }

                    } else {
                        // explicit return, no backlog left
                        return;
                    }
                }
            }
        }
    }
}

fn create_pair<T: Ord>(event: T) -> (Client, Waiter<T>) {
    let (tx, rx): (Sender<()>, Receiver<()>) = bounded(1);

    let client = Client::new(tx);
    let waiter = Waiter::new(event, rx);

    (client, waiter)
}

///
/// The `Client` is given to the creator of the event and belongs to the execution context. 
/// 
pub struct Client {
    channel: Sender<()>
}

impl Client {
    pub fn new(tx: Sender<()>) -> Client {
        Client { channel: tx }
    }
    pub fn consume(&self) {
        self.channel.send(()).unwrap();
    }
}

///
/// The `Waiter` stays inside our semaphore.
pub struct Waiter<T: Ord + Eq + PartialEq> {
    event: T,
    channel: Receiver<()>,
}

impl<T: Ord> Waiter<T> {
    pub fn new(event: T, rx: Receiver<()>) -> Waiter<T> {
        Waiter { event, channel: rx }
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

