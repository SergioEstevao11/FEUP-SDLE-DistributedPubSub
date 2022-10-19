extern crate strum;
extern crate strum_macros;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use strum_macros::{IntoStaticStr};

pub struct SocketAddress {
    pub ip:   String,
    pub port: u16,
}

pub type Topic = String;
pub type SequenceNum = u128; 
pub type UpdateContent = String;

#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
pub enum Message {
    GET   { ip: String, sequence_num: SequenceNum, topic: Topic },
    PUT   { ip: String, sequence_num: SequenceNum, topic: Topic, payload: UpdateContent },
    SUB   { ip: String, topic: Topic },
    UNSUB { ip: String, topic: Topic },
    UP    { ip: String, sequence_nums: HashMap<Topic, SequenceNum> },
    REP   { ip: String, result: Result<(UpdateContent, SequenceNum), ServiceError> }
}

pub enum IOError {
    ERCV(zmq::Error),
    ESND(zmq::Error),
    EDSL(serde_json::Error),
}

#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
pub enum ServiceError {
    NOTOPIC,
    NOSUB,
    ALREASUB,
}

impl Message {
    pub fn to_string(&self) -> &str {
        return <&Message as Into<&str>>::into(self) as &str;
    }
}

pub fn send_message_to(socket: &zmq::Socket, message: Message) -> Result<(), IOError> {
    let serialized_message = serde_json::to_string(&message).unwrap();
    
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
