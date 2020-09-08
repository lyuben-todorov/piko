
use std::sync::{mpsc};
use std::sync::mpsc::{Sender, Receiver};
use rayon::prelude::*;
use std::net::{SocketAddr, TcpStream};
use crate::proto::{ProtoParcel, Type, Body};
use crate::net::{write_parcel, read_parcel};

pub fn seq_recovery(neighbour_list: Vec<SocketAddr>, _id: u16) -> u8 {

    let (sender, receiver): (Sender<u8>, Receiver<u8>) = mpsc::channel(); // setup channel for results

    let req = ProtoParcel::seq_req();

    // begin parallel scope
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        recover(&addr, &req, s);
    });
    // end parallel scope

    let max_seq = receiver.iter().max_by_key(|seq| *seq).unwrap();
    println!("Recovered sequence number {}", max_seq);
    max_seq
}

fn recover(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<u8>) {
    println!("Recovering sequence from {}", host);

    let mut tcp_stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}: {}", err, host);
            return;
        }
    };
    write_parcel(&mut tcp_stream, &req_parcel);
    let res_parcel = read_parcel(&mut tcp_stream);
    match res_parcel.parcel_type {
        Type::SeqRes => {
            if let Body::SeqRes { seq_number } = res_parcel.body {
                tx.send(seq_number).unwrap();
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