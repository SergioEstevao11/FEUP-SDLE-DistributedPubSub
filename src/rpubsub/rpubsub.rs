
#![allow(dead_code)]
#![allow(unused_variables)]
extern crate strum;
extern crate strum_macros;
extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};
use strum_macros::{IntoStaticStr};

pub struct SocketAddress {
    pub ip:   String,
    pub port: u16,
}

pub type Topic = String;

#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
pub enum Message {
    GET   { ip: String, sequence_num: u128, topic: Topic },
    PUT   { ip: String, sequence_num: u128, topic: Topic, payload: String },
    SUB   { ip: String, topic: Topic },
    UNSUB { ip: String, topic: Topic },
    UP    { ip: String, sequence_num: u128},
    REP   { ip: String, status: u8 }
}

pub enum Error {
    ERCV(zmq::Error),
    ESND(zmq::Error),
    EDSL(serde_json::Error),
}

impl Message {
    pub fn to_string(&self) -> &str {
        return <&Message as Into<&str>>::into(self) as &str;
    }
}

pub fn send_message_to(socket: &zmq::Socket, message: Message) -> Result<(), Error>{
    let serialized_message = serde_json::to_string(&message).unwrap();
    
    return match socket.send(serialized_message.as_str(), 0) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::ESND(e)),
    };
    
}

pub fn receive_message_from(socket: &zmq::Socket) -> Result<Message, Error>{
    let mut msg = zmq::Message::new();

    match socket.recv(&mut msg, 0) {
        Ok(_) => (),
        Err(e) => return Err(Error::ERCV(e)),
    };
    
    let res: Result<Message, serde_json::Error> = serde_json::from_str(&msg.as_str().unwrap());
    
    return match res {
        Ok(message) => Ok(message),
        Err(e) => Err(Error::EDSL(e)),
    }
}
