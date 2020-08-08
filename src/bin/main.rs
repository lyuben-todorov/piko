use std::net::TcpListener;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::io::{Read, Write};
use std::fs;
use std::io::prelude::*;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;

fn main() {
    let port = 7878;
    let thread_count = 4;
    let thread_pool = ThreadPool::new(thread_count);


    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(address);

    let listener = match listener {
        Ok(listener) => {
            println!("Piko up on port {}", port);
            listener
        }
        Err(error) => panic!("Error during port binding: {}", error),
    };

    // Connection loop
    for stream in listener.incoming() {
        let mut buffer = [0; 1024];
        let mut stream = stream.unwrap();
        stream.read(&mut buffer).unwrap();

        thread_pool.execute(move || {
            println!("{}", String::from_utf8_lossy(&buffer));
        });
    }

    println!("Shutting down.");
}