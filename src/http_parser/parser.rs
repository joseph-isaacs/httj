use std::collections::HashMap;
use std::io::Bytes;
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
    pub chunk: Vec<u8>,
    pub total_bytes_read: usize,
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
    parse_headers(state)?;


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

    state.request.body_str = std::str::from_utf8(&state.request.body_bytes).unwrap().to_string();

    // println!("request: {:?}", state.request);

    return Some(ParsingResult::Complete)

}

fn parse_headers(state: &mut HttpParserState) -> Option<ParsingResult> {
    if state.stage != Stage::Headers {
        return Some(ParsingResult::Complete);
    }
    let mut headers: HashMap<String, String> = HashMap::new();

    let chunk = &state.chunk[state.pos..];
    // let chunk = &state.chunk[state.pos..];

    let mut start = state.pos;
    let mut middle: Option<usize> = None;
    if let Some(middleV) = chunk.iter().position(|c| *c == (':' as u8)) {
        middle = Some(start + middleV);
    }
    let mut end = start + chunk.iter().position(|c| *c == ('\r' as u8))?;

    loop {
        if start == end {
            start += 2; // skip \r\n
            break
        }

        if let Some(middle) = middle {
            let middle_pos = middle;
            let end_pos = end;
            let key = std::str::from_utf8(&state.chunk[start..middle_pos]).unwrap();
            let value = std::str::from_utf8(&state.chunk[middle_pos+2..end_pos]).unwrap();
            headers.insert(key.to_string(), value.to_string());
        }

        start = end + 2;
        end = start + state.chunk[start..].iter().position(|c| *c == ('\r' as u8))?;

        if let Some(middleV) = state.chunk[start..].iter().position(|c| *c == (':' as u8)) {
            middle = Some(start + middleV);
        }
    }

    state.request.headers = headers;
    state.stage = Stage::Body;
    state.pos = start;

    return Some(ParsingResult::Complete)
}

fn parse_request_line(state: &mut HttpParserState) -> Option<ParsingResult> {
    if state.stage != RequestLine {
        return Some(ParsingResult::Complete);
    }
    let mut chunk = state.chunk.iter();

    let mut start = 0;
    let mut end = chunk.position(|c| *c == (' ' as u8))?;

    state.request.method = Method::from_str(std::str::from_utf8(&state.chunk[start..end]).unwrap()).unwrap();
    start = end + 1;

    end += state.chunk[start..].iter().position(|c| *c == ' ' as u8)? + 1;
    state.request.path = (std::str::from_utf8(&state.chunk[start..end]).unwrap()).to_string();
    start = end + 1;

    end += state.chunk[start..].iter().position(|c| *c == '\r' as u8)? + 1;
    let s = std::str::from_utf8(&state.chunk[start..end]);
    state.request.version = Version::from_str(s.unwrap()).unwrap();
    start = end + 1;

    state.stage = Stage::Headers;
    state.pos = end + 2;

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

    let len = &state.chunk[state.pos..].iter().len();
    if  *len < content_length {
        return Some(ParsingResult::Partial)
    }
    if  *len > content_length {
        println!("this cannot happen {}", len);
        return None
    }
    state.request.body_bytes = state.chunk[state.pos..(state.pos+len)].to_vec();
    return Some(ParsingResult::Complete)
}

