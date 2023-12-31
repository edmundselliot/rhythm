use std::{collections::HashMap, time::{Instant, Duration}, hash::Hash, sync::Mutex};

/// A rate limiter that uses a token bucket algorithm.
/// This rate limiter is thread-safe.
pub struct RateLimiter<T: Hash + Eq> {
    default_bucket_size: u32,
    default_refill_rate: u32,
    default_refill_interval: Duration,
    buckets: Mutex<HashMap<T, Bucket>>,

    // Counters
    counters: Mutex<RateLimiterStats>,
}

struct Bucket {
    // How many tokens can be stored in the bucket
    size: u32,
    // How many tokens are currently in the bucket
    tokens: u32,
    // How many tokens to add per refill
    refill_rate: u32,
    // How often to refill the bucket
    refill_interval: Duration,
    // When the bucket was last refilled
    last_filled: Instant,

    // Counters
    counters: RateLimiterStats,
}

struct RateLimiterStats {
    requests_allowed:u64,
    requests_denied:u64,
}

impl<T: Hash + Eq> RateLimiter<T> {
    /// Create a new RateLimiter with the given default values.
    /// Note: Whatever type is chosen for the key must implement the Hash and Eq traits.
    ///
    /// # Arguments
    ///
    /// * `default_bucket_size` - The default bucket size.
    /// * `default_refill_rate` - The default refill rate (how many tokens will be added to each bucket on each refill).
    /// * `default_refill_interval` - The default refill interval (how often the bucket will be refilled).
    ///
    /// # Returns
    ///
    ///    A new RateLimiter with the given default values.
    pub fn new(default_bucket_size: u32, default_refill_rate:u32, default_refill_interval:Duration) -> Self {
        Self {
            default_bucket_size: default_bucket_size,
            default_refill_rate: default_refill_rate,
            default_refill_interval: default_refill_interval,
            buckets: Mutex::new(HashMap::new()),
            counters: Mutex::new(RateLimiterStats {
                requests_allowed: 0,
                requests_denied: 0,
            }),
        }
    }

    /// Process a request for a given key, including decrementing the current token count.
    /// This function also deals with refilling the bucket.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to process a request for
    ///
    /// # Returns
    ///
    /// * `true` if the request is allowed
    /// * `false` if the request is not allowed
    pub fn request(&mut self, key: T) -> bool {
        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets.entry(key).or_insert(Bucket {
            size: self.default_bucket_size,
            // Start the bucket full
            tokens: self.default_bucket_size,
            refill_rate: self.default_refill_rate,
            refill_interval: self.default_refill_interval,
            last_filled: Instant::now(),
            counters: RateLimiterStats {
                requests_allowed: 0,
                requests_denied: 0,
            },
        });

        // If required, we can put a mutex<arc> on each bucket to
        //    avoid holding table-level lock when doing a request
        let success = bucket.request();
        drop(buckets);

        let mut counters = self.counters.lock().unwrap();
        if success {
            counters.requests_allowed += 1;
        } else {
            counters.requests_denied += 1;
        }
        drop(counters);

        success
    }

    /// Set the bucket size, refill rate, and refill interval for a given key.
    /// If the VIP key already exists, it will be updated and refilled.
    /// If the VIP key does not exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to the VIP we are making a special case for.
    /// * `bucket_size` - The size of the VIP's bucket.
    /// * `refill_rate` - The refill rate of the VIP's bucket.
    pub fn set_vip(&mut self, key: T, bucket_size: u32, refill_rate: u32) {
        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets.entry(key).or_insert(Bucket {
            size: bucket_size,
            // Start the bucket full
            tokens: bucket_size,
            refill_rate: refill_rate,
            refill_interval: self.default_refill_interval,
            last_filled: Instant::now(),
            counters: RateLimiterStats {
                requests_allowed: 0,
                requests_denied: 0,
            },
        });

        bucket.last_filled = Instant::now();
        bucket.tokens = bucket.size;
    }

    /// Prune buckets that have not been used in the last `age` duration.
    ///
    /// # Arguments
    ///
    /// * `age` - The age of buckets to prune.
    ///
    /// # Example
    ///
    /// ```
    /// use std::time::Duration;
    /// use rhythm::RateLimiter;
    ///
    /// let mut rl: RateLimiter<String> = RateLimiter::new(10, 1, Duration::from_secs(1));
    ///
    /// // Do some work here...
    ///
    /// // Prune buckets that have not been used in the last 5 seconds
    /// rl.prune(Duration::from_secs(5));
    /// ```
    pub fn prune(&self, age: Duration) {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap();
        buckets.retain(|_, bucket| now.duration_since(bucket.last_filled) <= age);
    }

}

