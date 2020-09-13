extern crate config;
extern crate rayon;
extern crate sha2;
extern crate num;
extern crate num_derive;
extern crate lazy_static;
extern crate log;
extern crate fern;

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

use piko::internal::ThreadSignal;
use piko::proto::{set_sender_id};

use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info, trace, warn};

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
    info!("Setting worker pool count to {}.", thread_count);

    let addr = Ipv4Addr::from_str(&host_name).expect("Error parsing host name");
    let address = SocketAddr::from(SocketAddrV4::new(addr, port as u16));
    let listener = TcpListener::bind(address);
    let listener = match listener {
        Ok(listener) => {
            info!("Bound to socket {}.", listener.local_addr().unwrap());
            listener
        }
        Err(error) => panic!("Error during port binding: {}", error),
    };


    let neighbours = HashMap::<u16, Node>::new();
    let self_node_information = Node::new(name, Mode::Dsc, address);
    set_sender_id(self_node_information.id);


    let (state_sender, listener_receiver): (Sender<ThreadSignal>, Receiver<ThreadSignal>) = mpsc::channel();

    let state_inner = State::new(self_node_information, neighbours);
    let state = Arc::new(RwLock::new(state_inner));

    // start listener thread
    let state_ref = state.clone();
    rayon::spawn(move || listener_thread(listener_receiver, state_ref, listener));
    // start monitor
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
                wrk(state.clone(), state_sender.clone());
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
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::White)
        // depending on the terminals color scheme, this is the same as the background color
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
        // set the default log level. to filter out verbose log messages from dependencies, set
        // this to Warn and overwrite the log level for your crate.
        .level(log::LevelFilter::Debug)
        // change log levels for individual modules. Note: This looks for the record's target
        // field which defaults to the module path but can be overwritten with the `target`
        // parameter:
        // `info!(target="special_target", "This log message is about special_target");`
        .level_for("pretty_colored", log::LevelFilter::Trace)
        // output to stdout
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    debug!("finished setting up logging! yay!");
}