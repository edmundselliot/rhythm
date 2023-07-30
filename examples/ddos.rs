use rhythm::RateLimiter;

/*
IP-based rate limiting example.

Output:

    Connected to 169.254.0.1
    Connected to 169.254.0.2
    Connected to 169.254.0.3
    ...
    Connected to 169.254.0.253
    Connected to 169.254.0.254
    Connected to 169.254.0.255

    Connected to 12.34.56.78
    Connected to 12.34.56.78
    Connected to 12.34.56.78
    Connected to 12.34.56.78
    Connected to 12.34.56.78
    Connected to 12.34.56.78
    Connected to 12.34.56.78
    Connected to 12.34.56.78
    Connected to 12.34.56.78
    Connected to 12.34.56.78
    DDoS prevented! Connection to 12.34.56.78 denied
    DDoS prevented! Connection to 12.34.56.78 denied
    DDoS prevented! Connection to 12.34.56.78 denied
*/

struct Server {
    ip: String,
    port: u16,
    connections: u32,
    rate_limiter: RateLimiter<String>,
}

impl Server {
    fn new(ip: String, port: u16) -> Self {
        Self {
            ip,
            port,
            connections: 0,
            rate_limiter: RateLimiter::new(
                // 10 tokens per bucket
                10,
                // 1 token per refill
                1,
                // 1 refill per second
                std::time::Duration::from_secs(1),
            ),
        }
    }

    fn connect(&mut self, ip: String) -> bool {
        if self.rate_limiter.request(ip.clone()) {
            self.connections += 1;
            true
        } else {
            false
        }
    }

    fn disconnect(&mut self) {
        self.connections -= 1;
    }
}

fn main() {
    let mut server = Server::new("127.0.0.1".to_string(), 8080);

    // Normal load, a bunch of connections are extablished
    for i in 0..255 {
        let normal_client_ip = format!("169.254.0.{}", i+1);
        if server.connect(normal_client_ip.clone()) {
            println!("Connected to {}", normal_client_ip);
        } else {
            println!("DDoS prevented! Connection {} denied", normal_client_ip);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // DDoS attack from 12.34.56.78!
    let spam_ip = "12.34.56.78".to_string();
    for _ in 0..255 {
        if server.connect(spam_ip.clone()) {
            println!("Connected to {spam_ip}");
        } else {
            println!("DDoS prevented! Connection to {spam_ip} denied");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}