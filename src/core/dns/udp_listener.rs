use std::net::{SocketAddr, Ipv4Addr};
use std::sync::Arc;

use tokio::net::{UdpSocket, ToSocketAddrs};
use tokio::io::Result;

use crate::core::dns::dns_reader_writer::DnsWriter;

use super::dns_reader_writer::DnsReader;
use super::message::Message;
use super::message_builder::MessageBuilder;


pub struct UdpListener {
}

impl UdpListener {

    pub fn new() -> UdpListener {
        UdpListener {}
    }


    //
    pub async fn start<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        let socket = UdpSocket::bind(addr).await?;
        let socket = Arc::new(socket);
        
        loop {
            let mut buf = [0; 512];
            let info = socket.recv_from(&mut buf).await?;

            let socket = socket.clone();

            tokio::spawn(async move {
                let _ = Self::process(buf, info, socket).await;
            });
        }

        Ok(())
    }

    async fn process(buf: [u8; 512], info: (usize,SocketAddr), socket: Arc<UdpSocket>) -> Result<()> {
        let useful_bytes = &buf[..info.0];
        let mut reader = DnsReader::from(useful_bytes);

        let mut buf: Vec<u8> = Vec::with_capacity(512);
        let mut writer = DnsWriter::from(&mut buf);

        let msg = reader.read().await?;
        println!("{:?}", msg);

        let resp = Self::build_message(msg);
        println!("{:?}", resp);
        writer.write(resp).await?;
        socket.send_to(&buf, info.1).await?;

        Ok(())
    }

    fn build_message(request: Message) -> Message {
        let builder = MessageBuilder::from_request(request);

        let addr = Ipv4Addr::from([192,168,1,254]);
        let builder = builder.
                            add_typeA_answer("google.pt".to_owned(), addr, 10000);
        builder.build()
    }

}