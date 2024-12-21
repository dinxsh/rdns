# RustDNS

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docs.rs](https://docs.rs/rust-dns/badge.svg)](https://docs.rs/rust-dns)
[![GitHub issues](https://img.shields.io/github/issues/dinxsh/rdns)](https://github.com/dinxsh/rdns/issues)
[![GitHub stars](https://img.shields.io/github/stars/dinxsh/rdns)](https://github.com/dinxsh/rdns/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/dinxsh/rdns)](https://github.com/dinxsh/rdns/network)
[![Crates.io](https://img.shields.io/crates/v/rust-dns)](https://crates.io/crates/rust-dns)
[![Downloads](https://img.shields.io/crates/d/rust-dns)](https://crates.io/crates/rust-dns)

RustDNS [WIP] is an open-source, lightweight, high-performance DNS server implementation in Rust. It provides a simple yet powerful solution for handling DNS queries and managing DNS records.

## Installation

Add rust-dns to your project using cargo:

```bash
cargo add rust-dns
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
rust-dns = "0.1.1"
```

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

```rust
use rust_dns::{DnsServer, DnsRecord};
fn main() -> std::io::Result<()> {
    // Create a new DNS server instance
    let server = DnsServer::new("127.0.0.1:5300")?;
    // Run the server
    server.run()
}
```

## CLI Usage

Run the DNS server:

```bash
rust-dns run --address 127.0.0.1:5300
```

Add a DNS record:

```bash
rust-dns add example.com A 93.184.216.34 --ttl 3600
```

## Building from Source

1. Clone the repository:

```bash
git clone https://github.com/dinxsh/rdns.git
cd rdns
```

2. Build the project:

```bash
cargo build --release
```

## Documentation

For detailed documentation, please visit [docs.rs/rust-dns](https://docs.rs/rust-dns).

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
