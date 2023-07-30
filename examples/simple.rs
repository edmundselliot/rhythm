use rhythm::RateLimiter;

/*
Showcases rate limiting with a simple example.

Output:

    Request 0 granted
    Request 1 granted
    Request 2 granted
    Request 3 granted
    Request 4 denied
    Request 5 granted
    Request 6 denied
    Request 7 denied
    Request 8 denied
    Request 9 denied
    Request 10 granted
    Request 11 denied
    Request 12 denied
    Request 13 denied
    Request 14 denied
    Request 15 granted
    Request 16 denied
    Request 17 denied
    Request 18 denied
    Request 19 denied
*/

fn main() {
    let mut limiter = RateLimiter::new(
        // 10 tokens per bucket
        4,
        // 1 token per refill
        1,
        // 1 refill per second
        std::time::Duration::from_secs(1),
    );

    for i in 0..20 {
        if limiter.request("user") {
            println!("Request {i} granted");
        } else {
            println!("Request {i} denied");
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}