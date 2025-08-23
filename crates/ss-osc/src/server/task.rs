use xrc_commons::Result;

use tokio_util::sync::CancellationToken;
use tokio::net::{
    UdpSocket,
};
use tokio::sync::{broadcast};
use std::net::{
    SocketAddr,
};

#[derive(Debug)]
pub enum OscServerEvent {
    InboundPacket {
        packet: rosc::OscPacket,
        from: SocketAddr,
    },
}

pub(crate) struct OscServerTask {
    socket_addr: SocketAddr,
    cancellation_token: CancellationToken,
    event_sender: broadcast::Sender<OscServerEvent>,
}

impl OscServerTask {
    pub(crate) fn new(
        socket_addr: SocketAddr,
        cancellation_token: CancellationToken,
        event_sender: broadcast::Sender<OscServerEvent>,
    ) -> Self {
        Self {
            socket_addr,
            cancellation_token,
            event_sender,
        }
    }

    pub(crate) async fn run(
        &mut self,
    ) -> Result<()> {
        // todo: bind to multiple addresses (IPv4 + IPv6)
        let udp_socket = UdpSocket::bind(self.socket_addr).await?;

        // todo: bind TCP socket

        let mut buf = [0u8; rosc::decoder::MTU];

        loop {
            tokio::select! {
                result = udp_socket.recv_from(&mut buf) => {
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