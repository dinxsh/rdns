use std::net::{UdpSocket, IpAddr, SocketAddr};
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use clap::{App, Arg, SubCommand};
use std::fmt;

const MAX_PACKET_SIZE: usize = 512;
const CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Clone, Debug)]
struct DnsQuery {
    id: u16,
    domain: String,
    query_type: u16,
}

#[derive(Clone, Debug)]
enum DnsRecordType {
    A(String),    // IPv4
    AAAA(String), // IPv6
    MX(u16, String),
    TXT(String),
}

impl fmt::Display for DnsRecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DnsRecordType::A(_) => write!(f, "A"),
            DnsRecordType::AAAA(_) => write!(f, "AAAA"),
            DnsRecordType::MX(_, _) => write!(f, "MX"),
            DnsRecordType::TXT(_) => write!(f, "TXT"),
        }
    }
}

#[derive(Clone, Debug)]
struct DnsRecord {
    record_type: DnsRecordType,
    ttl: u32,
    last_updated: Instant,
}

struct DnsServer {
    socket: UdpSocket,
    records: Arc<RwLock<HashMap<String, Vec<DnsRecord>>>>,
}

impl DnsServer {
    fn new(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        let records = Arc::new(RwLock::new(HashMap::new()));
        Ok(DnsServer { socket, records })
    }

    fn add_record(&self, domain: &str, record: DnsRecord) {
        let mut records = self.records.write().unwrap();
        records.entry(domain.to_lowercase())
               .or_insert_with(Vec::new)
               .push(record);
    }

    fn run(&self) -> io::Result<()> {
        println!("DNS server listening on {}", self.socket.local_addr()?);

        let mut buf = [0; MAX_PACKET_SIZE];
        loop {
            let (amt, src) = self.socket.recv_from(&mut buf)?;
            
            let records = Arc::clone(&self.records);
            let socket = self.socket.try_clone()?;
            let query_buf = buf[..amt].to_vec();

            thread::spawn(move || {
                if let Err(e) = Self::handle_query(records, socket, &query_buf, src) {
                    eprintln!("Error handling query: {}", e);
                }
            });
        }
    }

    fn handle_query(records: Arc<RwLock<HashMap<String, Vec<DnsRecord>>>>, 
                    socket: UdpSocket, 
                    buf: &[u8], 
                    src: SocketAddr) -> io::Result<()> {
        let query = parse_dns_query(buf)?;
        let response = match Self::get_records(&records, &query.domain, query.query_type) {
            Some(records) => create_dns_response(&query, &records),
            None => create_dns_error_response(&query),
        };
        
        socket.send_to(&response, src)?;
        Ok(())
    }

    fn get_records(records: &Arc<RwLock<HashMap<String, Vec<DnsRecord>>>>, 
                   domain: &str, 
                   query_type: u16) -> Option<Vec<DnsRecord>> {
        let records = records.read().unwrap();
        records.get(&domain.to_lowercase()).and_then(|domain_records| {
            let valid_records: Vec<DnsRecord> = domain_records.iter()
                .filter(|record| record.last_updated.elapsed() < CACHE_TTL && 
                                 Self::match_query_type(record, query_type))
                .cloned()
                .collect();
            if valid_records.is_empty() { None } else { Some(valid_records) }
        })
    }

    fn match_query_type(record: &DnsRecord, query_type: u16) -> bool {
        matches!(
            (&record.record_type, query_type),
            (DnsRecordType::A(_), 1) |
            (DnsRecordType::AAAA(_), 28) |
            (DnsRecordType::MX(_, _), 15) |
            (DnsRecordType::TXT(_), 16)
        )
    }
}

fn parse_dns_query(buf: &[u8]) -> io::Result<DnsQuery> {
    if buf.len() < 12 {
        return Err(io::Error::new(ErrorKind::InvalidData, "Invalid DNS query"));
    }

    let id = u16::from_be_bytes([buf[0], buf[1]]);
    let query_type = u16::from_be_bytes([buf[buf.len() - 4], buf[buf.len() - 3]]);
    let domain = parse_domain_name(&buf[12..buf.len() - 4])?;

    Ok(DnsQuery { id, domain, query_type })
}

