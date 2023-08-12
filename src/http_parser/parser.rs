use std::collections::HashMap;
use std::str::{FromStr, Lines};
use crate::http_parser::http_request::{HttpRequest, Method, Version};

// function taking in an http 1.1 request in bytes and returning a parsed http request
pub fn parse_http_1_1_request(request: String) {
    let mut req = HttpRequest {
        version: Version::HTTP1_0,
        method: Method::GET,
        path: "".to_string(),
        headers: Default::default(),
        body: Default::default(),
    };
    let mut lines = request.lines();
    if let Some(request_line) = lines.next() {
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or_default();
        let http_version = parts.next().unwrap_or_default();

        req.version = Version::from_str(http_version).unwrap();
        req.method = Method::from_str(method).unwrap();
        req.path = path.to_string();
    }
    // create a headers hashmap
    let mut headers: HashMap<String, String> = HashMap::new();
    while let Some(line) = lines.next() {
        if line == "" {
            break
        }
        if let Some((key, value)) = line.split_once(":") {
            headers.insert(key.parse().unwrap(), value.chars().skip(1).collect::<String>());
        }
    }
    req.headers = headers;

    if let Some(content_length) = req.headers.get("Content-Length") {
        let len = content_length.clone();
        extract_fixed_size_body(&mut req, &lines, &len);
    } else {
        if let Some(_) = req.headers.get("Transfer-Encoding") {
            extract_chunked_body(&mut req, &lines);
        }
    }




    println!("Request is: {:#?}", req)

}

fn extract_chunked_body(req: &mut HttpRequest, lines: &Lines) {
    // extract http chunked body encoding
    let mut body = String::new();
    req.body = body;
    // for line in lines {
    //     let chunk_size = usize::from_str_radix(line, 16).unwrap();
    //     if chunk_size == 0 {
    //         break
    //     }
    //     let chunk = &line.chars().skip(2).collect::<String>()[..chunk_size];
    //     body.push_str(chunk);
    // }

}

fn extract_fixed_size_body(req: &mut HttpRequest, lines: &Lines, content_length: &String) {
    let content_length: usize = content_length.parse().unwrap();
    let rest_of_string: String = lines.clone().collect::<Vec<&str>>().join("\n");
    req.body = rest_of_string.chars().take(content_length).collect::<String>();
}

