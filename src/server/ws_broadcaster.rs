use ws::listen;

use std::net::Ipv4Addr;

use crate::ws::Error;

pub fn ws_broadcaster(ip: Ipv4Addr, port: u16) -> Result<(), Error> {
    Ok(listen(format!("{}:{}", ip, port), |out| {
        move |msg| out.broadcast(msg)
    })?)
}
