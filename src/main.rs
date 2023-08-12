use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Instant;
use crate::http_parser::http_request::{HttpRequest, Method, Version};
use crate::http_parser::parser::{HttpParserState, Stage};

pub mod http_parser;

fn handle_client(mut stream: TcpStream) {
    // Handle the client connection
    // let buffer = &mut [0u8; 1024];
    let buffer = &mut [0u8; 2048];
    // let mut buffer: Vec<u8> = Vec::with_capacity(65535);

    // let buffer: Vec<u8> = Vec::new();
    let len = stream.read(buffer.as_mut_slice()).unwrap();
    // let ascii_string = String::from_utf8_lossy(&buffer[..len]).to_string();
    // let ascii_string = String::from_utf8(buffer[..len].to_vec()).unwrap();


    // let str_len = ascii_string.len();

    let state = &mut HttpParserState {
        chunk: buffer[..len].to_vec(),
        total_bytes_read: len,
        pos: 0,
        stage: Stage::RequestLine,
        request: HttpRequest {
            version: Version::HTTP1_0,
            method: Method::GET,
            path: "".to_string(),
            headers: Default::default(),
            body_bytes: Default::default(),
            body_str: Default::default(),
        },
    };



    loop {
        let res = http_parser::parser::parse_http_1_1_request(state).unwrap();
        if res == http_parser::parser::ParsingResult::Complete {
            break;
        }
        // let buffer = &mut [0u8; 1024];
        let buffer = &mut [0u8; 2048];
        // let mut buffer: Vec<u8> = Vec::with_capacity(65535);

        let len = stream.read(buffer.as_mut_slice()).unwrap();
        // let ascii_string = String::from_utf8_lossy(&buffer[..len]).to_string();
        // let ascii_string = String::from_utf8(buffer[..len].to_vec()).unwrap();


        state.chunk.extend(&buffer[..len].to_vec());

        state.total_bytes_read += len;
    }

    // use parse_http_1_1_request in crate http_parser

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
                    let start = Instant::now();
                    handle_client(stream);
                    let elapsed = start.elapsed();
                    println!("Elapsed: {:?}", elapsed);
                });
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
