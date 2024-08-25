// This code implements a simple DNS server in Rust. Here's a breakdown of its components and functionality:

// Importing necessary modules
use std::net::UdpSocket;
use std::collections::HashMap;
use std::io::{self, Error, ErrorKind};
use std::sync::{Arc, RwLock};
use std::thread;

// Maximum size of a DNS packet
const MAX_PACKET_SIZE: usize = 512;

// Struct to represent a DNS query
#[derive(Clone)]
struct DnsQuery {
    id: u16,
    domain: String,
}

// Struct to represent the DNS server
struct DnsServer {
    socket: UdpSocket,
    records: Arc<RwLock<HashMap<String, String>>>,
}

impl DnsServer {
    // Create a new DNS server
    fn new(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        let records = Arc::new(RwLock::new(HashMap::new()));
        Ok(DnsServer { socket, records })
    }

    // Add a DNS record
    fn add_record(&self, domain: &str, ip: &str) {
        self.records.write().unwrap().insert(domain.to_string(), ip.to_string());
    }

    // Run the DNS server
    fn run(&self) -> io::Result<()> {
        println!("DNS server listening on {}", self.socket.local_addr()?);

        loop {
            let mut buf = [0; MAX_PACKET_SIZE];
            let (amt, src) = self.socket.recv_from(&mut buf)?;
            
            let records = Arc::clone(&self.records);
            let socket = self.socket.try_clone()?;

            // Spawn a new thread to handle each request
            thread::spawn(move || {
                if let Ok(query) = parse_dns_query(&buf[..amt]) {
                    let response = match records.read().unwrap().get(&query.domain) {
                        Some(ip) => create_dns_response(&query, ip),
                        None => create_dns_error_response(&query),
                    };
                    
                    if let Err(e) = socket.send_to(&response, src) {
                        eprintln!("Failed to send response: {}", e);
                    }
                } else {
                    eprintln!("Failed to parse DNS query");
                }
            });
        }
    }
}

// Parse a DNS query from a byte buffer
fn parse_dns_query(buf: &[u8]) -> io::Result<DnsQuery> {
    if buf.len() < 12 {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid DNS query"));
    }

    let id = u16::from_be_bytes([buf[0], buf[1]]);
    let mut domain = String::new();
    let mut i = 12;

    while i < buf.len() {
        let len = buf[i] as usize;
        if len == 0 { break; }
        if i + len + 1 > buf.len() {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid domain name"));
        }
        if !domain.is_empty() {
            domain.push('.');
        }
        domain.push_str(&String::from_utf8_lossy(&buf[i+1..i+1+len]));
        i += len + 1;
    }
    Ok(DnsQuery { id, domain })
}

// Create a DNS response for a successful query
fn create_dns_response(query: &DnsQuery, ip: &str) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(&query.id.to_be_bytes());
    response.extend_from_slice(&[0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);
    for part in query.domain.split('.') {
        response.push(part.len() as u8);
        response.extend_from_slice(part.as_bytes());
    }
    response.push(0);
    response.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);
    response.extend_from_slice(&[0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01]);
    response.extend_from_slice(&[0x00, 0x00, 0x05, 0x00]); // TTL: 1280 seconds
    response.extend_from_slice(&[0x00, 0x04]); // Data length: 4 bytes for IPv4
    response.extend(ip.split('.').map(|octet| octet.parse::<u8>().unwrap()));

    response
}

// Create a DNS error response for a failed query
fn create_dns_error_response(query: &DnsQuery) -> Vec<u8> {
    let mut response = Vec::new();
    response.extend_from_slice(&query.id.to_be_bytes());
    response.extend_from_slice(&[0x81, 0x83, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    
    for part in query.domain.split('.') {
        response.push(part.len() as u8);
        response.extend_from_slice(part.as_bytes());
    }
    response.push(0);
    response.extend_from_slice(&[0x00, 0x01, 0x00, 0x01]);

    response
}

// Main function to start the DNS server
fn main() -> io::Result<()> {
    let server = DnsServer::new("127.0.0.1:5300")?;
    
    // Add a sample DNS record
    server.add_record("example.com", "93.184.216.34");
    
    // Run the server
    server.run()
}

// This DNS server listens on 127.0.0.1:5300, handles DNS queries in separate threads,
// and responds with either the IP address for known domains or an error response for unknown domains.
// It uses a thread-safe HashMap to store DNS records and can be easily extended to add more records.