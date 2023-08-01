use std::{num, net::Ipv4Addr};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, Result};

#[async_trait]
pub trait FromAsyncReader<T: Send>: Sized {
    async fn from(reader: &mut T) -> Result<Self>;

    async fn from_n(reader: &mut T, quantity: usize) -> Result<Vec<Self>> {
        let mut result = Vec::with_capacity(quantity);
        for _ in 0..quantity {
            let elm = Self::from(reader).await?;
            result.push(elm);
        }

        Ok(result)
    }
}

#[async_trait]
pub trait Writable<T: Send>: Sized + Send {
    async fn write(&self, writer: &mut T) -> Result<()>;
}

#[derive(Debug)]
pub(crate) struct Message {
    pub(crate) header: Header,
    pub(crate) questions: Vec<Question>,
    pub(crate) answers: Vec<Record>,
    pub(crate) authority: Vec<Record>,
    pub(crate) resources: Vec<Record>,
}

impl Message {}

#[async_trait]
impl<T: AsyncReadExt + Unpin + Send> FromAsyncReader<T> for Message {
    async fn from(reader: &mut T) -> Result<Message> {
        let header: Header = FromAsyncReader::from(reader).await?;
        let questions = Question::from_n(reader, header.questions as usize).await?;
        let answers = Record::from_n(reader, header.awnsers as usize).await?;
        let authority = Record::from_n(reader, header.authority_entries as usize).await?;
        let resources = Record::from_n(reader, header.ressource_entries as usize).await?;

        return Ok(Message {
            header,
            questions,
            answers,
            authority,
            resources,
        });
    }
}

#[async_trait]
impl<T: AsyncWriteExt + Unpin + Send> Writable<T> for Message {
    async fn write(&self, writer: &mut T) -> Result<()> {
        self.header.write(writer).await?;
        self.questions.write(writer).await?;
        self.answers.write(writer).await?;
        self.authority.write(writer).await?;
        self.resources.write(writer).await?;

        Ok(())
    }
}

