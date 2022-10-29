use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use backend::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:42069").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_connection(stream)
                });
            },
            Err(e) => println!("Error accepting incoming connection: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    match request_line.as_str() {
        "POST /record HTTP/1.1" => handle_record(stream),
        "GET /live HTTP/1.1" => handle_live(stream),
        _ => handle_404(stream),
    }
}

fn handle_record(mut stream: TcpStream) {
    // TODO
    stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
}

fn handle_live(mut stream: TcpStream) {
    // TODO
    stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).unwrap();
}

fn handle_404(mut stream: TcpStream) {
    stream.write_all("HTTP/1.1 404 WTF\r\n\r\n".as_bytes()).unwrap();
}