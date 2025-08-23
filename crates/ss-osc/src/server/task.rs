use xrc_commons::Result;

use tokio_util::sync::CancellationToken;
use tokio::net::{
    UdpSocket,
    TcpListener,
};
use tokio::sync::{broadcast};
use std::net::{
    SocketAddr,
};
use derivative::Derivative;

#[derive(Debug)]
pub enum OscServerEvent {
    InboundPacket {
        packet: rosc::OscPacket,
        from: SocketAddr,
    },
}

#[derive(Debug)]
pub(crate) struct OscServerTask {
    udp_addrs: Vec<SocketAddr>,
    tcp_addrs: Vec<SocketAddr>,

    cancellation_token: CancellationToken,
    event_sender: broadcast::Sender<OscServerEvent>,
}

impl OscServerTask {
    pub(crate) fn new(
        udp_addrs: Vec<SocketAddr>,
        tcp_addrs: Vec<SocketAddr>,
        cancellation_token: CancellationToken,
        event_sender: broadcast::Sender<OscServerEvent>,
    ) -> Self {
        if udp_addrs.is_empty() && tcp_addrs.is_empty() {
            panic!("At least one UDP or TCP address must be provided for the OSC server.");
        }

        if tcp_addrs.len() > 0 {
            unimplemented!("TCP transport is not yet implemented.");
        }

        Self {
            udp_addrs,
            tcp_addrs,
            cancellation_token,
            event_sender,
        }
    }

    pub(crate) async fn run(
        &mut self,
    ) -> Result<()> {
        if self.udp_addrs.is_empty() && self.tcp_addrs.is_empty() {
            return Err(anyhow::anyhow!("At least one UDP or TCP address must be provided for the OSC server."));
        }

        let mut udp_socket = None;

        if self.udp_addrs.len() > 0 {
            udp_socket = Some(UdpSocket::bind(&self.udp_addrs[..]).await?);
            println!("OSC Server listening on UDP: {:?}", udp_socket.as_ref().unwrap().local_addr()?);
        }

        if self.tcp_addrs.len() > 0 {
            // todo: bind TCP sockets
            unimplemented!("TCP transport is not yet implemented.");
        }

        let mut buf = [0u8; rosc::decoder::MTU];

        loop {
            tokio::select! {
                result = udp_socket.as_ref().unwrap().recv_from(&mut buf), if udp_socket.is_some() => {
                    match result {
                        Ok((size, addr)) => {
                            self.handle_udp_packet(&buf[..size], &addr).await;
                        },
                        Err(e) => {
                            self.handle_error(anyhow::Error::new(e).context("Error receiving UDP packet")).await;
                        }
                    }
                },

                _ = self.cancellation_token.cancelled() => {
                    break;
                },
            }
        }

        Ok(())
    }

    async fn handle_udp_packet(&self, msg: &[u8], from: &SocketAddr) {
        // todo: add ACLs for filtering incoming packets

        match rosc::decoder::decode_udp(msg) {
            Ok((_, packet)) => {
                println!("Received packet from {}: {:?}", from, packet);

                match self.handle_osc_packet(&packet, from).await {
                    Ok(_) => {},
                    Err(e) => {
                        self.handle_error(e).await;
                    }
                }
            },
            Err(e) => {
                self.handle_error(anyhow::Error::new(e).context("Error decoding OSC packet")).await;
            }
        }
    }

    async fn handle_osc_packet(&self, packet: &rosc::OscPacket, from: &SocketAddr) -> Result<()> {
        let event = OscServerEvent::InboundPacket {
            packet: packet.clone(),
            from: *from,
        };

        match self.event_sender.send(event) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::Error::new(e).context("Error sending OSC server event")),
        }
    }

    async fn handle_error(&self, error: anyhow::Error) {
        // todo: improve error handling (e.g., report to Sentry, trace logs, etc.)

        eprintln!("OSC Server error: {}", error);
    }
}