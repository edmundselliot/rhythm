use tokio::sync::Mutex;
use std::sync::Arc;
use rhythm::RateLimiter;
use tokio::time::Duration;

async fn use_limiter_in_shared_way(limiter: Arc<Mutex<RateLimiter<u64>>>, task_id: u64) {
    for _ in 0..20 {
        for id in 0..2 {
            let mut limiter_guard = limiter.lock().await;
            if limiter_guard.request(id) {
                println!("Task {task_id} granted request to {}", id);
            } else {
                println!("Task {task_id} denied request to {}", id);
            }
            drop(limiter_guard);
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let limiter: Arc<Mutex<RateLimiter<u64>>> = Arc::new(Mutex::new(RateLimiter::new(
        // 4 tokens per bucket
        4,
        // 2 tokens per refill
        1,
        // 5 refill per second
        Duration::from_millis(200),
    )));

    let mut handles = Vec::new();

    for task_id in 0..2 {
        let limiter_clone = limiter.clone();
        let handle = tokio::spawn(async move {
            use_limiter_in_shared_way(limiter_clone, task_id).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}