extern crate config;
extern crate rayon;
extern crate sha2;
extern crate num;
extern crate num_derive;
extern crate lazy_static;
extern crate log;
extern crate fern;

use config::*;

use std::net::{TcpListener, Ipv4Addr, SocketAddrV4, ToSocketAddrs};
use std::net::SocketAddr;
use std::str::FromStr;
use piko::dsc::dsc;
use piko::state::{Mode, State, Node};
use std::collections::{HashMap, BinaryHeap};
use std::env;
use std::env::current_dir;
use std::path::{PathBuf};

use piko::net::listener_thread;
use std::sync::{Arc, mpsc, RwLock, Mutex};
use std::sync::mpsc::{Sender, Receiver};

use piko::internal::ThreadSignal;
use piko::proto::{set_sender_id, ResourceRequest};

use fern::colors::{Color, ColoredLevelConfig};
use log::{info};

use piko::heartbeat::heartbeat;
use piko::client::client_listener;
use piko::wrk::wrk;

fn main() {
    setup_logger();

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
    let socket_name = settings
        .get_str("node.socket")
        .expect("Missing main socket name");
    let client_socket_name = settings
        .get_str("node.client_socket")
        .expect("Missing client socket name");
    let thread_count = settings
        .get_int("node.thread_count")
        .expect("Missing node thread_count.");
    let neighbour_host_names = settings
        .get_array("cluster.neighbours")
        .expect("Missing cluster settings.");
    let mut neighbour_socket_addresses: Vec<SocketAddr> = Vec::new();

    for result in neighbour_host_names {
        let addr = result.into_str().expect("Error parsing cluster node entry.");
        let addr = addr.to_socket_addrs().expect("Error parsing host name").next().unwrap();
        neighbour_socket_addresses.push(addr);
    }

    let addr = socket_name.to_socket_addrs().expect("Error parsing host name").next().unwrap();
    let client_addr = client_socket_name.to_socket_addrs().expect("Error parsing client host name").next().unwrap();

    rayon::ThreadPoolBuilder::new().num_threads(thread_count as usize).build_global().unwrap();
    info!("Setting worker pool count to {}.", thread_count);

    let cluster_socket = match TcpListener::bind(addr) {
        Ok(listener) => {
            info!("Listening to cluster on {}.", listener.local_addr().unwrap());
            listener
        }
        Err(error) => panic!("Error binding cluster socket: {}", error),
    };

    let client_socket = match TcpListener::bind(client_addr) {
        Ok(listener) => {
            info!("Listening to clients on {}.", listener.local_addr().unwrap());
            listener
        }
        Err(error) => panic!("Error binding client socket: {}", error),
    };

    let neighbours = HashMap::<u16, Node>::new();
    let self_node_information = Node::new(name, Mode::Dsc, addr);
    set_sender_id(self_node_information.id);


    // Initiate state & shared data structures
    let state = Arc::new(RwLock::new(State::new(self_node_information, neighbours)));
    let pledge_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>> = Arc::new(Mutex::new(BinaryHeap::new()));
    let f_lock: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    // Start network listener thread
    let state_ref = state.clone();
    let pledge_queue_ref = pledge_queue.clone();
    let f_lock_ref = f_lock.clone();
    rayon::spawn(move || listener_thread(cluster_socket, state_ref, pledge_queue_ref, f_lock_ref));

    // Start client listener thread
    let state_ref = state.clone();
    let pledge_queue_ref = pledge_queue.clone();
    let f_lock_ref = f_lock.clone();
    rayon::spawn(move || client_listener(client_socket, state_ref, pledge_queue_ref, f_lock_ref));

    // Start heartbeat thread
    let state_ref = state.clone();
    let (_monitor_sender, monitor_receiver): (Sender<ThreadSignal>, Receiver<ThreadSignal>) = mpsc::channel();
    rayon::spawn(move || heartbeat(state_ref, 5, 5, monitor_receiver));


    info!("Started main worker thread!");
    loop {
        let state_lock = state.read().unwrap();
        let mode = &state_lock.self_node_information.mode;
        info!("Mode: {}", mode);
        match mode {
            Mode::Dsc => {
                drop(state_lock);
                dsc(state.clone(), &neighbour_socket_addresses);
            }
            Mode::Wrk => {
                drop(state_lock);
                wrk(state.clone());
            }
            Mode::Err => {}
            Mode::Panic => {}
            Mode::Shutdown => {
                info!("Bye!");
                break;
            }
            _ => {}
        }
    }


    info!("Shutting down.");
}

fn setup_logger() {
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let colors_level = colors_line.clone().info(Color::Green);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        .level(log::LevelFilter::Debug)
        // output to stdout
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}