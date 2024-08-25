use std::process::Command;
use std::net::UdpSocket;
use std::str;

#[test]
fn test_dns_server_startup() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(str::from_utf8(&output.stdout).unwrap().contains("DNS Server"));
}

#[test]
fn test_dns_query() {
    // Start the DNS server in the background
    let mut server = Command::new("cargo")
        .arg("run")
        .spawn()
        .expect("Failed to start DNS server");

    // Give the server some time to start up
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Create a UDP socket for sending DNS queries
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
    socket.connect("127.0.0.1:5300").expect("Failed to connect to DNS server");

    // Construct a simple DNS query for example.com
    let query = [
        0x00, 0x01, // Transaction ID
        0x01, 0x00, // Flags
        0x00, 0x01, // Questions
        0x00, 0x00, // Answer RRs
        0x00, 0x00, // Authority RRs
        0x00, 0x00, // Additional RRs
        0x07, 0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, // example
        0x03, 0x63, 0x6F, 0x6D, // com
        0x00, // null terminator
        0x00, 0x01, // Type A
        0x00, 0x01, // Class IN
    ];

    // Send the query
    socket.send(&query).expect("Failed to send DNS query");

    // Receive the response
    let mut buf = [0; 512];
    let (amt, _) = socket.recv_from(&mut buf).expect("Failed to receive DNS response");

    // Check that we received a response
    assert!(amt > 0);

    // Check that the response contains the expected IP address
    let response = str::from_utf8(&buf[..amt]).unwrap();
    assert!(response.contains("93.184.216.34"));

    // Stop the DNS server
    server.kill().expect("Failed to stop DNS server");
}

#[test]
fn test_add_record() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("add")
        .arg("test.com")
        .arg("--type")
        .arg("A")
        .arg("--value")
        .arg("1.2.3.4")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(str::from_utf8(&output.stdout).unwrap().contains("Added record: test.com A 1.2.3.4"));
}

// To run this test file:
// 1. Make sure you are in the root directory of your Rust project.
// 2. Open a terminal in this directory.
// 3. Run the following command:
//    cargo test --test t1
// 
// This command tells Cargo to run the tests in the file named t1.rs
// located in the tests directory of your project.
//
// If you want to run a specific test, you can use:
//    cargo test --test t1 test_name
// Replace test_name with the name of the test function you want to run.
//
// For example, to run only the test_dns_server_startup test:
//    cargo test --test t1 test_dns_server_startup
