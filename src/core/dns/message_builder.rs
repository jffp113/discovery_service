use std::{marker::PhantomData, net::Ipv4Addr};

use super::message::{Message, Header, Question, QueryType, Class, Record, ResultCode};

pub struct Request;
pub struct Response;

pub struct MessageBuilder<T> {
    id: u16,
    flags: u16,
    questions: Vec<Question>,
    answers: Vec<Record>,
    authority: Vec<Record>,
    resources: Vec<Record>,

    phantom: PhantomData<T>
}

impl MessageBuilder<Response> {
    pub(crate) fn from_request(message: Message) -> MessageBuilder<Response> {
        MessageBuilder{
            id: message.header.id,
            flags: 0b1000000000000000,
            questions: message.questions,
            answers: Vec::new(),
            authority: Vec::new(),
            resources: Vec::new(),
            phantom: PhantomData::default()
        }
    }

    pub(crate) fn set_is_authoritive(mut self) -> Self {
        self.flags = self.flags | 0b0000010000000000;
        self
    }

    pub(crate) fn set_recursive_available(mut self) -> Self {
        self.flags = self.flags | 0b0000000010000000;
        self
    }

    pub(crate) fn set_status_code(mut self, rcode: ResultCode) -> Self {
        let code = rcode.to();
        self.flags = self.flags & 0b1111111111111111;
        self.flags = self.flags & 0b1111111111111111;
        self
    }

    pub(crate) fn set_answers(mut self, answers: Vec<Record>) -> Self {
        self.answers = answers;
        self
    }

    pub(crate) fn add_answers(mut self, answer: Record) -> Self {
        self.answers.push(answer);
        self
    }

    pub(crate) fn add_typeA_answer(mut self, name: String, addr: Ipv4Addr, ttl: u32) -> Self {
        self.answers.push(Record::new_type_a(name, addr, ttl));
        self
    }


    pub(crate) fn set_authority(mut self, authority: Vec<Record>) -> Self {
        self.authority = authority;
        self
    }

    pub(crate) fn add_authority(mut self, authority: Record) -> Self {
        self.authority.push(authority);
        self
    }

    pub(crate) fn add_typeA_authority(mut self, name: String, addr: Ipv4Addr, ttl: u32) -> Self {
        self.authority.push(Record::new_type_a(name, addr, ttl));
        self
    }



    pub(crate) fn set_resources(mut self, resources: Vec<Record>) -> Self {
        self.resources = resources;
        self
    }

    pub(crate) fn add_resources(mut self, resources: Record) -> Self {
        self.resources.push(resources);
        self
    }

    pub(crate) fn add_typeA_resources(mut self, name: String, addr: Ipv4Addr, ttl: u32) -> Self {
        self.resources.push(Record::new_type_a(name, addr, ttl));
        self
    }

}


impl MessageBuilder<Request> {
    pub(crate) fn new_request(id: u16) -> MessageBuilder<Request> {
        MessageBuilder {
            id,
            flags: 0b0000000000000000,
            questions: Vec::new(),
            answers: Vec::new(),
            authority: Vec::new(),
            resources: Vec::new(),
            phantom: PhantomData::default(),
        }
    }

    pub(crate) fn set_recursion_desired(mut self) -> Self {
        self.flags = self.flags | 0b0000000100000000;
        self
    }

    pub(crate) fn set_questions(mut self, questions: Vec<Question>) -> Self {
        self.questions = questions;
        self
    }

    pub(crate) fn add_question(mut self, question: Question) -> Self {
        self.questions.push(question);
        self
    }

    pub(crate) fn add_new_question(mut self, name: String, r#type: QueryType, class: Class) -> Self {
        self.questions.push(Question{
            name,
            r#type,
            class
        });
        self
    }
}

impl<T> MessageBuilder<T> {
    pub(crate) fn build(self) -> Message {
        Message{
            header: Header {
                id: self.id,
                flags: self.flags,
                questions: self.questions.len() as u16,
                awnsers: self.answers.len() as u16,
                authority_entries: self.authority.len() as u16,
                ressource_entries: self.resources.len() as u16,
            },
            questions: self.questions,
            answers: self.answers,
            authority: self.authority,
            resources: self.resources,
        }
    }
}