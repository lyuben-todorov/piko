use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::net::{SocketAddr, TcpStream};
use crate::proto::{ProtoParcel, Type, Body};
use rayon::prelude::*;
use crate::net::{write_parcel, read_parcel};
use crate::state::Mode;
use crate::internal::ThreadSignal;

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
    println!("Pushing update to {}", host);
    let mut tcp_stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}: {}", err, host);
            return;
        }
    };
    let m_id = req_parcel.id;

    write_parcel(&mut tcp_stream, &req_parcel);

    let res_parcel = read_parcel(&mut tcp_stream);

    match res_parcel.parcel_type {
        Type::Ack => {
            if let Body::Ack { message_id } = res_parcel.body {
                if message_id == m_id {
                    println!("Acked {}", message_id);
                    tx.send(ThreadSignal::Success).unwrap();
                } else {
                    tx.send(ThreadSignal::Fail).unwrap();
                }
            } else {
                println!("Body-header type mismatch!");
                return;
            }
        }
        Type::ProtoError => {}
        _ => {
            println!("Unexpected response type to SeqReq, {}", res_parcel.parcel_type);
            return;
        }
    }
}