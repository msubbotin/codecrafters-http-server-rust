use std::fs::File;
use std::io::Read;
use std::{env, fs, thread};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use itertools::Itertools;
use nom::AsBytes;
#[derive(Debug)]
enum Request {
    GET {
        path: String,
        user_agent: Option<String>,
    },
    POST {
        path: String,
        body: Vec<u8>,
    },
}

fn parse_request(stream: &mut TcpStream) -> Option<Request> {
    let reader = BufReader::new(stream.try_clone().unwrap());

    let lines: Vec<String> = reader
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("lines: {:?}", lines);
    let user_agent: Option<String> = lines
        .iter()
        .filter(|line| line.starts_with("User-Agent"))
        .map(|line| line.split_whitespace())
        .flatten()
        .map_into()
        .nth(1);

    let request_path: Vec<&str> = lines
        .first()
        .map(|line| line.split_whitespace().collect_vec())
        .unwrap_or_default();

    if let [type_request, _path] = request_path.as_slice()[..2] {
        let path = String::from(_path.split_once('/').unwrap_or_default().1);
        return match type_request {
            "GET" => Some(Request::GET {
                path: String::from(path),
                user_agent,
            }),
            "POST" => {
                let body_length: usize = lines
                    .iter()
                    .filter(|line| line.starts_with("Content-Length"))
                    .map(|line| line.split_whitespace())
                    .flatten()
                    .map(|value| value.parse::<usize>().unwrap())
                    .nth(1)
                    .unwrap_or_default();
                let mut buffer = vec![0; body_length];
                let mut reader = BufReader::new(stream);
                reader.read_exact(&mut buffer).unwrap(); //New Vector with size of Content

                Some(Request::POST {
                    path: String::from(path),
                    body: buffer,
                })
            }
            _ => None,
        };
    }
    None
}

fn ok_response(content: String, content_type: &str) -> Option<String> {
    let mut responce = String::new();
    responce.push_str("HTTP/1.1 200 OK\r\nContent-Type: ");
    responce.push_str(content_type);
    responce.push_str("\r\nContent-Length: ");
    responce.push_str(content.len().to_string().as_str());
    if content.len() != 0 {
        responce.push_str("\r\n\r\n");
        responce.push_str(content.as_str());
    }
    responce.push_str("\r\n\r\n");
    Some(responce)
}

fn get_response(path: String, dir_path: &String, user_agent: Option<String>) -> Option<String> {
    if path.is_empty() {
        return ok_response(String::new(), "text/plain");
    } else if path.starts_with("user-agent") {
        return ok_response(user_agent.unwrap_or_default(), "text/plain");
    }

    match path.split_once('/') {
        Some(("echo", other)) => ok_response(other.to_string(), "text/plain"),
        Some(("files", file_name)) => {
            let file = fs::read_to_string(format!("{}/{}", dir_path, file_name));
            if let Ok(_content) = file {
                ok_response(_content, "application/octet-stream")
            } else {
                None
            }
        }
        _ => None,
    }
}

fn post_response(path: String, dir_path: &String, body: Vec<u8>) -> Option<String> {
    match path.split_once('/') {
        Some(("files", file_name)) => {
            let file = File::create(format!("{}/{}", dir_path, file_name));
            if let Err(_) = file.unwrap().write_all(body.as_bytes()) {
                None
            } else {
                Some(String::from(
                    "HTTP/1.1 201 OK\r\nContent-Type: text/plain\r\n\r\n",
                ))
            }
        }
        _ => None,
    }
}

fn make_response(request: Option<Request>, dir_path: &String) -> Option<String> {
    println!("{:?}", request);
    match request {
        Some(Request::GET { path, user_agent }) => get_response(path, dir_path, user_agent),
        Some(Request::POST { path, body }) => post_response(path, dir_path, body),
        _ => None,
    }
}

fn handle_connection(mut stream: TcpStream, dir_path: &String) {
    let response = match make_response(parse_request(&mut stream), dir_path) {
        Some(_response) => _response,
        None => String::from("HTTP/1.1 404 Not Found\r\n\r\n"),
    };

    if let Err(e) = stream.write_all(response.as_bytes()) {
        println!("Error stream writer: {}", e);
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    let dir_path: String = env::args().nth(2).unwrap_or_default();
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let val = dir_path.clone();
                thread::spawn(move || handle_connection(_stream, &val));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
