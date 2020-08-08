extern crate threadpool;
extern crate glob;
extern crate config;

use config::*;
use glob::glob;
use threadpool::ThreadPool;

use std::net::TcpListener;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::io::{Read, Write};
use std::fs;
use std::io::prelude::*;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::error::Error;

fn main() {
    let mut settings = Config::default();
    settings.merge(File::with_name("conf/prtkl.toml")).expect("Couldn't open config file!");

    let name = settings.get_str("node.name").expect("Missing node name.");
    let port = settings.get_str("node.port").expect("Missing node port.");
    let thread_count = settings.get_int("node.thread_count").expect("Missing node thread_count.");

    let thread_pool = ThreadPool::new(thread_count as usize);


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