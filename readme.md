# RustDNS

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docs.rs](https://docs.rs/rustdns/badge.svg)](https://docs.rs/rustdns)
[![GitHub issues](https://img.shields.io/github/issues/dinxsh/rdns)](https://github.com/dinxsh/rdns/issues)
[![GitHub stars](https://img.shields.io/github/stars/dinxsh/rdns)](https://github.com/dinxsh/rdns/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/dinxsh/rdns)](https://github.com/dinxsh/rdns/network)

RustDNS [WIP] is an open-source, lightweight, high-performance DNS server implementation in Rust. It provides a simple yet powerful solution for handling DNS queries and managing DNS records.

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

## Building from Source

1. Clone the repository:
   ```
   git clone https://github.com/dinxsh/rdns.git
   cd rustdns
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the DNS server:
   ```
   cargo run --release
   ```

## Documentation

For detailed documentation, please visit [docs.rs/rustdns](https://docs.rs/rustdns).

## Contributing

We welcome contributions to RustDNS! Here are some ways you can contribute:

- Report bugs and suggest features by opening issues
- Submit pull requests to fix issues or add new features
- Improve documentation
- Share your experience and help others in discussions

Please see our [Contributing Guide](CONTRIBUTING.md) for more details on how to get started.

## Code of Conduct

We are committed to providing a friendly, safe, and welcoming environment for all contributors. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

RustDNS is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

If you have any questions or need support, please open an issue on GitHub or reach out to the maintainers.
