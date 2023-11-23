use anyhow::{bail, Result};
use itertools::Itertools;
use std::borrow::Cow;
use std::fmt::Display;
use std::fs::File;
use std::{env, fs, thread};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

const LINE_DELIMITER: &str = "\r\n";
const BODY_DELIMITER: &str = "\r\n\r\n";

#[derive(Debug, Clone)]
struct HttpRequest<'a> {
    pub request_type: RequestType,
    user_agent: Cow<'a, str>,
    path: Cow<'a, str>,
    body: Cow<'a, str>,
}

impl<'a> HttpRequest<'a> {
    fn try_new(stream: &TcpStream) -> Result<HttpRequest<'a>> {
        let received: Vec<u8> = BufReader::new(stream)
            .fill_buf()
            .unwrap_or_default()
            .to_vec();

        // convert to utf8 String
        let full_request: String = String::from_utf8(received)?;

        // parse to get request_type, path, vesion, user_agent and body
        if let Some((head, body)) = full_request.split_once(BODY_DELIMITER) {
            //request_type, path, vesion
            let first_line: Vec<String> = head
                .split(LINE_DELIMITER)
                .collect_vec()
                // it always the first line
                .first()
                .map(|line| line.split_whitespace().collect_vec())
                .unwrap()
                .iter()
                .map(|s| s.to_string())
                .collect_vec();
            // need to be always
            let request_type: &str = first_line.get(0).unwrap();
            // need to be always
            // skip 1 because the first element always is empty
            let path = first_line.get(1).unwrap();

            let user_agent: String = head
                .split(LINE_DELIMITER)
                .collect_vec()
                // it always the third line
                .get(2)
                .map(|line| line.split_whitespace().collect_vec())
                .unwrap()
                .get(1)
                .unwrap()
                .to_string();

            return Ok(HttpRequest {
                request_type: request_type.into(),
                user_agent: user_agent.into(),
                path: path.to_owned().into(),
                body: body.to_owned().into(),
            });
        }
        bail!("Can't parse request {}", full_request)
    }

    fn path_root(&self) -> &str {
        let value: Vec<&str> = self.path.split("/").skip(1).take(1).collect();
        if value.is_empty() {
            ""
        } else {
            value[0]
        }
    }
    fn path_other(&self) -> String {
        let value: Vec<&str> = self.path.split("/").skip(2).collect();
        value.join("/")
    }
}

#[derive(Debug, Clone, Copy)]
enum RequestType {
    GET,
    POST,
}

impl From<&str> for RequestType {
    fn from(value: &str) -> RequestType {
        match value {
            // expect only 2 request tupes GET and POST
            "POST" => RequestType::POST,
            _ => RequestType::GET,
        }
    }
}

#[derive(Debug)]
struct HttpResponce<'a> {
    version: &'a str,
    status_code: &'a str,
    content_type: &'a str,
    body: Option<String>,
}

impl<'a> HttpResponce<'a> {
    fn ok(body: Option<String>) -> Self {
        Self {
            version: "HTTP/1.1",
            status_code: "200 OK",
            content_type: "text/plain",
            body: body,
        }
    }

    fn ok_201(body: Option<String>) -> Self {
        Self {
            version: "HTTP/1.1",
            status_code: "201 OK",
            content_type: "text/plain",
            body: body,
        }
    }
    fn not_found() -> Self {
        Self {
            version: "HTTP/1.1",
            status_code: "404 Not Found",
            content_type: "text/plain",
            body: None,
        }
    }
}

impl<'a> Display for HttpResponce<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content_length = self.body.as_ref().unwrap_or(&String::default()).len();
        let body: String = match self.body.as_ref() {
            Some(value) => format!("{}{}", value, BODY_DELIMITER),
            None => String::from(""),
        };
        write!(
            f,
            "{} {}\r\nContent-Type: {}\r\nContent-Length: {}{}{}",
            self.version, self.status_code, self.content_type, content_length, BODY_DELIMITER, body
        )
    }
}

fn request_mapping<'a>(request: HttpRequest, dir_path: &'a str) -> HttpResponce<'a> {
    match (request.request_type, request.path_root()) {
        (RequestType::GET, "") => HttpResponce::ok(None),
        (RequestType::GET, "user-agent") => HttpResponce::ok(Some(request.user_agent.to_string())),
        (RequestType::GET, "echo") => HttpResponce::ok(Some(request.path_other())),
        (RequestType::GET, "files") => {
            match fs::read_to_string(format!("{}/{}", dir_path, request.path_other())) {
                Ok(file) => HttpResponce {
                    content_type: "application/octet-stream",
                    ..HttpResponce::ok(Some(file))
                },
                Err(_) => HttpResponce::not_found(),
            }
        }
        (RequestType::POST, "files") => {
            let file = File::create(format!("{}/{}", dir_path, request.path_other()));
            match file.map(|mut file| file.write_all(request.body.as_bytes())) {
                Ok(_) => HttpResponce::ok_201(None),
                Err(_) => HttpResponce::not_found(),
            }
        }
        _ => HttpResponce::not_found(),
    }
}

fn request_processor(stream: &mut TcpStream, dir_path: &str) {
    println!("Get a new request.");
    // read all information from stream

    let response: HttpResponce = match HttpRequest::try_new(stream) {
        Ok(request) => request_mapping(request, dir_path),
        Err(_) => HttpResponce::not_found(),
    };

    if let Err(e) = stream.write_all(response.to_string().as_bytes()) {
        println!("Error stream writer: {}", e);
    } else {
        println!("Request process worked fine.");
    }
}

fn main() {
    println!("Logs from your program will appear here!");
    // parse dir folder name from args
    let dir_path: String = env::args().nth(2).unwrap_or_default();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let dir_path_clone = dir_path.clone();
                thread::spawn(move || request_processor(&mut _stream, &dir_path_clone));
            }
            Err(e) => {
                println!("Can't get stream from listener: {}", e);
            }
        }
    }
}
