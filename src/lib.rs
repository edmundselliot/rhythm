use std::{collections::HashMap, time::{Instant, Duration}, hash::Hash};

struct RateLimiter<T: Hash + Eq> {
    default_bucket_size: u32,
    default_refill_rate: u32,
    default_refill_interval_ms: u64,
    buckets: HashMap<T, Bucket>,
}

struct Bucket {
    // How many tokens can be stored in the bucket
    size: u32,
    // How many tokens are currently in the bucket
    tokens: u32,
    // How many tokens to add per refill
    refill_rate: u32,
    // How often to refill the bucket
    refill_interval_ms: u64,
    // When the bucket was last refilled
    last_filled: Instant,
}

impl<T: Hash + Eq> RateLimiter<T> {
    /// Create a new RateLimiter with the given default values.
    ///
    /// # Arguments
    ///
    /// * `default_bucket_size` - The default bucket size.
    /// * `default_refill_rate` - The default refill rate (how many tokens will be added to each bucket on each refill).
    /// * `default_refill_interval_ms` - The default refill interval (how often the bucket will be refilled).
    ///
    /// # Returns
    ///
    ///    A new RateLimiter with the given default values.
    pub fn new(default_bucket_size: u32, default_refill_rate:u32, default_refill_interval_ms:u64) -> Self {
        Self {
            default_bucket_size: default_bucket_size,
            default_refill_rate: default_refill_rate,
            default_refill_interval_ms: default_refill_interval_ms,
            buckets: HashMap::new(),
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
        let bucket = self.buckets.entry(key).or_insert(Bucket {
            size: self.default_bucket_size,
            // Start the bucket full
            tokens: self.default_bucket_size,
            refill_rate: self.default_refill_rate,
            refill_interval_ms: self.default_refill_interval_ms,
            last_filled: Instant::now(),
        });

        bucket.request()
    }

    /// Set the bucket size, refill rate, and refill interval for a given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to  of the VIP we are making a special case for.
    /// * `bucket_size` - The size of the VIP's bucket.
    /// * `refill_rate` - The refill rate of the VIP's bucket.
    /// * `refill_interval_ms` - The refill interval of the VIP's bucket.
    fn set_vip(&mut self, key: T, bucket_size: u32, refill_rate: u32) {
        let bucket = self.buckets.entry(key).or_insert(Bucket {
            size: bucket_size,
            // Start the bucket full
            tokens: bucket_size,
            refill_rate: refill_rate,
            refill_interval_ms: self.default_refill_interval_ms,
            last_filled: Instant::now(),
        });

        bucket.last_filled = Instant::now();
        bucket.tokens = bucket.size;
    }

}

impl Bucket {
    fn refill(&mut self) {
        let now = Instant::now();
        let orig_tokens = self.tokens;


        let time_passed = now.duration_since(self.last_filled);

        // Tweak this if we want to move to microseconds or nanoseconds
        let ms_passed = time_passed.as_millis() as u64;

        let tokens_to_add = ms_passed * self.refill_rate as u64 / self.refill_interval_ms;

        // Add the tokens to the bucket, but don't exceed the bucket size
        self.tokens = (self.tokens + tokens_to_add as u32).min(self.size);

        // Q: Why not just set the last_filled to now?
        // A: Because of integer division, we would often under-fill the bucket. If we update the last_filled
        //      time naively, then multiple calls to refill() in a row will result in the bucket never being refilled.
        self.last_filled = self.last_filled +
            Duration::from_millis((tokens_to_add * self.refill_interval_ms / self.refill_rate as u64) as u64);
    }

    fn request(&mut self) -> bool {
        self.refill();

        if self.tokens > 0 {
            // Allow the request
            self.tokens -= 1;
            true
        } else {
            // Fail the request
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slow_limiter() {
        // Rate limiter with default bucket size of 10 and refill rate of 1 token per second
        let mut ratelimiter: RateLimiter<String> = RateLimiter::new(10, 1, 1000);

        // Test that we can make 10 requests in a row
        let sample_key = "Erin".to_string();
        for _ in 0..10 {
            assert!(ratelimiter.request(sample_key.clone()));
        }

        // Test that we can't make an 11th request
        assert!(!ratelimiter.request(sample_key.clone()));

        // Test that an unrelated key is not affected
        let unrelated_key = "Honey".to_string();
        for _ in 0..10 {
            assert!(ratelimiter.request(unrelated_key.clone()));
        }

        // Test that we can make an 11th request after waiting 1 second
        std::thread::sleep(Duration::from_millis(1000));
        assert!(ratelimiter.request(sample_key.clone()));
    }

    #[test]
    fn test_vip() {
        // Rate limiter with default bucket size of 10 and refill rate of 1 token per second
        let mut ratelimiter: RateLimiter<String> = RateLimiter::new(10, 1, 1000);

        // There's someone super important who needs to make 100 requests in a row, let them do it.
        let vip_bucket_size = 100;
        let vip_refill_rate = 10;
        let vip_key = "Elliot".to_string();
        let other_key = "Waffle".to_string();

        ratelimiter.set_vip(vip_key.clone(), vip_bucket_size, vip_refill_rate);

        for i in 0..vip_bucket_size {
            // VIP should be ok the entire time
            assert!(ratelimiter.request(vip_key.clone()));
            // The other requesters... not so much
            if i < 10 {
                assert!(ratelimiter.request(other_key.clone()));
            } else {
                assert!(!ratelimiter.request(other_key.clone()));
            }
        }
    }
}