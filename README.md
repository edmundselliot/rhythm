# Rhythm's Rate Limiter

Rhythm's Rate Limiter is a thread-safe rate limiting library implemented in Rust. It allows you to limit the rate of operations, such as requests to a server, by associating these operations with keys. Each key has a "bucket" of tokens, and each operation consumes a token. When the bucket is empty, the operations are limited.

## Features

- **Thread-Safe**: Rhythm's Rate Limiter uses a `Mutex` to ensure that it can be safely used from multiple threads.
- **Customizable**: You can customize the default bucket size, refill rate, and refill interval.
- **Per-Key Limits**: Each key has its own bucket, so you can limit the rate of operations per key.

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

## Installation

Add the following to your Cargo.toml:

```toml
[dependencies]
rhythm-rate-limiter = "0.1.0"
```

Then, run `cargo build` to build your project.

## License

Rhythm's Rate Limiter is licensed under the MIT License. See [LICENSE](./LICENSE) for more information.
