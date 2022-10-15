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

#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
pub enum Message {
    GET   { ip: String, sequence_num: u128, topic: String },
    PUT   { ip: String, sequence_num: u128, topic: String, payload: String },
    SUB   { ip: String, topic: String },
    UNSUB { ip: String, topic: String },
    UP    { ip: String, sequence_num: u128},
    REP   { ip: String, status: u8 }
}

impl Message {
    pub fn to_string(&self) -> &str {
        return <&Message as Into<&str>>::into(self) as &str;
    }
}