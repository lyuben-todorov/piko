use std::net::{SocketAddr, TcpStream};
use crate::proto::{ResourceRelease, ProtoParcel};
use std::sync::mpsc::{Sender, Receiver};
use crate::internal::TaskSignal;
use std::sync::mpsc;
use rayon::prelude::*;
use crate::net::{write_parcel, read_parcel, is_acked};
use log::{error, info, debug};

pub fn replicate_message(neighbour_list: &Vec<SocketAddr>, _resource: ResourceRelease) {
    let (sender, _receiver): (Sender<TaskSignal>, Receiver<TaskSignal>) = mpsc::channel(); // setup channel for results

    let req = ProtoParcel::ack(1);

    // begin parallel scope
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        update(&addr, &req, s);
    });
    // end parallel scope
}

fn update(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<TaskSignal>) {
    debug!("Pushing event to {}", host);
    let mut stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            error!("{}: {}", err, host);
            return;
        }
    };
    let m_id = req_parcel.id;
    write_parcel(&mut stream, &req_parcel);

    let res_parcel = match read_parcel(&mut stream) {
        Ok(parcel) => parcel,
        Err(e) => {
            error!("Invalid parcel! {}", e);
            return;
        }
    };
    let result = is_acked(res_parcel, m_id);
    tx.send(result).unwrap();
}
