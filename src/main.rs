use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use itertools::Itertools;
#[derive(Debug)]
enum Request {
    GET {
        path: String,
        user_agent: Option<String>,
    },
    POST(String),
}

fn parse_path(lines: &Vec<String>) -> Option<Request> {
    println!("{:?}", lines);
    let user_agent: Option<String> = lines
        .iter()
        .filter(|line| line.starts_with("User-Agent"))
        .map(|line| line.split_whitespace())
        .flatten()
        .map(|value| String::from(value))
        .nth(1);
    if let Some(line) = lines.first() {
        if let [request_type, path] = line.split_whitespace().collect_vec().as_slice()[0..=1] {
            let path = String::from(path.split_once('/').unwrap_or_default().1);
            return match request_type {
                "GET" => Some(Request::GET {
                    path: path,
                    user_agent: user_agent,
                }),
                "POST" => Some(Request::POST(path)),
                _ => None,
            };
        }
    }
    None
}

fn make_responce(request: Option<Request>) -> Option<String> {
    println!("{:?}", request);
    match request {
        Some(Request::GET { path, user_agent }) => {
            if path.starts_with("echo") {
                match path.split_once('/') {
                    Some(("echo", other)) => Some(format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
                    other.len(),
                    other
                )),
                    None => None,
                    _ => None,
                }
            } else if path.starts_with("user-agent") {
                if let Some(_user_agent) = user_agent {
                    Some(format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
                        _user_agent.len(),
                        _user_agent
                    ))
                } else {
                    None
                }
            } else if path.is_empty() {
                Some(String::from("HTTP/1.1 200 OK\r\n\r\n"))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn handle_connection(mut stream: TcpStream) {
    let request: Vec<String> = BufReader::new(&stream)
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let response = match make_responce(parse_path(&request)) {
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

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                handle_connection(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
