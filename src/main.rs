use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use itertools::Itertools;
#[derive(Debug)]
struct Request {
    request_type: String,
    path: Vec<String>,
}

fn parse_path(lines: &Vec<String>) -> Option<Request> {
    if let Some(line) = lines.first() {
        if let [request_type, path] = line.split_whitespace().collect_vec().as_slice()[0..=1] {
            return Some(Request {
                request_type: String::from(request_type),
                path: path
                    .split('/')
                    .skip(1)
                    .map(|value| String::from(value))
                    .collect_vec(),
            });
        }
    }
    None
}

fn make_responce(request: Option<Request>) -> Option<String> {
    if request.is_none() {
        return None;
    }
    let value = request.unwrap();
    println!("{value:?}");
    if value.path.len() < 1 {
        return None;
    }
    let main_path = value.path[0].as_str();
    return match main_path {
        "echo" => {
            if let Some(v) = value.path.get(1) {
                Some(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n{}\r\n\r\n", v.as_bytes().len(), v))
            } else {
                None
            }
        }
        "" => Some(String::from("HTTP/1.1 200 OK\r\n\r\n")),
        _ => None,
    };
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
