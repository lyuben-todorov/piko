extern crate glob;
extern crate config;
extern crate rayon;
extern crate bit_vec;
extern crate sha2;
extern crate byteorder;
extern crate bytes;
extern crate num;
extern crate num_derive;

use config::*;

use std::net::{TcpListener, Ipv4Addr, SocketAddrV4};
use std::net::SocketAddr;
use std::str::FromStr;
use piko::dsc::dsc;
use piko::state::{Mode, State, Node};
use std::collections::HashMap;
use std::env;
use std::env::current_dir;
use std::path::{PathBuf};

use piko::net::listener_thread;
use std::sync::{Arc, mpsc, RwLock};
use std::sync::mpsc::{Sender, Receiver};
use piko::wrk::wrk;


fn main() {
    let mut settings = Config::default();

    let argv: Vec<String> = env::args().collect();

    let mut config_path = current_dir().expect("Couldn't get working directory");

    if argv.len() > 1 {
        let arg1 = argv.get(1).unwrap();
        let arg2 = argv.get(2).unwrap();

        if arg1 == "-p" {
            config_path = PathBuf::from(arg2);
        }
    } else {
        config_path.push("conf/prtkl.toml");
    }

    settings.merge(File::from(config_path)).expect("Couldn't open config file!");

    let name = settings
        .get_str("node.name")
        .expect("Missing node name.");

    let port = settings
        .get_int("node.port")
        .expect("Missing node port.");
    let host_name = settings
        .get_str("node.host")
        .expect("Missing node hostname");
    let thread_count = settings
        .get_int("node.thread_count")
        .expect("Missing node thread_count.");

    let neighbour_host_names = settings
        .get_array("cluster.neighbours")
        .expect("Missing cluster settings.");
    let mut neighbour_socket_addresses: Vec<SocketAddr> = Vec::new();
    for result in neighbour_host_names {
        let neighbour_host_name = result.into_str().expect("Error parsing cluster node entry.");
        let socket = SocketAddr::from_str(&neighbour_host_name).unwrap();
        neighbour_socket_addresses.push(socket);
    }


    rayon::ThreadPoolBuilder::new().num_threads(thread_count as usize).build_global().unwrap();
    println!("Setting worker pool count to {}.", thread_count);


    let addr = Ipv4Addr::from_str(&host_name).expect("Error parsing host name");
    let address = SocketAddr::from(SocketAddrV4::new(addr, port as u16));
    let listener = TcpListener::bind(address);
    let listener = match listener {
        Ok(listener) => {
            println!("Bound to socket {}.", listener.local_addr().unwrap());
            listener
        }
        Err(error) => panic!("Error during port binding: {}", error),
    };


    let neighbours = HashMap::<String, Node>::new();
    let self_node_information = Node::new(name, Mode::DSC, host_name);


    let (_state_sender, listener_receiver): (Sender<u32>, Receiver<u32>) = mpsc::channel();
    let (listener_sender, _state_receiver): (Sender<u32>, Receiver<u32>) = mpsc::channel();

    let state_inner = State::new(self_node_information, neighbours);
    let state = Arc::new(RwLock::new(state_inner)); // pass sender to state

    let state_ref = state.clone();
    rayon::spawn(move || listener_thread(listener_receiver, listener_sender, state_ref, listener));
    // pass receiver to listener thread

    println!("Node {} initialized", state.read().unwrap().self_node_information.name);
    println!("Starting main worker process.");

    loop {
        let mode = &state.read().unwrap().self_node_information.mode;

        println!("Mode: {}", mode);
        match mode {
            Mode::DSC => {
                dsc(state.clone(), &neighbour_socket_addresses);
            }
            Mode::WRK => {
                wrk(state.clone());
            }
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