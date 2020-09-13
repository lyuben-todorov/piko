use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::net::{SocketAddr, TcpStream};
use crate::proto::{ProtoParcel};
use rayon::prelude::*;
use crate::net::{write_parcel, read_parcel, ack};
use crate::state::Mode;
use crate::internal::ThreadSignal;
use log::{debug, error, info, trace, warn};

/*
    Pushes state update to each host given.
 */
pub fn push_state(neighbour_list: &Vec<SocketAddr>, state: Mode) {
    let (sender, _receiver): (Sender<ThreadSignal>, Receiver<ThreadSignal>) = mpsc::channel(); // setup channel for results

    let req = ProtoParcel::state_change(state);

    // begin parallel scope
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        update(&addr, &req, s);
    });
    // end parallel scope
}

fn update(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<ThreadSignal>) {
    info!("Pushing update to {}", host);
    let mut tcp_stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            error!("{}: {}", err, host);
            return;
        }
    };
    let m_id = req_parcel.id;
    write_parcel(&mut tcp_stream, &req_parcel);
    let res_parcel = read_parcel(&mut tcp_stream);
    let result = ack(res_parcel, m_id);
    tx.send(result).unwrap();
}