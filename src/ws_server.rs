use super::settings;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::thread::spawn;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<RwLock<HashMap<SocketAddr, Tx>>>;

pub fn spawn_ws() {
    spawn(|| {
        ws_server();
    });
}

lazy_static! {
    static ref SUBSCRIBERS: PeerMap = PeerMap::new(RwLock::new(HashMap::new()));
}

#[tokio::main]
async fn ws_server() {
    let cfg = &settings::ServerSettings::shared().http;
    let host = &cfg.bind;
    let port = &cfg.ws_port;
    info!("Start WS at {}:{}", host, port);

    let try_socket = TcpListener::bind(format!("{}:{}", host, port)).await;
    let listener = try_socket.expect("Failed to bind");

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr));
    }
}

async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (tx, rx) = unbounded();
    SUBSCRIBERS.write().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let handle_incoming = incoming.try_for_each(|_| future::ok(()));

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(handle_incoming, receive_from_others);
    future::select(handle_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    SUBSCRIBERS.write().unwrap().remove(&addr);
}

pub fn ws_push(msg: &impl serde::Serialize) {
    let ser = serde_json::to_string(msg).unwrap();
    info!("Broadcast WS msg: {}", ser);
    let peers = SUBSCRIBERS.read().unwrap();

    let broadcast_recipients = peers.iter().map(|(_, ws_sink)| ws_sink);

    for recp in broadcast_recipients {
        recp.unbounded_send(Message::Text(ser.clone())).unwrap();
    }
}
