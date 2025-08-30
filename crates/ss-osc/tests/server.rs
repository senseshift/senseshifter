use regex::Regex;
use rosc::{decoder, encoder, OscMessage, OscPacket};
use ss_osc::server::{
  config::{
    OscServerConfig, OscServerSocketConfig, RouterForwardConfig, RouterForwardTargetConfig,
    RouterRouteConfig,
  },
  connection_manager::ConnectionManagerConfig,
  OscServerBuilder,
};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::time::{timeout, Duration};
use tokio::{net::UdpSocket, sync::mpsc};

async fn start_udp_server() -> (
  SocketAddr,
  mpsc::Receiver<OscPacket>,
  tokio::task::JoinHandle<()>,
) {
  let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
  let addr = socket.local_addr().unwrap();
  let (tx, rx) = mpsc::channel(8);
  let handle = tokio::spawn(async move {
    let mut buf = [0u8; decoder::MTU];
    loop {
      match socket.recv_from(&mut buf).await {
        Ok((len, _)) => {
          let (_, packet) = decoder::decode_udp(&buf[..len]).unwrap();
          if tx.send(packet).await.is_err() {
            break;
          }
        }
        Err(_) => break,
      }
    }
  });
  (addr, rx, handle)
}

#[tokio::test]
async fn routes_packets_to_multiple_targets() -> Result<(), Box<dyn std::error::Error>> {
  let (addr_a, mut rx_a, handle_a) = start_udp_server().await;
  let (addr_b, mut rx_b, handle_b) = start_udp_server().await;

  let temp_socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
  let proxy_addr = temp_socket.local_addr()?;
  drop(temp_socket);

  let config = OscServerConfig {
    server: OscServerSocketConfig {
      udp: vec![proxy_addr],
      tcp: vec![],
    },
    routes: vec![
      RouterRouteConfig {
        address: Regex::new(r"^/a$")?,
        stop_propagation: true,
        forward: vec![RouterForwardConfig {
          target: RouterForwardTargetConfig::udp(addr_a),
          rewrite: None,
        }],
      },
      RouterRouteConfig {
        address: Regex::new(r"^/b$")?,
        stop_propagation: true,
        forward: vec![RouterForwardConfig {
          target: RouterForwardTargetConfig::udp(addr_b),
          rewrite: None,
        }],
      },
    ],
    connection_manager: ConnectionManagerConfig::default(),
  };

  let server = OscServerBuilder::new(config).build()?;
  let (_, mut conn_events) = server.subscribe_to_events();
  let mut connected = 0;
  while connected < 2 {
    if let Ok(event) = timeout(Duration::from_secs(5), conn_events.recv()).await? {
      if matches!(
        event,
        ss_osc::server::connection_manager::ConnectionEvent::Connected { .. }
      ) {
        connected += 1;
      }
    }
  }

  let client = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
  client.connect(proxy_addr).await?;

  let msg_a = OscPacket::Message(OscMessage {
    addr: "/a".to_string(),
    args: vec![],
  });
  client.send(&encoder::encode(&msg_a)?).await?;
  let received_a = timeout(Duration::from_secs(2), rx_a.recv()).await?;
  let received_a = received_a.ok_or("server A did not receive")?;
  match received_a {
    OscPacket::Message(msg) => assert_eq!(msg.addr, "/a"),
    _ => panic!("expected message"),
  }
  assert!(rx_b.try_recv().is_err());

  let msg_b = OscPacket::Message(OscMessage {
    addr: "/b".to_string(),
    args: vec![],
  });
  client.send(&encoder::encode(&msg_b)?).await?;
  let received_b = timeout(Duration::from_secs(2), rx_b.recv()).await?;
  let received_b = received_b.ok_or("server B did not receive")?;
  match received_b {
    OscPacket::Message(msg) => assert_eq!(msg.addr, "/b"),
    _ => panic!("expected message"),
  }
  assert!(rx_a.try_recv().is_err());

  server.shutdown().await;
  handle_a.abort();
  handle_b.abort();

  Ok(())
}
