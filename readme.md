# RustDNS

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

RustDNS is a lightweight, high-performance DNS server implementation in Rust. It provides a simple yet powerful solution for handling DNS queries and managing DNS records.

## Features

- Fast and efficient DNS query parsing and handling
- Support for multiple DNS record types (A, AAAA, CNAME, MX, TXT)
- Multithreaded architecture for optimal performance
- In-memory cache with configurable TTL
- Support for IPv4 and IPv6 address resolution
- Extensible design for easy addition of new record types

## Prerequisites

- Rust (latest stable version recommended)

## Quick Start

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/rustdns.git
   cd rustdns
   ```

2. Run the DNS server:
   ```
   cargo run
   ```

   The server will start listening on `127.0.0.1:5300` by default.

3. Test the server using a DNS client tool like `dig`:
   ```
   dig @127.0.0.1 -p 5300 example.com
   ```

## Usage

To use RustDNS in your own project:

1. Add the following to your `Cargo.toml`:
   ```toml
   [dependencies]
   # Add the actual crate name and version when published
   rustdns = { git = "https://github.com/yourusername/rustdns.git" }
   ```

2. Use RustDNS in your code:
   ```rust
   use std::io;
   use rustdns::DnsServer;

   fn main() -> io::Result<()> {
       let server = DnsServer::new("127.0.0.1:5300")?;
       
       // Add DNS records
       server.add_record("example.com", DnsRecord::new_a("93.184.216.34", 3600));
       server.add_record("rust-lang.org", DnsRecord::new_a("13.35.11.83", 3600));
       
       server.run()
   }
   ```

## Contributing

Contributions to RustDNS are welcome! Please feel free to submit issues, fork the repository and send pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.