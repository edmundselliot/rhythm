use std::{thread, sync::{Arc, Mutex}};
use rhythm::RateLimiter;

/*
Share a rate limiter across multiple threads, using an Arc<Mutex<>>
*/

fn use_limiter_in_shared_way(limiter: Arc<Mutex<RateLimiter<u64>>>) {
    for _ in 0..20 {
        for id in 0..2 {
            let mut limiter_guard = limiter.lock().unwrap();
            if limiter_guard.request(id) {
                println!("Thread {:?} granted request to {}", thread::current().id(), id);
            } else {
                println!("Thread {:?} denied request to {}", thread::current().id(), id);
            }
            drop(limiter_guard);
            thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

fn main() {
    let limiter: Arc<Mutex<RateLimiter<u64>>> = Arc::new(Mutex::new(RateLimiter::new(
        // 4 tokens per bucket
        4,
        // 2 token per refill
        2,
        // 1 refill per second
        std::time::Duration::from_millis(500),
    )));

    let mut threads = Vec::new();
    for _ in 0..2 {
        let limiter_clone = limiter.clone();
        let handle = thread::spawn(move ||
            use_limiter_in_shared_way(limiter_clone)
        );
        threads.push(handle);
    }

    for handle in threads {
        handle.join().unwrap();
    }
}