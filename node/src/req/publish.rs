use std::net::{SocketAddr, TcpStream};
use crate::internal::TaskSignal;
use std::sync::mpsc::{Receiver, Sender};
use crate::proto::{ResourceRequest, ProtoParcel, ResourceRelease};
use std::sync::mpsc;
use rayon::prelude::*;
use crate::net::{write_parcel, read_parcel, is_acked};
use log::{info, error, debug};

pub fn pub_req(neighbour_list: &Vec<SocketAddr>, req: ResourceRequest) -> TaskSignal {
    let (sender, receiver): (Sender<TaskSignal>, Receiver<TaskSignal>) = mpsc::channel(); // setup channel for results

    let req = ProtoParcel::resource_request(req);

    // begin parallel scope
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        publish_request(&addr, &req, s);
    });
    // end parallel scope

    let total_acks = neighbour_list.len();
    let mut received_acks: usize = 0;
    for res in receiver.iter() {
        match res {
            TaskSignal::Success => {
                received_acks += 1;
                debug!("Acked {}/{}", received_acks, total_acks);

                if total_acks == received_acks {
                    return TaskSignal::Success;
                }
            }
            _ => {
                return TaskSignal::Fail;
            }
        }
    }
    TaskSignal::Fail
}

fn publish_request(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<TaskSignal>) {
    debug!("Pushing request to {}", host);
    let mut stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            error!("{}: {}", err, host);
            tx.send(TaskSignal::Fail).unwrap();
            return;
        }
    };

    let m_id = req_parcel.id;
    write_parcel(&mut stream, &req_parcel);

    let res_parcel = match read_parcel(&mut stream) {
        Ok(parcel) => parcel,
        Err(e) => {
            error!("Invalid parcel! {}", e);
            tx.send(TaskSignal::Fail).unwrap();
            return;
        }
    };

    let result = is_acked(res_parcel, m_id);
    tx.send(result).unwrap();
}

pub fn pub_rel(neighbour_list: &Vec<SocketAddr>, rel: ResourceRelease) {
    let (sender, _receiver): (Sender<TaskSignal>, Receiver<TaskSignal>) = mpsc::channel(); // setup channel for results

    let req = ProtoParcel::resource_release(rel);

    // begin parallel scope
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        publish_release(&addr, &req, s);
    });
    // end parallel scope
}

fn publish_release(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<TaskSignal>) {
    debug!("Pushing release to {}", host);
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