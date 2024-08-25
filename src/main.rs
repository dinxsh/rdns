use std::net::UdpSocket;
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use std::net::IpAddr;

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
    CNAME(String),
    MX(u16, String),
    TXT(String),
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

        loop {
            let mut buf = [0; MAX_PACKET_SIZE];
            let (amt, src) = self.socket.recv_from(&mut buf)?;
            
            let records = Arc::clone(&self.records);
            let socket = self.socket.try_clone()?;

            thread::spawn(move || {
                match parse_dns_query(&buf[..amt]) {
                    Ok(query) => {
                        let response = match Self::get_records(&records, &query.domain, query.query_type) {
                            Some(records) => create_dns_response(&query, &records),
                            None => create_dns_error_response(&query),
                        };
                        
                        if let Err(e) = socket.send_to(&response, src) {
                            eprintln!("Failed to send response: {}", e);
                        }
                    },
                    Err(e) => eprintln!("Failed to parse DNS query: {}", e),
                }
            });
        }
    }

    fn get_records(records: &Arc<RwLock<HashMap<String, Vec<DnsRecord>>>>, domain: &str, query_type: u16) -> Option<Vec<DnsRecord>> {
        let records: std::sync::RwLockReadGuard<HashMap<String, Vec<DnsRecord>>> = records.read().unwrap();
        records.get(&domain.to_lowercase()).and_then(|domain_records| {
            let valid_records: Vec<DnsRecord> = domain_records.iter()
                .filter(|record| record.last_updated.elapsed() < CACHE_TTL && Self::match_query_type(record, query_type))
                .cloned()
                .collect();
            if valid_records.is_empty() { None } else { Some(valid_records) }
        })
    }

    fn match_query_type(record: &DnsRecord, query_type: u16) -> bool {
        match (&record.record_type, query_type) {
            (DnsRecordType::A(_), 1) => true,
            (DnsRecordType::AAAA(_), 28) => true,
            (DnsRecordType::CNAME(_), 5) => true,
            (DnsRecordType::MX(_, _), 15) => true,
            (DnsRecordType::TXT(_), 16) => true,
            _ => false,
        }
    }
}

fn parse_dns_query(buf: &[u8]) -> io::Result<DnsQuery> {
    if buf.len() < 12 {
        return Err(io::Error::new(ErrorKind::InvalidData, "Invalid DNS query"));
    }

    let id = u16::from_be_bytes([buf[0], buf[1]]);
    let query_type = u16::from_be_bytes([buf[buf.len() - 4], buf[buf.len() - 3]]);

    let mut domain = String::new();
    let mut i = 12;
    while i < buf.len() - 4 {
        let len = buf[i] as usize;
        if len == 0 {
            break;
        }
        if !domain.is_empty() {
            domain.push('.');
        }
        domain.push_str(std::str::from_utf8(&buf[i + 1..i + 1 + len]).map_err(|_| io::Error::new(ErrorKind::InvalidData, "Invalid domain name"))?);
        i += len + 1;
    }

    Ok(DnsQuery { id, domain, query_type })
}

fn create_dns_response(query: &DnsQuery, records: &[DnsRecord]) -> Vec<u8> {
    let mut response = Vec::with_capacity(512);
    response.extend_from_slice(&query.id.to_be_bytes());
    response.extend_from_slice(&[0x81, 0x80, 0x00, 0x01, 0x00, (records.len() as u8), 0x00, 0x00, 0x00, 0x00]);
    
    // Question section
    for part in query.domain.split('.') {
        response.push(part.len() as u8);
        response.extend_from_slice(part.as_bytes());
    }
    response.push(0);
    response.extend_from_slice(&query.query_type.to_be_bytes());
    response.extend_from_slice(&[0x00, 0x01]);

    // Answer section
    for record in records {
        response.extend_from_slice(&[0xc0, 0x0c]); // Pointer to domain name
        let (record_type, rdata) = match &record.record_type {
            DnsRecordType::A(ip) => (1u16, ip.split('.').map(|octet| octet.parse::<u8>().unwrap_or(0)).collect::<Vec<u8>>()),
            DnsRecordType::AAAA(ip) => (28u16, ip.parse::<IpAddr>().unwrap().to_string().into_bytes()),
            DnsRecordType::CNAME(name) => (5u16, name.as_bytes().to_vec()),
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

    response
}

fn create_dns_error_response(query: &DnsQuery) -> Vec<u8> {
    let mut response = Vec::with_capacity(512);
    response.extend_from_slice(&query.id.to_be_bytes());
    response.extend_from_slice(&[0x81, 0x83, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    
    // Question section
    for part in query.domain.split('.') {
        response.push(part.len() as u8);
        response.extend_from_slice(part.as_bytes());
    }
    response.push(0);
    response.extend_from_slice(&query.query_type.to_be_bytes());
    response.extend_from_slice(&[0x00, 0x01]);

    response
}

fn main() -> io::Result<()> {
    let server = DnsServer::new("127.0.0.1:5300")?;
    
    // Add sample DNS records
    server.add_record("example.com", DnsRecord {
        record_type: DnsRecordType::A("93.184.216.34".to_string()),
        ttl: 3600,
        last_updated: Instant::now(),
    });
    server.add_record("rust-lang.org", DnsRecord {
        record_type: DnsRecordType::A("13.35.11.83".to_string()),
        ttl: 3600,
        last_updated: Instant::now(),
    });
    server.add_record("example.com", DnsRecord {
        record_type: DnsRecordType::AAAA("2606:2800:220:1:248:1893:25c8:1946".to_string()),
        ttl: 3600,
        last_updated: Instant::now(),
    });
    server.add_record("mail.example.com", DnsRecord {
        record_type: DnsRecordType::MX(10, "mailserver.example.com".to_string()),
        ttl: 3600,
        last_updated: Instant::now(),
    });
    server.add_record("example.com", DnsRecord {
        record_type: DnsRecordType::TXT("v=spf1 include:_spf.example.com ~all".to_string()),
        ttl: 3600,
        last_updated: Instant::now(),
    });
    
    server.run()
}