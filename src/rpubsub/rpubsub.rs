pub struct ServerAddress {
    pub ip: String,
    pub bind_port: u16,
}

pub struct MessageType {}

impl MessageType {
    const GET: &str = "GET";
    const PUT: &str = "PUT";
    const SUB: &str = "SUB";
    const UNSUB: &str = "UNSUB";
    const RECOVER: &str = "RECOVER";
}
