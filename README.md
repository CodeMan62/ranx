# Ranx - High-Performance Reverse Proxy

Ranx is a modern, high-performance reverse proxy written in Rust. It provides advanced features like load balancing, rate limiting, circuit breaking, and real-time metrics collection.

## Features

- 🚀 **High Performance**: Built with Rust for maximum speed and reliability
- 🔄 **Load Balancing**: Round-robin load balancing across multiple backend servers
- 🛡️ **Rate Limiting**: Protect your services from abuse with configurable rate limits
- ⚡ **Circuit Breaking**: Automatic failure detection and recovery
- 📊 **Metrics Collection**: Real-time monitoring of request/response metrics
- 🔒 **TLS Support**: Secure communication with SSL/TLS
- 🎯 **Path-based Routing**: Flexible routing based on URL paths
- 📝 **Structured Logging**: Comprehensive logging with different log levels

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/ranx.git
cd ranx
```

2. Build the project:
```bash
cargo build --release
```

3. Run the proxy:
```bash
cargo run --release
```

### Configuration

Create a `config.yaml` file in the project root:

```yaml
server:
  listen_addr: 127.0.0.1:8080
  # Optional TLS configuration
  # tls:
  #   cert_path: ./certs/cert.pem
  #   key_path: ./certs/key.pem

backends:
  api_servers:
    servers:
      - "http://localhost:3000"
      - "http://localhost:3001"
    timeout: 30
  web:
    servers:
      - "http://localhost:8000"
    timeout: 60

routes:
  - path: "/api"
    backend: "api_servers"
    strip_prefix: true
  - path: "/"
    backend: "web"
    strip_prefix: false
```

## Features in Detail

### Load Balancing

Ranx supports round-robin load balancing across multiple backend servers. When a backend has multiple servers configured, requests are distributed evenly across them.

### Rate Limiting

Protect your services from abuse with configurable rate limits:

```yaml
rate_limit:
  requests_per_second: 100
  burst_size: 50
```

### Circuit Breaking

Automatic failure detection and recovery:

```yaml
circuit_breaker:
  failure_threshold: 5
  reset_timeout: 30
  half_open_timeout: 10
```

### Metrics Collection

Real-time monitoring of:
- Request/response counts
- Latency statistics
- Error rates
- Circuit breaker states

## Production Deployment

### Docker Support

Build the Docker image:
```bash
docker build -t ranx .
```

Run the container:
```bash
docker run -p 8080:8080 -v $(pwd)/config.yaml:/app/config.yaml ranx
```

### Monitoring

Ranx provides metrics endpoints for integration with monitoring systems:
- `/metrics`: Prometheus-compatible metrics
- `/health`: Health check endpoint
- `/status`: Detailed proxy status

## Performance

- Handles thousands of concurrent connections
- Sub-millisecond latency for most operations
- Memory-efficient design
- Zero-copy request/response handling

## Security

- TLS support for secure communication
- Rate limiting to prevent abuse
- Circuit breaking to prevent cascading failures
- Header sanitization
- IP-based access control

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Author

Your Name - [@yourtwitter](https://twitter.com/yourtwitter)

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- HTTP handling with [Hyper](https://github.com/hyperium/hyper)
- Configuration with [serde](https://github.com/serde-rs/serde)
- Logging with [tracing](https://github.com/tokio-rs/tracing)
