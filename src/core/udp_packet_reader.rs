use tokio::io::AsyncReadExt;

use super::message::{Message, Header};


struct UdpPacketReader<T: AsyncReadExt + Unpin> {
    //buf: [u8; 512],
    //pos: usize
    reader: T
}

impl<T: AsyncReadExt + Unpin> UdpPacketReader<T> {

    pub fn from(reader: T) -> UdpPacketReader<T> {
        UdpPacketReader{
            reader
        }
    }

    pub fn read_dns_message(&mut self) -> Message {
        let mut reader = &mut self.reader;
        let message = Header::from(&mut reader);
        
        todo!();
    }
}

#[cfg(test)]
mod test{
/*     use tokio::fs::File;
    use super::UdpPacketReader;


    #[tokio::test]
    async fn from_file() {
        let f = File::open("foo.txt").await.unwrap();
        let reader = UdpPacketReader::from(f);
    }*/
}