impl Bucket {
    fn refill(&mut self) {
        let now = Instant::now();

        let time_passed = now.duration_since(self.last_filled);
        let intervals_passed = time_passed.as_nanos() / self.refill_interval.as_nanos();

        // Add the tokens to the bucket, but don't exceed the bucket size
        let tokens_to_add = intervals_passed * self.refill_rate as u128;
        self.tokens = (self.tokens + tokens_to_add as u32).min(self.size);

        // --- Setting last filled ---
        // Q: Why not just set the last_filled to now?
        // A: Because of integer division, we would often under-fill the bucket. If we update the last_filled
        //      time naively, then multiple calls to refill() in a row will result in the bucket never being refilled.

        // Note: commented line below leads to issues because over a long enough time, because last_filled
        //         will drift if we only set it relative to itself.
        // >> self.last_filled = self.last_filled + Duration::from_nanos((intervals_passed * self.refill_interval.as_nanos()) as u64);

        // Instead, we need to set it relative to the current time.
        let remainder_nanos = time_passed.as_nanos() % self.refill_interval.as_nanos();
        self.last_filled = now - Duration::from_nanos(remainder_nanos as u64);
    }

    fn request(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            self.counters.requests_allowed += 1;
            true
        } else {
            self.counters.requests_denied += 1;
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_slow_limiter() {
        test_limiter(10, 1, Duration::from_secs(1));
    }

    #[test]
    fn test_fast_limiter() {
        test_limiter(10, 1, Duration::from_millis(1));
    }

    #[test]
    fn test_fast_big_limiter() {
        test_limiter(1000000, 1, Duration::from_millis(1));
    }

    #[test]
    fn test_big_refill_limiter() {
        test_limiter(100, 100, Duration::from_millis(10));
    }

    fn test_limiter(bucketsize: u32, refillrate: u32, refill_interval: Duration) {
        // Rate limiter with default bucket size of 10 and refill rate of 1 token per 100 ms
        let mut ratelimiter: RateLimiter<String> = RateLimiter::new(
            bucketsize,
            refillrate,
            refill_interval
        );

        // Test that we can make a bunch of requests in a row
        let start_time = Instant::now();
        let sample_key = "Erin".to_string();
        for _ in 0..bucketsize {
            assert!(ratelimiter.request(sample_key.clone()));
        }

        // Test that we can't make an extra request
        if start_time.elapsed() < refill_interval {
            assert!(!ratelimiter.request(sample_key.clone()));
        }

        // Test that an unrelated key is not affected
        let unrelated_key = "Coconut".to_string();
        for _ in 0..bucketsize {
            assert!(ratelimiter.request(unrelated_key.clone()));
        }

        // Exhaust the bucket
        while ratelimiter.request(sample_key.clone()) {
            continue;
        }

        // Test that we can make an extra request after for bucket refill
        let start_time = Instant::now();
        std::thread::sleep(refill_interval);
        let refills_expected = start_time.elapsed().as_nanos() / refill_interval.as_nanos();
        let tokens_expected: u128 = (bucketsize as u128).min(refills_expected * refillrate as u128);

        for _ in 0..tokens_expected {
            assert!(ratelimiter.request(sample_key.clone()));
        }
    }

    #[test]
    fn test_vip() {
        // Rate limiter with default bucket size of 10 and refill rate of 1 token per second
        let mut ratelimiter: RateLimiter<String> = RateLimiter::new(10, 1, Duration::from_secs(1));

        let normal_key = "Elliot".to_string();

        // There's someone super important who needs to make 100 requests in a row, let them do it.
        let vip_bucket_size = 100;
        let vip_refill_rate = 10;
        let vip_key = "Waffle".to_string();

        ratelimiter.set_vip(vip_key.clone(), vip_bucket_size, vip_refill_rate);

        for i in 0..vip_bucket_size {
            // VIP should be ok the entire time
            assert!(ratelimiter.request(vip_key.clone()));
            // The other requesters... not so much
            if i < 10 {
                assert!(ratelimiter.request(normal_key.clone()));
            } else {
                assert!(!ratelimiter.request(normal_key.clone()));
            }
        }
    }

    #[tokio::test]
    async fn test_limiter_async() {
        let mut ratelimiter: RateLimiter<String> = RateLimiter::new(10, 1, Duration::from_secs(1));
        let key = "Honey".to_string();

        for _ in 0..10 {
            assert!(ratelimiter.request(key.clone()));
        }
        assert!(!ratelimiter.request(key.clone()));
    }

    #[test]
    fn test_prune() {
        let mut ratelimiter: RateLimiter<String> = RateLimiter::new(10, 1, Duration::from_secs(1));

        ratelimiter.request("Lillo".to_string());

        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(ratelimiter.buckets.lock().unwrap().len(), 1);

        ratelimiter.request("Dawn".to_string());
        ratelimiter.prune(Duration::from_millis(10));
        assert_eq!(ratelimiter.buckets.lock().unwrap().len(), 1);

        std::thread::sleep(Duration::from_millis(10));
        ratelimiter.prune(Duration::from_millis(10));
        assert_eq!(ratelimiter.buckets.lock().unwrap().len(), 0);
    }
}
