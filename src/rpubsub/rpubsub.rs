extern crate strum;
extern crate strum_macros;
extern crate serde;
extern crate serde_json;
extern crate sha2;

use std::{collections::HashMap, fmt::format};

use serde::{Serialize, Deserialize};

use strum_macros::{IntoStaticStr};

pub struct SocketAddress {
    pub ip:   String,
    pub port: u16,
}

pub type Topic = String;
pub type SequenceNum = u128; 
pub type UpdateContent = String;
pub type MessageHash = String;

#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
pub enum Message {
    GET   { ip: String, topic: Topic, sequence_num: SequenceNum },
    PUT   { ip: String, topic: Topic, sequence_num: SequenceNum, payload: UpdateContent },
    SUB   { ip: String, topic: Topic },
    UNSUB { ip: String, topic: Topic },
    UP    { ip: String, sequence_nums: HashMap<Topic, SequenceNum> },
    REP   { result: Result<ReplyOption, ServiceError> },
    NOMSG
}

#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
pub enum ReplyOption {
    NoOk,
    TUP((Option<UpdateContent>, SequenceNum))
}

pub enum IOError {
    ECON(zmq::Error),
    EBIN(zmq::Error),
    ERCV(zmq::Error),
    ESND(zmq::Error),
    EDSL(serde_json::Error),
}

impl IOError {
    pub fn to_string(&self) -> String {
        match self {
            IOError::ECON(e) => format!("error: couldn't connect to the socket - {}", e),
            IOError::EBIN(e) => format!("error: couldn't bind the socket - {}", e),
            IOError::ERCV(e) => format!("error: couldn't receive message - {}", e),
            IOError::ESND(e) => format!("error: couldn't send message - {}", e),
            IOError::EDSL(e) => format!("error: received unknown message - {}", e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
pub enum ServiceError {
    NOTOPIC(Topic),
    NOSUB(Topic),
    ALREASUB(Topic),
    ALREAPUT,
    UNKNOMSG
}

impl ServiceError {
    pub fn to_string(&self) -> String {
        match self {
            ServiceError::NOTOPIC(topic) => format!("error: topic {} doesn't exist", topic),
            ServiceError::NOSUB(topic) => format!("error: not subscribed to topic {}", topic),
            ServiceError::ALREASUB(topic) => format!("error: already subscribed to topic {}", topic),
            ServiceError::ALREAPUT => todo!(),
            ServiceError::UNKNOMSG => String::from("error: unknown request"),
        }
    }
}

impl Message {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn bind_to(socket: &zmq::Socket, socket_address: &SocketAddress) -> Result<(), IOError> {
    let endpoint = format!("tcp://{}:{}", socket_address.ip, socket_address.port);

    match socket.bind(&endpoint) {
        Ok(()) => Ok(()),

        Err(e) => Err(IOError::EBIN(e))
    }
}

pub fn connect_to(socket: &zmq::Socket, socket_address: &SocketAddress) -> Result<(), IOError> {
    let endpoint = format!("tcp://{}:{}", socket_address.ip, socket_address.port);

    match socket.connect(&endpoint) {
        Ok(()) => Ok(()),

        Err(e) => Err(IOError::ECON(e))
    }
}

pub fn send_message_to(socket: &zmq::Socket, message: &Message) -> Result<(), IOError> {
    let serialized_message = serde_json::to_string(message).unwrap();
    
    return match socket.send(serialized_message.as_str(), 0) {
        Ok(_) => Ok(()),
        Err(e) => Err(IOError::ESND(e)),
    };
    
}

pub fn receive_message_from(socket: &zmq::Socket) -> Result<Message, IOError> {
    let mut msg = zmq::Message::new();

    match socket.recv(&mut msg, 0) {
        Ok(_) => (),
        Err(e) => return Err(IOError::ERCV(e)),
    };
    
    let res: Result<Message, serde_json::Error> = serde_json::from_str(&msg.as_str().unwrap());
    
    return match res {
        Ok(message) => Ok(message),
        Err(e) => Err(IOError::EDSL(e)),
    }
}
