extern crate threadpool;
extern crate glob;
extern crate config;

use config::*;
use threadpool::ThreadPool;

use std::net::{TcpListener, Ipv4Addr};
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use piko::discovery::dsc;
use piko::state::Mode::{SHUTDOWN, DSC};
use piko::state::{Mode, State, Node};
use std::collections::HashMap;
use std::hash::Hash;

fn main() {
    let mut settings = Config::default();
    settings.merge(File::with_name("conf/prtkl.toml")).expect("Couldn't open config file!");

    let name = settings
        .get_str("node.name")
        .expect("Missing node name.");
    let port = settings
        .get_int("node.port")
        .expect("Missing node port.") as u16;
    let host_name = settings
        .get_str("node.host")
        .expect("Missing node hostname");
    let thread_count = settings
        .get_int("node.thread_count")
        .expect("Missing node thread_count.");

    let neighbour_host_names_settings = settings
        .get_array("cluster.neighbours")
        .expect("Missing cluster settings.");
    let mut neighbour_host_names = Vec::new();
    for result in neighbour_host_names_settings {
        neighbour_host_names.push(result.into_str().expect("Error parsing cluster node entry."));
    }

    let thread_pool = ThreadPool::new(thread_count as usize);
    println!("Starting worker pool with count {}.", thread_count);


    let addr = Ipv4Addr::from_str(&host_name).expect("Error parsing host name");
    let address = SocketAddr::from((addr, port));
    let listener = TcpListener::bind(address);
    let listener = match listener {
        Ok(listener) => {
            println!("Bound to port {}.", port);
            listener
        }
        Err(error) => panic!("Error during port binding: {}", error),
    };
    println!("Node {} accepting connections.", name);
    println!("Starting main worker process.");
    let neighbours = HashMap::<String, Node>::new();

    let mut state: State = State::new(name, Mode::DSC, neighbours);

    loop {
        println!("Loop");

        match &state.mode {
            Mode::DSC => {
                dsc(&thread_pool, &mut state, &neighbour_host_names);
                continue;
            }
            Mode::WRK => {}
            Mode::ERR => {}
            Mode::PANIC => {}
            Mode::SHUTDOWN => {
                println!("Bye!");
                break;
            }
        }
    }


    println!("Shutting down.");
}