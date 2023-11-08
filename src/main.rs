use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(mut stream: TcpStream) {
    let buf_readr = BufReader::new(&stream);
    let _request: Vec<String> = buf_readr
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let path_line: &str = _request
        .iter()
        .filter(|line| line.starts_with("GET"))
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap();

    // println!("Request: {:#?}", _request);
    // println!("Path: {:#?}", path);
    let response = match path_line {
        "/" => "HTTP/1.1 200 OK\r\n\r\n".as_bytes(),
        _ => "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes(),
    };
    if let Err(e) = stream.write_all(response) {
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
