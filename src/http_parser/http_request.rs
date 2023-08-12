use std::collections::HashMap;
use strum_macros::EnumString;


// an enum for all the http methods
#[derive(Debug, EnumString)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

// an enum for all the http version
#[derive(Debug, EnumString)]
pub enum Version {
    #[strum(serialize = "HTTP/1.0")]
    HTTP1_0,
    #[strum(serialize = "HTTP/1.1")]
    HTTP1_1,
    #[strum(serialize = "HTTP/2.0")]
    HTTP2_0,
}


// create a rust struct
#[derive(Debug)]
pub struct HttpRequest {
    pub version: Version,
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}