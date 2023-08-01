mod core;

use crate::core::dns::udp_listener::UdpListener;


#[tokio::main]
async fn main() {
    let listener = UdpListener::new();
    let _ = listener.start("0.0.0.0:1053").await;
}
