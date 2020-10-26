use std::net::{SocketAddr};
use crate::proto::{ProtoParcel, Body};
use crate::net::{write_parcel, read_parcel};

use log::{error};
use tokio::net::TcpStream;

// Returns the route through which the sender is contacted
pub async fn get_ext_addr_from_neighbour(host: &SocketAddr) -> Option<SocketAddr> {
    let req = ProtoParcel::ext_addr_req();

    let mut stream = match TcpStream::connect(host).await {
        Ok(stream) => stream,
        Err(err) => {
            error!("{}: {}", err, host);
            return None;
        }
    };

    write_parcel(&mut stream, &req);

    match read_parcel(&mut stream).await {
        Ok(res) => {
            if let Body::ExtAddrRes { addr } = res.body {
                Some(addr)
            } else {
                error!("Body-header type mismatch!");
                None
            }
        }
        Err(err) => {
            error!("{}: {}", err, host);
            None
        }
    }
}