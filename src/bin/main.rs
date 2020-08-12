extern crate glob;
extern crate config;
extern crate rayon;

use config::*;

use std::net::{TcpListener, Ipv4Addr, SocketAddrV4};
use std::net::SocketAddr;
use std::str::FromStr;
use piko::discovery::dsc;
use piko::state::{Mode, State, Node};
use std::collections::HashMap;
use std::env;
use std::env::current_dir;
use std::path::{Path, PathBuf};

fn main() {
    let mut settings = Config::default();

    let argv: Vec<String> = env::args().collect();

    let mut config_path = current_dir().expect("Couldn't get working directory");

    println!("{}", argv.len());
    if argv.len() > 1 {
        let arg1 = argv.get(1).unwrap();
        let arg2 = argv.get(2).unwrap();

        println!("{}  {}", arg1, arg2);

        if arg1 == "-p" {
            config_path = PathBuf::from(arg2);
        }
    } else {
        config_path.push("conf/prtkl.toml");
    }

    println!("{}", config_path.to_str().unwrap());
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

    let neighbour_host_names_settings = settings
        .get_array("cluster.neighbours")
        .expect("Missing cluster settings.");
    let mut neighbour_host_names = Vec::new();
    for result in neighbour_host_names_settings {
        neighbour_host_names.push(result.into_str().expect("Error parsing cluster node entry."));
    }

    rayon::ThreadPoolBuilder::new().num_threads(thread_count as usize).build_global().unwrap();
    println!("Setting worker pool count to {}.", thread_count);


    let addr = Ipv4Addr::from_str(&host_name).expect("Error parsing host name");
    let address = SocketAddr::from(SocketAddrV4::new(addr, port as u16));
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
                dsc(&mut state, &neighbour_host_names);
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