fn parse_domain_name(buf: &[u8]) -> io::Result<String> {
    let mut domain = String::new();
    let mut i = 0;
    while i < buf.len() {
        let len = buf[i] as usize;
        if len == 0 { break; }
        
        if !domain.is_empty() { domain.push('.'); }
        
        i += 1;
        if i + len > buf.len() {
            return Err(io::Error::new(ErrorKind::InvalidData, "Invalid domain name"));
        }
        
        domain.push_str(std::str::from_utf8(&buf[i..i + len])
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid domain name"))?);
        i += len;
    }
    Ok(domain)
}

fn create_dns_response(query: &DnsQuery, records: &[DnsRecord]) -> Vec<u8> {
    let mut response = Vec::with_capacity(512);
    response.extend_from_slice(&query.id.to_be_bytes());
    response.extend_from_slice(&[0x81, 0x80, 0x00, 0x01, 0x00, (records.len() as u8), 0x00, 0x00, 0x00, 0x00]);
    
    append_question_section(&mut response, query);
    append_answer_section(&mut response, records);

    response
}

fn append_question_section(response: &mut Vec<u8>, query: &DnsQuery) {
    for part in query.domain.split('.') {
        response.push(part.len() as u8);
        response.extend_from_slice(part.as_bytes());
    }
    response.push(0);
    response.extend_from_slice(&query.query_type.to_be_bytes());
    response.extend_from_slice(&[0x00, 0x01]);
}

fn append_answer_section(response: &mut Vec<u8>, records: &[DnsRecord]) {
    for record in records {
        response.extend_from_slice(&[0xc0, 0x0c]); // Pointer to domain name
        let (record_type, rdata) = match &record.record_type {
            DnsRecordType::A(ip) => (1u16, ip.split('.').map(|octet| octet.parse::<u8>().unwrap_or(0)).collect::<Vec<u8>>()),
            DnsRecordType::AAAA(ip) => (28u16, ip.parse::<IpAddr>().unwrap().to_string().into_bytes()),
            DnsRecordType::MX(pref, name) => {
                let mut data = pref.to_be_bytes().to_vec();
                data.extend_from_slice(name.as_bytes());
                (15u16, data)
            },
            DnsRecordType::TXT(text) => (16u16, text.as_bytes().to_vec()),
        };
        response.extend_from_slice(&record_type.to_be_bytes());
        response.extend_from_slice(&[0x00, 0x01]);
        response.extend_from_slice(&record.ttl.to_be_bytes());
        response.extend_from_slice(&((rdata.len() as u16).to_be_bytes()));
        response.extend_from_slice(&rdata);
    }
}

fn create_dns_error_response(query: &DnsQuery) -> Vec<u8> {
    let mut response = Vec::with_capacity(512);
    response.extend_from_slice(&query.id.to_be_bytes());
    response.extend_from_slice(&[0x81, 0x83, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    
    append_question_section(&mut response, query);

    response
}

fn main() -> io::Result<()> {
    let matches = App::new("RustDNS")
        .version("1.0")
        .author("Dinesh Talwadker")
        .about("A simple DNS server implemented in Rust")
        .subcommand(SubCommand::with_name("run")
            .about("Run the DNS server")
            .arg(Arg::with_name("address")
                  .short('a')
                .long("address")
                .value_name("ADDRESS")
                .help("Sets the address to listen on")
                .takes_value(true)
                .default_value("127.0.0.1:5300")))
        .subcommand(SubCommand::with_name("add")
            .about("Add a DNS record")
            .arg(Arg::with_name("domain")
                .help("The domain name")
                .required(true)
                .index(1))
            .arg(Arg::with_name("type")
                .help("The record type (A, AAAA, MX, TXT)")
                .required(true)
                .index(2))
            .arg(Arg::with_name("value")
                .help("The record value")
                .required(true)
                .index(3))
            .arg(Arg::with_name("ttl")
                .short('t')
                .long("ttl")
                .value_name("TTL")
                .help("Time to live in seconds")
                .takes_value(true)
                .default_value("3600")))
        .get_matches();

    match matches.subcommand() {
        Some(("run", run_matches)) => {
            let address = run_matches.value_of("address").unwrap();
            let server = DnsServer::new(address)?;
            
            // Add sample DNS records
            let sample_records = [
                ("example.com", DnsRecordType::A("93.184.216.34".to_string())),
                ("rust-lang.org", DnsRecordType::A("13.35.11.83".to_string())),
                ("example.com", DnsRecordType::AAAA("2606:2800:220:1:248:1893:25c8:1946".to_string())),
                ("mail.example.com", DnsRecordType::MX(10, "mailserver.example.com".to_string())),
                ("example.com", DnsRecordType::TXT("v=spf1 include:_spf.example.com ~all".to_string())),
            ];

            for (domain, record_type) in sample_records.iter() {
                server.add_record(domain, DnsRecord {
                    record_type: record_type.clone(),
                    ttl: 3600,
                    last_updated: Instant::now(),
                });
            }
            
            server.run()
        },
        Some(("add", add_matches)) => {
            let domain = add_matches.value_of("domain").unwrap();
            let record_type = add_matches.value_of("type").unwrap();
            let value = add_matches.value_of("value").unwrap();
            let ttl = add_matches.value_of("ttl").unwrap().parse().unwrap_or(3600);

            let record_type = match record_type.to_uppercase().as_str() {
                "A" => DnsRecordType::A(value.to_string()),
                "AAAA" => DnsRecordType::AAAA(value.to_string()),
                "MX" => {
                    let parts: Vec<&str> = value.split_whitespace().collect();
                    if parts.len() != 2 {
                        return Err(io::Error::new(ErrorKind::InvalidInput, "MX record should have preference and exchange"));
                    }
                    let preference = parts[0].parse().map_err(|_| io::Error::new(ErrorKind::InvalidInput, "Invalid MX preference"))?;
                    DnsRecordType::MX(preference, parts[1].to_string())
                },
                "TXT" => DnsRecordType::TXT(value.to_string()),
                _ => return Err(io::Error::new(ErrorKind::InvalidInput, "Unsupported record type")),
            };

            println!("Added record: {} {} {} (TTL: {})", domain, record_type, value, ttl);
            Ok(())
        },
        _ => {
            // Default behavior: run the DNS server with default settings
            let address = "127.0.0.1:5300";
            let server = DnsServer::new(address)?;
            
            // Add sample DNS records
            let sample_records = [
                ("example.com", DnsRecordType::A("93.184.216.34".to_string())),
                ("rust-lang.org", DnsRecordType::A("13.35.11.83".to_string())),
                ("example.com", DnsRecordType::AAAA("2606:2800:220:1:248:1893:25c8:1946".to_string())),
                ("mail.example.com", DnsRecordType::MX(10, "mailserver.example.com".to_string())),
                ("example.com", DnsRecordType::TXT("v=spf1 include:_spf.example.com ~all".to_string())),
            ];

            for (domain, record_type) in sample_records.iter() {
                server.add_record(domain, DnsRecord {
                    record_type: record_type.clone(),
                    ttl: 3600,
                    last_updated: Instant::now(),
                });
            }
            
            println!("No command specified. Running DNS server with default settings...");
            server.run()
        },
    }
}

// To run this DNS server:
// 1. Make sure you have Rust installed on your system.
// 2. Save this code in a file named `main.rs` in your project directory.
// 3. Open a terminal and navigate to the project directory.
// 4. Run the following command to build and run the server:
//    cargo run
// 5. The server will start and listen on 127.0.0.1:5300.
// 6. You can test it using the dig command. Here are some examples with common options: