# RustDNS

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/workflow/status/dinxsh/rustdns/CI)](https://github.com/dinxsh/rustdns/actions)
[![Crates.io](https://img.shields.io/crates/v/rustdns.svg)](https://crates.io/crates/rustdns)
[![Docs.rs](https://docs.rs/rustdns/badge.svg)](https://docs.rs/rustdns)

RustDNS is a lightweight, high-performance DNS server implementation in Rust. It provides a simple yet powerful solution for handling DNS queries and managing DNS records.

## Features

- Fast and efficient DNS query parsing and handling
- Support for IPv4 and IPv6 address resolution
- Multithreaded architecture for optimal performance
- Configurable caching mechanism
- Extensible plugin system for custom record types
- Comprehensive logging and monitoring
- Easy-to-use API for programmatic DNS record management

## Prerequisites

- Rust 1.56.0 or later

## Quick Start

1. Add RustDNS to your `Cargo.toml`:
   ```toml
   [dependencies]
   rustdns = "0.1.0"
   ```

2. Use RustDNS in your project:
   ```rust
   use rustdns::{DnsServer, Config};

   fn main() {
       let config = Config::new()
           .bind_address("127.0.0.1:5300")
           .add_record("example.com", "93.184.216.34");

       let server = DnsServer::new(config);
       server.run().expect("Failed to start DNS server");
   }
   ```

## Documentation

For detailed documentation, please visit [docs.rs/rustdns](https://docs.rs/rustdns).

## Contributing

We welcome contributions to RustDNS! Please see our [Contributing Guide](CONTRIBUTING.md) for more details.

## License

RustDNS is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.