#[async_trait]
impl<T, W> Writable<T> for Vec<W>
where
    T: AsyncWriteExt + Send,
    W: Writable<T> + Send + Sync,
{
    async fn write(&self, writer: &mut T) -> Result<()> {
        for elm in self {
            elm.write(writer).await?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ResultCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
    UNKNOWN = 6,
}

impl ResultCode {
    fn from(num: u8) -> ResultCode {
        match num {
            0 => ResultCode::NOERROR,
            1 => ResultCode::FORMERR,
            2 => ResultCode::SERVFAIL,
            3 => ResultCode::NXDOMAIN,
            4 => ResultCode::NOTIMP,
            5 => ResultCode::REFUSED,
            _ => ResultCode::UNKNOWN,
        }
    }

    pub(crate) fn to(&self) -> u8 {
        match self {
            ResultCode::NOERROR => 0,
            ResultCode::FORMERR => 1,
            ResultCode::SERVFAIL => 2,
            ResultCode::NXDOMAIN => 3,
            ResultCode::NOTIMP => 4,
            ResultCode::REFUSED => 5,
            ResultCode::UNKNOWN => 6,
        }
    } 
}

#[derive(Debug)]
pub struct Header {
    pub id: u16,

    pub flags: u16, //Split into multiple fields

    pub questions: u16,
    pub awnsers: u16,
    pub authority_entries: u16,
    pub ressource_entries: u16,
}

impl Header {
    
    fn new() -> Header {
        Header {
            id: 0,
            flags: 0,
            questions: 0,
            awnsers: 0,
            authority_entries: 0,
            ressource_entries: 0,
        }
    }

    pub(crate) fn is_query(&self) -> bool {
        (self.flags & 0b1000000000000000) >> 15 == 0
    }

    pub(crate) fn op_code(&self) -> u8 {
        ((self.flags & 0b0111100000000000) >> 11) as u8
    }

    pub(crate) fn is_authoritative(&self) -> bool {
        (self.flags & 0b0000010000000000) >> 10 == 1
    }

    pub(crate) fn is_truncated(&self) -> bool {
        (self.flags & 0b0000001000000000) >> 9 == 1
    }

    pub(crate) fn is_recursion_desired(&self) -> bool {
        (self.flags & 0b0000000100000000) >> 8 == 1
    }

    pub(crate) fn is_recursion_available(&self) -> bool {
        (self.flags & 0b0000000010000000) >> 7 == 1
    }

    pub(crate) fn result_code(&self) -> ResultCode {
        let result_code = (self.flags & 0b0000000000001111) as u8;
        ResultCode::from(result_code)
    }
}

#[async_trait]
impl<T: AsyncReadExt + Unpin + Send> FromAsyncReader<T> for Header {
    async fn from(reader: &mut T) -> Result<Header> {
        let mut header = Header::new();

        header.id = reader.read_u16().await?;
        header.flags = reader.read_u16().await?;
        header.questions = reader.read_u16().await?;
        header.awnsers = reader.read_u16().await?;
        header.authority_entries = reader.read_u16().await?;
        header.ressource_entries = reader.read_u16().await?;

        Ok(header)
    }
}

#[async_trait]
impl<T: AsyncWriteExt + Unpin + Send> Writable<T> for Header {
    async fn write(&self, writer: &mut T) -> Result<()> {
        writer.write_u16(self.id).await?;
        writer.write_u16(self.flags).await?;
        writer.write_u16(self.questions).await?;
        writer.write_u16(self.awnsers).await?;
        writer.write_u16(self.authority_entries).await?;
        writer.write_u16(self.ressource_entries).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QueryType {
    A,
    AAAA,
    UNKNOWN, // TODO there are more
}

impl QueryType {
    fn from(value: u16) -> QueryType {
        match value {
            1 => QueryType::A,
            28 => QueryType::AAAA,
            _ => QueryType::UNKNOWN,
        }
    }

    fn as_u16(self) -> u16 {
        self.to_u16()
    }

    fn to_u16(&self) -> u16 {
        match self {
            QueryType::A => 1,
            QueryType::AAAA => 28,
            QueryType::UNKNOWN => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Class {
    UNKNOWN,
    RESERVED,
    IN,
    QCLASSNONE,
    QCLASSANY, // TODO there are more
}

impl Class {
    fn from(value: u16) -> Class {
        match value {
            0 => Class::RESERVED,
            1 => Class::IN,
            254 => Class::QCLASSNONE,
            255 => Class::QCLASSANY,
            _ => Class::UNKNOWN,
        }
    }

    fn as_u16(self) -> u16 {
        self.to_u16()
    }

    fn to_u16(&self) -> u16 {
        match self {
            Class::RESERVED => 0,
            Class::IN => 1,
            Class::QCLASSNONE => 254,
            Class::QCLASSANY => 255,
            Class::UNKNOWN => 256,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Question {
    pub name: String,
    pub r#type: QueryType,
    pub class: Class,
}

impl Question {
    fn new() -> Question {
        Question {
            name: String::new(),
            r#type: QueryType::UNKNOWN,
            class: Class::UNKNOWN,
        }
    }
}

#[async_trait]
impl<T: AsyncReadExt + Unpin + Send> FromAsyncReader<T> for Question {
    async fn from(reader: &mut T) -> Result<Question> {
        let mut question = Question::new();

        read_dns_encoded_name(reader, &mut question.name).await?;

        let r#type = reader.read_u16().await?;
        question.r#type = QueryType::from(r#type);

        let class = reader.read_u16().await?;
        question.class = Class::from(class);

        Ok(question)
    }
}

#[async_trait]
impl<T: AsyncWriteExt + Unpin + Send> Writable<T> for Question {
    async fn write(&self, writer: &mut T) -> Result<()> {
        write_dns_encoded_name(writer, &self.name).await?;
        writer.write_u16(self.r#type.to_u16()).await?;
        writer.write_u16(self.class.to_u16()).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum Record {
    UNKNOWN {
        name: String,
        r#type: u16,
        class: Class,
        ttl: u32,
        len: u16
    },
    A {
        name: String,
        class: Class,
        addr: Ipv4Addr,
        ttl: u32
    }
}

impl Record {
    pub(crate) fn new_type_a(name: String, addr: Ipv4Addr, ttl: u32) -> Record {
        Record::A {
            name: name,
            class: Class::IN,
            ttl,
            addr,
        }
    }
}

#[async_trait]
impl<T: AsyncReadExt + Unpin + Send> FromAsyncReader<T> for Record {
    async fn from(reader: &mut T) -> Result<Record> {
        let mut name = String::new();
        read_dns_encoded_name(reader, &mut name).await?;

        let qtype_u16 = reader.read_u16().await?;
        let qtype = QueryType::from(qtype_u16);
        let class = reader.read_u16().await?;
        let class = Class::from(class);
        let ttl = reader.read_u32().await?;
        let len = reader.read_u16().await?;

        let res = match qtype {
            QueryType::A => {
                let a = reader.read_u8().await?;
                let b = reader.read_u8().await?;
                let c = reader.read_u8().await?;
                let d = reader.read_u8().await?;
                let addr = Ipv4Addr::new(a,b,c,d);

                Self::A {
                    name,
                    class,
                    addr,
                    ttl
                }
            },
            qtype => Self::UNKNOWN {
                name,
                r#type: qtype_u16,
                class,
                ttl,
                len
            },
        };

        Ok(res)
    }
}


#[async_trait]
impl<T: AsyncWriteExt + Unpin + Send> Writable<T> for Record {
    async fn write(&self, writer: &mut T) -> Result<()> {
        match self {
            Record::UNKNOWN { name, r#type, class, ttl,len } => {
                write_dns_encoded_name(writer, name).await?;
                writer.write_u16(*r#type).await?;
                writer.write_u16(class.to_u16()).await?;
                writer.write_u32(*ttl).await?;
                writer.write_u16(*len).await?;

            },
            Record::A { name, class, addr, ttl } => {
                write_dns_encoded_name(writer, name).await?;
                writer.write_u16(1).await?;
                writer.write_u16(class.to_u16()).await?;
                writer.write_u32(*ttl).await?;
                writer.write_u16(4).await?;
                

                let bytes = addr.octets();
                writer.write_u8(bytes[0]).await?;
                writer.write_u8(bytes[1]).await?;
                writer.write_u8(bytes[2]).await?;
                writer.write_u8(bytes[3]).await?;
            },
        };

        
        Ok(())
    }
}

/**
 * QNAME has the following format:
 * 0x03 -> String of lenght 3 follows
 * 0x777777 -> String is www
 * 0x0c -> String of lenght 12 follows
 * 0x6e6f7274686561737465726e -> notherastern
 * 0x03 -> String of lenght 3 follows
 * 0x656475 -> String is edu
 * 0x00 -> End of this name
 */
pub async fn read_dns_encoded_name<T>(reader: &mut T, str: &mut String) -> Result<()>
where
    T: AsyncReadExt + Unpin + Send,
{
    const SPLIT: char = '.';

    while let Ok(lenght) = reader.read_u8().await {
        if lenght == 0 {
            break;
        }

        if str.len() > 0 {
            str.push(SPLIT);
        }

        let mut pos = 0;
        while lenght - pos > 0 {
            let c = reader.read_u8().await?;
            str.push(c as char);
            pos += 1;
        }
    }

    Ok(())
}

pub async fn write_dns_encoded_name<T>(writer: &mut T, str: &str) -> Result<()>
where
    T: AsyncWriteExt + Unpin + Send,
{
    const SPLIT: char = '.';

    for word in str.split(SPLIT) {
        writer.write_u8(word.len() as u8).await?;
        for c in word.chars() {
            writer.write_u8(c as u8).await?;
        }
    }

    writer.write_u8(0).await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use tokio::io::BufReader;

    use super::{
        read_dns_encoded_name, write_dns_encoded_name, Class, FromAsyncReader, Message, ResultCode,
        QueryType, Writable,
    };

    #[tokio::test]
    async fn deserialize_request_message() {
        let mut file = tokio::fs::File::open("./src/core/dns/test/query_packet.txt")
            .await
            .unwrap();

        let message: Message = FromAsyncReader::from(&mut file).await.unwrap();

        // Message should have id 5475 and one question
        assert_eq!(message.header.id, 5475);
        assert_eq!(message.header.questions, 1);
        assert_eq!(message.questions.len(), 1);
        assert_eq!(message.header.awnsers, 0);
        assert_eq!(message.header.authority_entries, 0);
        assert_eq!(message.header.ressource_entries, 0);

        // Verify flags
        assert!(message.header.is_query());
        assert_eq!(message.header.op_code(), 0);
        assert_eq!(message.header.is_authoritative(), false);
        assert_eq!(message.header.is_truncated(), false);
        assert_eq!(message.header.is_recursion_desired(), true);
        assert_eq!(message.header.is_recursion_available(), false);
        assert_eq!(message.header.result_code(), ResultCode::NOERROR);

        // Question should be for google.com of type A and class IN
        let question = &message.questions[0];
        assert_eq!(question.name, "google.com");
        assert_eq!(question.r#type, QueryType::A);
        assert_eq!(question.class, Class::IN);

        println!("{:?}", message);
    }

    #[tokio::test]
    async fn deseliaze_request_message() {
        let expects = tokio::fs::read("./src/core/dns/test/query_packet.txt")
            .await
            .unwrap();
        let mut reader = BufReader::new(&*expects);

        let message: Message = FromAsyncReader::from(&mut reader).await.unwrap();

        let mut result = Vec::new();
        message.write(&mut result).await.unwrap();

        assert_eq!(expects, result);
        println!("{:?}", message);
    }

    #[tokio::test]
    async fn test_read_qname() {
        let hex: Vec<u8> = vec![
            0x03, 0x77, 0x77, 0x77, 0x0c, 0x6e, 0x6f, 0x72, 0x74, 0x68, 0x65, 0x61, 0x73, 0x74,
            0x65, 0x72, 0x6e, 0x03, 0x65, 0x64, 0x75, 0x00,
        ];

        let mut reader = tokio::io::BufReader::new(&*hex);
        let mut buff = String::new();
        read_dns_encoded_name(&mut reader, &mut buff).await.unwrap();

        assert_eq!(buff, "www.northeastern.edu")
    }

    #[tokio::test]
    async fn test_write_qname() {
        let given = "www.northeastern.edu";
        let expects: Vec<u8> = vec![
            0x03, 0x77, 0x77, 0x77, 0x0c, 0x6e, 0x6f, 0x72, 0x74, 0x68, 0x65, 0x61, 0x73, 0x74,
            0x65, 0x72, 0x6e, 0x03, 0x65, 0x64, 0x75, 0x00,
        ];

        let mut result = Vec::<u8>::new();
        write_dns_encoded_name(&mut result, given).await.unwrap();
        assert_eq!(result, expects);
    }
}
