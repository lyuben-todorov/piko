#[cfg(test)]
mod tests {
    use crate::semaphore::OrdSemaphore;
    use std::sync::Arc;

    #[test]
    fn test1() {
        let s = OrdSemaphore::new();
        let c = s.create_task(0);
        c.consume();
        s.wait_until(&1);
    }

    #[test]
    fn test2() {
        use std::thread;
        use std::time::Duration;
        let s = Arc::new(OrdSemaphore::new());
        let s_ = s.clone();
        let c0 = s.create_task(0);
        let c1 = s.create_task(1);
        let t0 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            s_.wait_until(&1);
            c1.consume();
        });
        let t1 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            c0.consume();
        });
        s.wait_until(&2);
        t0.join().unwrap();
        t1.join().unwrap();
    }

    #[test]
    fn test3() {
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;
        let s = Arc::new(OrdSemaphore::new());
        let c = s.create_task(0);
        let s_ = s.clone();
        let t0 = thread::spawn(move || {
            s_.wait_until(&1);
        });
        let s_ = s.clone();
        let t1 = thread::spawn(move || {
            s_.wait_until(&1);
        });
        thread::sleep(Duration::from_millis(100));
        c.consume();
        t0.join().unwrap();
        t1.join().unwrap();
    }
}