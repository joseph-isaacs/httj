use std::collections::HashMap;
use std::os::macos::raw::stat;
use std::str::{FromStr, Lines};
use std::sync::atomic::Ordering::AcqRel;
use crate::http_parser::http_request::{HttpRequest, Method, Version};
use crate::http_parser::parser::Stage::RequestLine;

#[derive(PartialEq)]
pub enum Stage {
    RequestLine,
    Headers,
    Body,
}

pub struct HttpParserState {
    pub chunk: String,
    pub total_bytes_read: usize,
    pub total_chars_read: usize,
    pub pos: usize,
    pub stage: Stage,
    pub request: HttpRequest
}

#[derive(PartialEq)]
pub enum ParsingResult {
    Complete,
    Partial,
}

// function taking in an http 1.1 request in bytes and returning a parsed http request
pub fn parse_http_1_1_request(state: &mut HttpParserState) -> Option<ParsingResult> {



    parse_request_line(state)?;

    // create a headers hashmap
    parse_headers(state);


    if let Some(content_length) = state.request.headers.get("Content-Length") {
        let len = content_length.clone();
        let res = extract_fixed_size_body(state, &len)?;
        if res == ParsingResult::Partial {
            return Some(ParsingResult::Partial)
        }
    } else {
        if let Some(_) = state.request.headers.get("Transfer-Encoding") {
            let rest_of_string: String = String::from("5\r\nhello");
            extract_chunked_body(state, rest_of_string);
        }
    }

    return Some(ParsingResult::Complete)

}

fn parse_headers(state: &mut HttpParserState) {
    if state.stage != Stage::Headers {
        return;
    }
    let mut chunk = state.chunk.as_str();

    chunk = &chunk[state.pos..];
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut lines = chunk.lines();
    while let Some(line) = lines.next() {
        state.pos += line.len() + 2;
        if line == "" {
            break
        }
        if let Some((key, value)) = line.split_once(":") {
            headers.insert(key.parse().unwrap(), value.chars().skip(1).collect::<String>());
        }
    }
    state.request.headers = headers;
    state.stage = Stage::Body;
}

fn parse_request_line(state: &mut HttpParserState) -> Option<ParsingResult> {
    if state.stage != RequestLine {
        return Some(ParsingResult::Complete);
    }
    let mut chunk = state.chunk.as_str();

    let mut start = 0;
    let mut end = chunk.find(" ")?;
    state.request.method = Method::from_str(&chunk[..end]).unwrap();
    start = end + 1;

    end += chunk[start..].find(" ")? + 1;
    state.request.path = (&chunk[start..end]).to_string();
    start = end + 1;

    end += chunk[start..].find("\r\n")? + 1;
    state.request.version = Version::from_str(&chunk[start..end]).unwrap();
    start = end + 2;

    state.stage = Stage::Headers;
    state.pos = end + 2;

    // let rest = &state.chunk[state.pos..];

    return Some(ParsingResult::Complete)

}

fn extract_chunked_body(req: &mut HttpParserState, str: String) {
    // extract http chunked body encoding
    // let mut string_builder = String::new();
    //
    // let mut start = 0;
    //
    // loop {
    //     let c_str = str[start..].chars(); // Create a new iterator for each line
    //
    //     let x = c_str.take_while(|&c| c != '\n');
    //
    //     let size_str: String = x.collect();
    //
    //     if let Ok(chunk_size) = usize::from_str_radix(&size_str, 16) {
    //         if chunk_size == 0 {
    //             break
    //         }
    //
    //         let chunk_start = start + size_str.len() + 1;
    //
    //         let chunk = &str[chunk_start..(chunk_start+chunk_size)];
    //
    //         string_builder.push_str(chunk);
    //
    //         start = chunk_start + chunk_size; // Move the 'start' index to the beginning of the next line
    //     } else {
    //         break
    //     }
    // }
    //
    // req.body = string_builder

}

fn extract_fixed_size_body(state: &mut HttpParserState, content_length: &String) -> Option<ParsingResult> {
    let content_length: usize = content_length.parse().unwrap();
    let rest = &state.chunk.as_str()[state.pos..];
    let len = rest.len();
    let lenb = rest.bytes().len();
    if  len < content_length {
        return Some(ParsingResult::Partial)
    }
    state.request.body = state.chunk.chars().take(content_length).collect::<String>();
    return Some(ParsingResult::Complete)
}

