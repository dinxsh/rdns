# Simple DNS Server in Rust

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A basic DNS server implementation in Rust. This project demonstrates how to create a simple DNS server that can handle queries and respond with IPv4 addresses.

## Features

- DNS query parsing and handling
- IPv4 address resolution for configured domains
- Multithreaded request handling for improved performance
- Error responses for non-existent domains
- Easy-to-use API for adding DNS records

## Prerequisites

- Rust programming language (https://www.rust-lang.org/tools/install)

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/dns_server.git
   cd dns_server
   ```

2. Build the project:
   ```
   cargo build --release
   ```

## Usage

1. Run the DNS server:
   ```
   cargo run --release
   ```

   The server will start and listen on `127.0.0.1:5300`.

2. Test the server using `nslookup`:
   ```
   nslookup -port=5300 example.com 127.0.0.1
   ```

   You should receive a response with the IP address `93.184.216.34` for `example.com`.

## Configuration

The DNS server is pre-configured with one record:
- Domain: `example.com`
- IP: `93.184.216.34`

To add more records, modify the `main()` function in `src/main.rs`: