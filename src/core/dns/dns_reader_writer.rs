use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};

use super::message::{FromAsyncReader, Header, Message, Writable};

struct DnsReader<T> {
    reader: T,
}

impl<T: AsyncReadExt + Unpin + Send> DnsReader<T> {
    pub fn from(reader: T) -> DnsReader<T> {
        DnsReader { reader }
    }

    pub async fn read(&mut self) -> Result<Message> {
        let mut reader = &mut self.reader;
        FromAsyncReader::from(&mut reader).await
    }
}

struct DnsWriter<T> {
    writer: T,
}

impl<T: AsyncWriteExt + Unpin + Send> DnsWriter<T> {
    pub fn from(writer: T) -> DnsWriter<T> {
        DnsWriter { writer }
    }

    pub async fn write<W: Writable<T>>(&mut self, w: W) -> Result<()> {
        w.write(&mut self.writer).await?;
        Ok(())
    }
}
//TODO packet writer

#[cfg(test)]
mod test {
    /*     use tokio::fs::File;
    use super::UdpPacketReader;


    #[tokio::test]
    async fn from_file() {
        let f = File::open("foo.txt").await.unwrap();
        let reader = UdpPacketReader::from(f);
    }*/
}
