use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:42069").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => println!("Error accepting incoming connection: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    if request_line == "POST /record HTTP/1.1" {
        // TODO
        stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
    } else if request_line == "GET /live HTTP/1.1" {
        // TODO
        stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
    } else {
        stream.write_all("HTTP/1.1 404 WTF\r\n\r\n".as_bytes()).unwrap();
    }
}