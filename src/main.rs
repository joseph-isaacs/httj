use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub mod http_parser;

fn handle_client(mut stream: TcpStream) {
    // Handle the client connection
    let mut buffer = [0; 2*1024];
    stream.read(&mut buffer).unwrap();

    println!("Received data: {:?}", &buffer);

    let ascii_string = String::from_utf8_lossy(&buffer).to_string();

    println!("Received data as string: {:?}", &ascii_string);

    // use parse_http_1_1_request in crate http_parser
    http_parser::parser::parse_http_1_1_request(ascii_string);

    let response = r#"
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 13

Hello, world!
"#.trim_start_matches('\n');

    // write response to stream
    stream.write(response.as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind");
    println!("Listening on 127.0.0.1:8080...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
