use std::net::{SocketAddr, TcpStream};
use crate::internal::ThreadSignal;
use std::sync::mpsc::{Receiver, Sender};
use crate::proto::{ResourceRequest, ProtoParcel, ResourceRelease};
use std::sync::mpsc;
use rayon::prelude::*;
use crate::net::{write_parcel, read_parcel, is_acked};
use log::{info, error};

pub fn pub_req(neighbour_list: &Vec<SocketAddr>, req: ResourceRequest) {
    let (sender, receiver): (Sender<ThreadSignal>, Receiver<ThreadSignal>) = mpsc::channel(); // setup channel for results

    let req = ProtoParcel::resource_request(req);

    // begin parallel scope
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        publish_request(&addr, &req, s);
    });
    // end parallel scope

    for res in receiver.iter() {
        match res {
            ThreadSignal::Success => info!("Site acked req"),
            _ => {}
        }

    }
}

fn publish_request(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<ThreadSignal>) {
    info!("Pushing request to {}", host);
    let mut stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            error!("{}: {}", err, host);
            tx.send(ThreadSignal::Fail);
            return;
        }
    };

    let m_id = req_parcel.id;
    write_parcel(&mut stream, &req_parcel);

    let res_parcel = match read_parcel(&mut stream) {
        Ok(parcel) => parcel,
        Err(e) => {
            error!("Invalid parcel! {}", e);
            tx.send(ThreadSignal::Fail);
            return;
        }
    };

    let result = is_acked(res_parcel, m_id);
    tx.send(result).unwrap();
}

pub fn pub_rel(neighbour_list: &Vec<SocketAddr>, rel: ResourceRelease) {
    let (sender, _receiver): (Sender<ThreadSignal>, Receiver<ThreadSignal>) = mpsc::channel(); // setup channel for results

    let req = ProtoParcel::resource_release(rel);

    // begin parallel scope
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        publish_request(&addr, &req, s);
    });
    // end parallel scope
}

fn publish_release(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<ThreadSignal>) {
    info!("Pushing release to {}", host);
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