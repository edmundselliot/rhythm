# Rate limit with Rhythm

Rhythm's `RateLimiter` is a thread-safe rate limiting library implemented in Rust. It allows you to limit the rate of operations, such as requests to a server, by associating these operations with keys. Each key has a "bucket" of tokens, and each operation consumes a token. When the bucket is empty, the operations are limited.

## Features

- **Thread-Safe**: Rhythm's Rate Limiter uses a `Mutex` to ensure that it can be safely used from multiple threads.
- **Customizable**: You can tune the default bucket size, refill rate, and refill interval.
- **Per-Key Limits**: Rhythm introduces the concept of VIPs, who have their own tunable limits. This allows granular control over the limit that each individual has.

## Usage

First, create a new `RateLimiter`, tuned to your use case:

```rust
let rate_limiter: RateLimiter<MyKeyType> = RateLimiter::new(
    bucket_size,
    refill_rate,
    refill_interval
);
```

Then, use the `request` method to perform an operation:

```rust
if rate_limiter.request(my_key) {
    // The operation is allowed.
} else {
    // The operation is not allowed.
}
```

In this example, `my_key` is the key associated with the operation. The `request` method returns `true` if the operation is allowed (i.e., the bucket associated with the key is not empty), and `false` otherwise.

By default, all keys will have the rate limiter's default bucket size, refill rate, and refill interval.

To customize this, you can mark a key as a VIP:

```rust
rate_limiter.set_vip(
    vip_key.clone(),
    vip_bucket_size,
    vip_refill_rate
);
```

## Installation

Add the following to your Cargo.toml:

Run `cargo add rhythm` to add the latest version to your `Cargo.toml`.

Then, run `cargo build` to build your project.

## TODO

Performance improvements:

1. Per-bucket mutex to avoid holding the table lock for each request.
2. Move pruning to a LRU-cache style trimming.

Functionality improvements:

1. VIP buckets should be able to set their own refill interval.

## License

Rhythm's Rate Limiter is licensed under the MIT License. See [LICENSE](./LICENSE) for more information.
